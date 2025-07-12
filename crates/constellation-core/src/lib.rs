/*
 * Constellation Studio - Professional Real-time Video Processing
 * Copyright (c) 2025 MACHIKO LAB
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

pub mod error;
pub mod hardware;
pub mod resilience;
pub mod telemetry;
use constellation_vulkan::{MemoryManager, VulkanContext};
pub use error::{ConstellationError, ConstellationResult, ErrorCategory, ErrorSeverity};
pub use hardware::{
    CompatibilityLevel, CompatibilityReport, HardwareCompatibilityChecker, SystemInfo,
};
pub use resilience::{HealthMonitor, RecoveryAction, ResilienceManager, SystemStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
pub use telemetry::{MetricValue, SessionStats, TelemetryManager};
use uuid::Uuid;

pub struct ConstellationEngine {
    #[allow(dead_code)]
    vulkan_context: VulkanContext,
    #[allow(dead_code)]
    memory_manager: MemoryManager,
    node_graph: NodeGraph,
    frame_processors: Vec<FrameProcessor>,
    resilience_manager: Option<ResilienceManager>,
    telemetry_manager: TelemetryManager,
    hardware_checker: HardwareCompatibilityChecker,
}

impl ConstellationEngine {
    pub fn new() -> ConstellationResult<Self> {
        let vulkan_context = VulkanContext::new().map_err(|e| match e {
            constellation_vulkan::VulkanError::InitializationFailed { reason } => {
                ConstellationError::EngineInitializationFailed { reason }
            }
            constellation_vulkan::VulkanError::DeviceCreationFailed { reason } => {
                ConstellationError::EngineInitializationFailed { reason }
            }
            constellation_vulkan::VulkanError::HardwareNotSupported { hardware } => {
                ConstellationError::HardwareNotSupported { hardware }
            }
            constellation_vulkan::VulkanError::InsufficientMemory { required_bytes } => {
                ConstellationError::InsufficientMemory { required_bytes }
            }
            constellation_vulkan::VulkanError::GpuProcessingFailed { reason } => {
                ConstellationError::GpuProcessingFailed { reason }
            }
        })?;
        let memory_manager = MemoryManager::new(&vulkan_context).map_err(|e| match e {
            constellation_vulkan::VulkanError::InitializationFailed { reason } => {
                ConstellationError::EngineInitializationFailed { reason }
            }
            constellation_vulkan::VulkanError::DeviceCreationFailed { reason } => {
                ConstellationError::EngineInitializationFailed { reason }
            }
            constellation_vulkan::VulkanError::HardwareNotSupported { hardware } => {
                ConstellationError::HardwareNotSupported { hardware }
            }
            constellation_vulkan::VulkanError::InsufficientMemory { required_bytes } => {
                ConstellationError::InsufficientMemory { required_bytes }
            }
            constellation_vulkan::VulkanError::GpuProcessingFailed { reason } => {
                ConstellationError::GpuProcessingFailed { reason }
            }
        })?;
        let node_graph = NodeGraph::new();
        let frame_processors = Vec::new();

        // ハードウェア互換性チェック
        let mut hardware_checker = HardwareCompatibilityChecker::new()?;
        let compatibility_report = hardware_checker.check_compatibility()?;

        // 互換性チェック結果をログに記録
        tracing::info!(
            compatibility = ?compatibility_report.overall_compatibility,
            supported_phases = ?compatibility_report.supported_phases,
            "Hardware compatibility checked"
        );

        if matches!(
            compatibility_report.overall_compatibility,
            CompatibilityLevel::NotSupported
        ) {
            return Err(ConstellationError::HardwareNotSupported {
                hardware: "System does not meet minimum requirements for any phase".to_string(),
            });
        }

        Ok(Self {
            vulkan_context,
            memory_manager,
            node_graph,
            frame_processors,
            resilience_manager: None, // 後で初期化
            telemetry_manager: TelemetryManager::new(),
            hardware_checker,
        })
    }

    /// レジリエンス機能を有効化
    pub fn enable_resilience(&mut self) -> ConstellationResult<()> {
        let engine_ref = std::sync::Arc::new(unsafe {
            // 注意: これは安全でない操作です。本来は適切な設計でArcを共有する必要があります
            std::ptr::read(self as *const Self)
        });
        self.resilience_manager = Some(ResilienceManager::new(engine_ref));
        Ok(())
    }

    pub fn process_frame(&mut self, input: &FrameData) -> ConstellationResult<FrameData> {
        let frame_id = Uuid::new_v4();
        let _frame_span = self.telemetry_manager.start_frame_processing(frame_id);

        let start_time = std::time::Instant::now();
        let mut current_frame = input.clone();

        for processor in &mut self.frame_processors {
            match processor.process(&current_frame) {
                Ok(frame) => {
                    current_frame = frame;
                }
                Err(error) => {
                    // エラーをテレメトリに記録
                    self.telemetry_manager.record_error(&error, None);

                    // エラーハンドリングと復旧処理
                    if let Some(ref mut resilience_manager) = self.resilience_manager {
                        match resilience_manager.handle_error(&error) {
                            Ok(RecoveryAction::Retry {
                                max_attempts,
                                delay,
                                backoff_multiplier,
                            }) => {
                                // 再試行ロジック
                                let mut attempts = 0;
                                let mut current_delay = delay;

                                while attempts < max_attempts {
                                    std::thread::sleep(current_delay);
                                    attempts += 1;
                                    current_delay = Duration::from_millis(
                                        (current_delay.as_millis() as f32 * backoff_multiplier)
                                            as u64,
                                    );

                                    match processor.process(&current_frame) {
                                        Ok(frame) => {
                                            current_frame = frame;
                                            break;
                                        }
                                        Err(retry_error) if attempts >= max_attempts => {
                                            return Err(retry_error);
                                        }
                                        Err(_) => {
                                            // 次の試行へ続行
                                        }
                                    }
                                }
                            }
                            Ok(RecoveryAction::QualityReduced) => {
                                // 品質低下モードで続行
                                tracing::warn!(
                                    "Processing in reduced quality mode due to error: {}",
                                    error
                                );
                                // 簡略化された処理を続行
                            }
                            Ok(RecoveryAction::Fallback {
                                processor: fallback_processor,
                                ..
                            }) => {
                                // フォールバックプロセッサを使用
                                let mut fallback =
                                    FrameProcessor::new(Uuid::new_v4(), fallback_processor);
                                current_frame = fallback.process(&current_frame)?;
                            }
                            Ok(RecoveryAction::GracefulShutdown { .. }) => {
                                // システム停止
                                return Err(ConstellationError::EngineNotRunning);
                            }
                            Ok(RecoveryAction::LogAndContinue) => {
                                // エラーをログに記録して続行
                                tracing::error!("Frame processing error (continuing): {}", error);
                            }
                            Err(recovery_error) => {
                                // 復旧自体に失敗
                                return Err(recovery_error);
                            }
                        }
                    } else {
                        // レジリエンス機能が無効の場合は従来通りエラーを返す
                        return Err(error);
                    }
                }
            }
        }

        // パフォーマンス監視とメトリクス記録
        let processing_time = start_time.elapsed();

        // テレメトリにフレーム統計を記録
        self.telemetry_manager
            .metrics_collector
            .frame_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.telemetry_manager
            .metrics_collector
            .total_processing_time
            .fetch_add(
                processing_time.as_micros() as u64,
                std::sync::atomic::Ordering::Relaxed,
            );

        // レジリエンス監視
        if let Some(ref mut resilience_manager) = self.resilience_manager {
            resilience_manager.monitor_performance(&current_frame, processing_time);
        }

        Ok(current_frame)
    }

    pub fn add_node(
        &mut self,
        node_type: NodeType,
        config: NodeConfig,
    ) -> ConstellationResult<Uuid> {
        let node_id = Uuid::new_v4();
        let node = Node::new(node_id, node_type, config);
        self.node_graph.add_node(node);
        Ok(node_id)
    }

    pub fn connect_nodes(
        &mut self,
        source_id: Uuid,
        target_id: Uuid,
        connection_type: ConnectionType,
    ) -> ConstellationResult<()> {
        self.node_graph
            .connect_nodes(source_id, target_id, connection_type)
    }

    /// セッション統計の取得
    pub fn get_session_stats(&self) -> SessionStats {
        self.telemetry_manager.get_session_stats()
    }

    /// カスタムメトリクスの記録
    pub fn record_metric(&self, name: String, value: MetricValue) {
        self.telemetry_manager.record_metric(name, value);
    }

    /// システム状態の記録
    pub fn record_system_state(&self, cpu_usage: f32, memory_usage: u64, gpu_usage: f32) {
        self.telemetry_manager
            .record_system_state(cpu_usage, memory_usage, gpu_usage);
    }

    /// ログの書き出し（JSON形式）
    pub fn export_logs_json(&self) -> serde_json::Result<String> {
        self.telemetry_manager.export_logs_json()
    }

    /// パフォーマンストレースの書き出し
    pub fn export_traces_json(&self) -> serde_json::Result<String> {
        self.telemetry_manager.export_traces_json()
    }

    /// システム情報の取得
    pub fn get_system_info(&self) -> &SystemInfo {
        self.hardware_checker.get_system_info()
    }

    /// ハードウェア互換性レポートの取得
    pub fn get_compatibility_report(&self) -> Option<&CompatibilityReport> {
        self.hardware_checker.get_compatibility_report()
    }

    /// ハードウェア互換性レポートのJSON出力
    pub fn export_hardware_report_json(&self) -> ConstellationResult<String> {
        self.hardware_checker.export_report_json()
    }

    /// 現在サポートされているフェーズの取得
    pub fn get_supported_phases(&self) -> Vec<String> {
        if let Some(report) = self.hardware_checker.get_compatibility_report() {
            report.supported_phases.clone()
        } else {
            vec![]
        }
    }

    /// 特定フェーズの要件チェック
    pub fn can_run_phase(&self, phase: &str) -> bool {
        self.get_supported_phases().contains(&phase.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct FrameData {
    pub render_data: Option<RenderData>,
    pub audio_data: Option<UnifiedAudioData>,
    pub control_data: Option<ControlData>,
    // Tally自動伝播用メタデータ
    pub tally_metadata: TallyMetadata,
}

#[derive(Debug, Clone)]
pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    pub format: VideoFormat,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct AudioFrame {
    pub sample_rate: u32,
    pub channels: u16,
    pub samples: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct TallyData {
    pub program_tally: bool,
    pub preview_tally: bool,
    pub custom_tally: HashMap<String, bool>,
}

// Tally自動伝播システム
#[derive(Debug, Clone, Default)]
pub struct TallyMetadata {
    // 現在のノードのTally状態
    pub program_tally: bool,
    pub preview_tally: bool,
    pub custom_tally: HashMap<String, bool>,

    // 伝播履歴（無限ループ防止）
    pub propagation_path: Vec<Uuid>,

    // 伝播制御フラグ
    pub should_propagate: bool,
    pub propagation_source: Option<Uuid>,
}

impl TallyMetadata {
    pub fn new() -> Self {
        Self {
            program_tally: false,
            preview_tally: false,
            custom_tally: HashMap::new(),
            propagation_path: Vec::new(),
            should_propagate: true,
            propagation_source: None,
        }
    }

    pub fn with_program_tally(mut self, enabled: bool) -> Self {
        self.program_tally = enabled;
        self
    }

    pub fn with_preview_tally(mut self, enabled: bool) -> Self {
        self.preview_tally = enabled;
        self
    }

    pub fn add_to_path(&mut self, node_id: Uuid) {
        self.propagation_path.push(node_id);
    }

    pub fn has_visited(&self, node_id: Uuid) -> bool {
        self.propagation_path.contains(&node_id)
    }

    pub fn merge_with(&mut self, other: &TallyMetadata) {
        // OR演算でTally状態をマージ
        self.program_tally |= other.program_tally;
        self.preview_tally |= other.preview_tally;

        // カスタムTallyもマージ
        for (key, value) in &other.custom_tally {
            let current = self.custom_tally.get(key).copied().unwrap_or(false);
            self.custom_tally.insert(key.clone(), current | *value);
        }
    }
}

// 新しい統合データ構造

#[derive(Debug, Clone)]
pub enum RenderData {
    // 2D最終画像
    Raster2D(VideoFrame),

    // 3Dシーン（Phase 4）
    Scene3D(Scene3DData),

    // 中間表現（Vulkan中間状態）
    Intermediate {
        gpu_buffers: Vec<u32>, // VulkanBufferのID参照（簡素化）
        render_state: String,  // レンダリング状態（簡素化）
        transform_matrix: [f32; 16],
    },
}

#[derive(Debug, Clone)]
pub enum UnifiedAudioData {
    Stereo {
        sample_rate: u32,
        channels: u16,
        samples: Vec<f32>,
    },
    Spatial {
        sources: Vec<SpatialAudioSource>,
        listener: AudioListener,
        room_response: Option<Vec<f32>>,
    },
}

#[derive(Debug, Clone)]
pub struct AudioListener {
    pub position: Vector3,
    pub orientation: Vector3,
    pub up: Vector3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioLevel {
    pub peak_left: f32,
    pub peak_right: f32,
    pub rms_left: f32,
    pub rms_right: f32,
    pub db_peak_left: f32,
    pub db_peak_right: f32,
    pub db_rms_left: f32,
    pub db_rms_right: f32,
    pub is_clipping: bool,
    pub timestamp: u64,
}

impl Default for AudioLevel {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioLevel {
    pub fn new() -> Self {
        Self {
            peak_left: 0.0,
            peak_right: 0.0,
            rms_left: 0.0,
            rms_right: 0.0,
            db_peak_left: -f32::INFINITY,
            db_peak_right: -f32::INFINITY,
            db_rms_left: -f32::INFINITY,
            db_rms_right: -f32::INFINITY,
            is_clipping: false,
            timestamp: 0,
        }
    }

    /// Convert linear amplitude to decibels
    pub fn linear_to_db(linear: f32) -> f32 {
        if linear <= 0.0 {
            -f32::INFINITY
        } else {
            20.0 * linear.log10()
        }
    }

    /// Calculate audio levels from UnifiedAudioData
    pub fn from_audio_data(audio_data: &UnifiedAudioData) -> Self {
        match audio_data {
            UnifiedAudioData::Stereo { samples, channels, .. } => {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;

                match *channels {
                    1 => {
                        // Mono: use same values for both channels
                        let (peak, rms) = Self::calculate_peak_rms(samples);
                        let db_peak = Self::linear_to_db(peak);
                        let db_rms = Self::linear_to_db(rms);
                        
                        Self {
                            peak_left: peak,
                            peak_right: peak,
                            rms_left: rms,
                            rms_right: rms,
                            db_peak_left: db_peak,
                            db_peak_right: db_peak,
                            db_rms_left: db_rms,
                            db_rms_right: db_rms,
                            is_clipping: peak >= 1.0,
                            timestamp,
                        }
                    }
                    2 => {
                        // Stereo: separate left and right channels
                        let (left_samples, right_samples) = Self::deinterleave_stereo(samples);
                        let (peak_left, rms_left) = Self::calculate_peak_rms(&left_samples);
                        let (peak_right, rms_right) = Self::calculate_peak_rms(&right_samples);
                        
                        Self {
                            peak_left,
                            peak_right,
                            rms_left,
                            rms_right,
                            db_peak_left: Self::linear_to_db(peak_left),
                            db_peak_right: Self::linear_to_db(peak_right),
                            db_rms_left: Self::linear_to_db(rms_left),
                            db_rms_right: Self::linear_to_db(rms_right),
                            is_clipping: peak_left >= 1.0 || peak_right >= 1.0,
                            timestamp,
                        }
                    }
                    _ => {
                        // Multi-channel: mix down to stereo for simplicity
                        let (left_mix, right_mix) = Self::mixdown_to_stereo(samples, *channels);
                        let (peak_left, rms_left) = Self::calculate_peak_rms(&left_mix);
                        let (peak_right, rms_right) = Self::calculate_peak_rms(&right_mix);
                        
                        Self {
                            peak_left,
                            peak_right,
                            rms_left,
                            rms_right,
                            db_peak_left: Self::linear_to_db(peak_left),
                            db_peak_right: Self::linear_to_db(peak_right),
                            db_rms_left: Self::linear_to_db(rms_left),
                            db_rms_right: Self::linear_to_db(rms_right),
                            is_clipping: peak_left >= 1.0 || peak_right >= 1.0,
                            timestamp,
                        }
                    }
                }
            }
            UnifiedAudioData::Spatial { .. } => {
                // For spatial audio, return silence levels for now
                // TODO: Implement spatial audio level calculation
                Self::new()
            }
        }
    }

    /// Calculate peak and RMS values for a slice of samples
    fn calculate_peak_rms(samples: &[f32]) -> (f32, f32) {
        if samples.is_empty() {
            return (0.0, 0.0);
        }

        let mut peak = 0.0f32;
        let mut sum_squares = 0.0f32;

        for &sample in samples {
            let abs_sample = sample.abs();
            peak = peak.max(abs_sample);
            sum_squares += sample * sample;
        }

        let rms = (sum_squares / samples.len() as f32).sqrt();
        (peak, rms)
    }

    /// Deinterleave stereo samples into separate left and right channels
    fn deinterleave_stereo(samples: &[f32]) -> (Vec<f32>, Vec<f32>) {
        let mut left = Vec::with_capacity(samples.len() / 2);
        let mut right = Vec::with_capacity(samples.len() / 2);

        for chunk in samples.chunks_exact(2) {
            left.push(chunk[0]);
            right.push(chunk[1]);
        }

        (left, right)
    }

    /// Mix down multi-channel audio to stereo
    fn mixdown_to_stereo(samples: &[f32], channels: u16) -> (Vec<f32>, Vec<f32>) {
        if channels <= 2 {
            return Self::deinterleave_stereo(samples);
        }

        let frames = samples.len() / channels as usize;
        let mut left = Vec::with_capacity(frames);
        let mut right = Vec::with_capacity(frames);

        for frame in samples.chunks_exact(channels as usize) {
            // Simple mixdown: take first two channels as L/R
            left.push(frame[0]);
            right.push(if frame.len() > 1 { frame[1] } else { frame[0] });
        }

        (left, right)
    }

    /// Get mono level (average of left and right)
    pub fn mono_peak(&self) -> f32 {
        (self.peak_left + self.peak_right) / 2.0
    }

    /// Get mono RMS level (average of left and right)
    pub fn mono_rms(&self) -> f32 {
        (self.rms_left + self.rms_right) / 2.0
    }

    /// Get mono dB peak level
    pub fn mono_db_peak(&self) -> f32 {
        Self::linear_to_db(self.mono_peak())
    }

    /// Get mono dB RMS level
    pub fn mono_db_rms(&self) -> f32 {
        Self::linear_to_db(self.mono_rms())
    }
}

#[derive(Debug, Clone)]
pub enum ControlData {
    // 単一パラメータ制御
    Parameter {
        target_node_id: Uuid,
        parameter_name: String,
        value: ParameterValue,
    },

    // 複数制御（MIDIコントローラ等）
    MultiControl {
        commands: Vec<ControlCommand>,
    },

    // 3D変換制御
    Transform {
        position: Option<Vector3>,
        rotation: Option<Quaternion>,
        scale: Option<Vector3>,
    },

    // カメラ制御
    Camera {
        position: Option<Vector3>,
        target: Option<Vector3>,
        fov: Option<f32>,
        near: Option<f32>,
        far: Option<f32>,
    },

    // アニメーション制御
    Animation {
        keyframes: Vec<Keyframe>,
        time: f32,
        interpolation: InterpolationType,
    },
}

#[derive(Debug, Clone)]
pub struct ControlCommand {
    pub target_node_id: Uuid,
    pub parameter_name: String,
    pub value: ParameterValue,
    pub timestamp: std::time::Instant,
}

#[derive(Debug, Clone)]
pub enum ParameterValue {
    Float(f32),
    Integer(i32),
    Boolean(bool),
    String(String),
    Vector3(Vector3),
    Color([f32; 4]),
    Array(Vec<ParameterValue>),
}

#[derive(Debug, Clone)]
pub struct Keyframe {
    pub time: f32,
    pub value: ParameterValue,
    pub interpolation: InterpolationType,
}

#[derive(Debug, Clone)]
pub enum InterpolationType {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bezier(f32, f32, f32, f32),
}

// Controller Node制御マッピングシステム

#[derive(Debug, Clone)]
pub struct ControlMapping {
    pub source_parameter: String,      // ソースパラメータ名
    pub target_node_id: Uuid,          // ターゲットノードID
    pub target_parameter: String,      // ターゲットパラメータ名
    pub value_range: (f32, f32),       // 入力値範囲
    pub target_range: (f32, f32),      // 出力値範囲
    pub response_curve: ResponseCurve, // レスポンスカーブ
    pub enabled: bool,                 // マッピング有効/無効
}

#[derive(Debug, Clone)]
pub enum ResponseCurve {
    Linear,                  // 線形
    Exponential(f32),        // 指数カーブ
    Logarithmic(f32),        // 対数カーブ
    Sine,                    // サインカーブ
    Custom(Vec<(f32, f32)>), // カスタムカーブポイント
}

impl ControlMapping {
    pub fn new(source_parameter: String, target_node_id: Uuid, target_parameter: String) -> Self {
        Self {
            source_parameter,
            target_node_id,
            target_parameter,
            value_range: (0.0, 1.0),
            target_range: (0.0, 1.0),
            response_curve: ResponseCurve::Linear,
            enabled: true,
        }
    }

    /// 入力値をマッピングして出力値に変換
    pub fn apply(&self, input_value: f32) -> f32 {
        if !self.enabled {
            return input_value;
        }

        // 入力値を0-1範囲に正規化
        let normalized =
            (input_value - self.value_range.0) / (self.value_range.1 - self.value_range.0);
        let normalized = normalized.clamp(0.0, 1.0);

        // レスポンスカーブを適用
        let curved = self.apply_response_curve(normalized);

        // ターゲット範囲にスケール
        self.target_range.0 + curved * (self.target_range.1 - self.target_range.0)
    }

    fn apply_response_curve(&self, normalized_value: f32) -> f32 {
        match &self.response_curve {
            ResponseCurve::Linear => normalized_value,
            ResponseCurve::Exponential(exp) => normalized_value.powf(*exp),
            ResponseCurve::Logarithmic(base) => {
                if *base <= 0.0 || *base == 1.0 {
                    normalized_value
                } else {
                    (normalized_value * (*base - 1.0) + 1.0).log(*base) / base.log(*base)
                }
            }
            ResponseCurve::Sine => (normalized_value * std::f32::consts::PI / 2.0).sin(),
            ResponseCurve::Custom(points) => {
                if points.is_empty() {
                    return normalized_value;
                }

                // 線形補間でカスタムカーブを適用
                let mut prev_point = (0.0, 0.0);
                for &point in points {
                    if normalized_value <= point.0 {
                        let t = (normalized_value - prev_point.0) / (point.0 - prev_point.0);
                        return prev_point.1 + t * (point.1 - prev_point.1);
                    }
                    prev_point = point;
                }
                points.last().unwrap().1
            }
        }
    }
}

// Phase 4: 3D/VR/XR対応データ構造（既存維持）

#[derive(Debug, Clone)]
pub struct Scene3DData {
    pub meshes: Vec<Mesh3D>,
    pub materials: Vec<Material3D>,
    pub lights: Vec<Light3D>,
    pub camera: Camera3D,
    pub transform_matrix: [f32; 16], // 4x4 transformation matrix
}

#[derive(Debug, Clone)]
pub struct SpatialAudioData {
    pub audio_sources: Vec<SpatialAudioSource>,
    pub listener_position: Vector3,
    pub listener_orientation: Vector3,
    pub room_impulse_response: Option<Vec<f32>>,
}

#[derive(Debug, Clone)]
pub struct TransformData {
    pub position: Vector3,
    pub rotation: Quaternion,
    pub scale: Vector3,
    pub view_matrix: [f32; 16],
    pub projection_matrix: [f32; 16],
    pub mvp_matrix: [f32; 16], // Model-View-Projection matrix
}

// 3D補助データ構造

#[derive(Debug, Clone)]
pub struct Mesh3D {
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u32>,
    pub material_id: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Vertex3D {
    pub position: Vector3,
    pub normal: Vector3,
    pub uv: Vector2,
    pub color: [f32; 4], // RGBA
}

#[derive(Debug, Clone)]
pub struct Material3D {
    pub id: u32,
    pub albedo: [f32; 4], // RGBA
    pub metallic: f32,
    pub roughness: f32,
    pub emission: [f32; 3], // RGB
    pub texture_ids: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct Light3D {
    pub light_type: LightType,
    pub position: Vector3,
    pub direction: Vector3,
    pub color: [f32; 3], // RGB
    pub intensity: f32,
    pub range: f32,
    pub spot_angle: f32, // For spot lights
}

#[derive(Debug, Clone)]
pub struct Camera3D {
    pub position: Vector3,
    pub target: Vector3,
    pub up: Vector3,
    pub fov: f32,
    pub near_plane: f32,
    pub far_plane: f32,
    pub aspect_ratio: f32,
}

#[derive(Debug, Clone)]
pub struct SpatialAudioSource {
    pub position: Vector3,
    pub velocity: Vector3,
    pub audio_data: Vec<f32>,
    pub sample_rate: u32,
    pub attenuation: f32,
    pub doppler_factor: f32,
}

#[derive(Debug, Clone)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Debug, Clone)]
pub enum LightType {
    Directional,
    Point,
    Spot,
    Area,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VideoFormat {
    Rgba8,
    Rgb8,
    Bgra8,
    Bgr8,
    Yuv420p,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeType {
    Input(InputType),
    Output(OutputType),
    Effect(EffectType),
    Audio(AudioType),
    Tally(TallyType),
    Control(ControlType),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InputType {
    Camera,
    ScreenCapture,
    WindowCapture,
    VideoFile,
    TestPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputType {
    VirtualWebcam,
    Preview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EffectType {
    ColorCorrection,
    Blur,
    Sharpen,
    Transform,
    Composite,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AudioType {
    Input,
    Mixer,
    Effect,
    Output,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TallyType {
    Generator,
    Monitor,
    Logic,
    Router,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
#[allow(clippy::upper_case_acronyms)]
pub enum ControlType {
    Lfo,                 // Low Frequency Oscillator
    Timeline,            // タイムライン・キーフレーム
    MidiController,      // MIDIコントローラー
    MathController,      // 数式演算・式制御
    AudioReactive,       // 音声反応制御
    GamepadController,   // ゲームパッド・ジョイスティック
    TouchOSC,            // タッチOSC・モバイル制御
    Envelope,            // ADSR エンベロープ
    RandomController,    // ランダム値生成器
    LogicController,     // 論理演算・条件制御
    OSCReceiver,         // OSC受信・外部機器連携
    WebSocketController, // WebSocket制御・Web統合
    APIController,       // REST API制御・クラウド連携
    VideoAnalysis,       // 映像解析制御・モーション検出
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConnectionType {
    RenderData, // 映像・3Dデータ（メイン処理線）
    Audio,      // 音声データ（ステレオ・3D音響統合）
    Control,    // 制御信号線（パラメータ・変換制御）
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub parameters: HashMap<String, serde_json::Value>,
}

pub struct Node {
    pub id: Uuid,
    pub node_type: NodeType,
    pub config: NodeConfig,
    pub inputs: Vec<Connection>,
    pub outputs: Vec<Connection>,
}

impl Node {
    pub fn new(id: Uuid, node_type: NodeType, config: NodeConfig) -> Self {
        Self {
            id,
            node_type,
            config,
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub connection_type: ConnectionType,
    pub connected_node: Option<Uuid>,
}

pub struct NodeGraph {
    nodes: HashMap<Uuid, Node>,
    connections: Vec<(Uuid, Uuid, ConnectionType)>,
}

impl Default for NodeGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            connections: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id, node);
    }

    pub fn connect_nodes(
        &mut self,
        source_id: Uuid,
        target_id: Uuid,
        connection_type: ConnectionType,
    ) -> ConstellationResult<()> {
        if !self.nodes.contains_key(&source_id) {
            return Err(ConstellationError::NodeNotFound { node_id: source_id });
        }
        if !self.nodes.contains_key(&target_id) {
            return Err(ConstellationError::NodeNotFound { node_id: target_id });
        }

        // 循環参照チェック
        if self.would_create_cycle(source_id, target_id) {
            return Err(ConstellationError::ConnectionCycleDetected {
                path: self.find_cycle_path(source_id, target_id),
            });
        }

        self.connections
            .push((source_id, target_id, connection_type));
        Ok(())
    }

    pub fn get_node(&self, id: &Uuid) -> Option<&Node> {
        self.nodes.get(id)
    }

    pub fn get_node_mut(&mut self, id: &Uuid) -> Option<&mut Node> {
        self.nodes.get_mut(id)
    }

    /// 循環参照をチェックする
    fn would_create_cycle(&self, source_id: Uuid, target_id: Uuid) -> bool {
        self.has_path(target_id, source_id)
    }

    /// ノード間にパスが存在するかチェック
    fn has_path(&self, from: Uuid, to: Uuid) -> bool {
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![from];

        while let Some(current) = stack.pop() {
            if current == to {
                return true;
            }

            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);

            // 現在のノードから接続されているノードを探す
            for (source, target, _) in &self.connections {
                if *source == current {
                    stack.push(*target);
                }
            }
        }

        false
    }

    /// 循環パスを見つける
    fn find_cycle_path(&self, source_id: Uuid, target_id: Uuid) -> Vec<Uuid> {
        let mut path = Vec::new();
        let mut visited = std::collections::HashSet::new();
        self.find_path_recursive(target_id, source_id, &mut path, &mut visited);
        path.push(source_id);
        path.push(target_id);
        path
    }

    fn find_path_recursive(
        &self,
        current: Uuid,
        target: Uuid,
        path: &mut Vec<Uuid>,
        visited: &mut std::collections::HashSet<Uuid>,
    ) -> bool {
        if current == target {
            return true;
        }

        if visited.contains(&current) {
            return false;
        }
        visited.insert(current);
        path.push(current);

        for (source, next, _) in &self.connections {
            if *source == current && self.find_path_recursive(*next, target, path, visited) {
                return true;
            }
        }

        path.pop();
        false
    }
}

pub struct FrameProcessor {
    #[allow(dead_code)]
    node_id: Uuid,
    processor_type: ProcessorType,
}

impl FrameProcessor {
    pub fn new(node_id: Uuid, processor_type: ProcessorType) -> Self {
        Self {
            node_id,
            processor_type,
        }
    }

    pub fn process(&mut self, input: &FrameData) -> ConstellationResult<FrameData> {
        match &self.processor_type {
            ProcessorType::PassThrough => Ok(input.clone()),
            ProcessorType::ColorCorrection => self.process_color_correction(input),
            ProcessorType::Blur => self.process_blur(input),
            ProcessorType::Transform => self.process_transform(input),
        }
    }

    fn process_color_correction(&mut self, input: &FrameData) -> ConstellationResult<FrameData> {
        Ok(input.clone())
    }

    fn process_blur(&mut self, input: &FrameData) -> ConstellationResult<FrameData> {
        Ok(input.clone())
    }

    fn process_transform(&mut self, input: &FrameData) -> ConstellationResult<FrameData> {
        Ok(input.clone())
    }
}

#[derive(Debug, Clone)]
pub enum ProcessorType {
    PassThrough,
    ColorCorrection,
    Blur,
    Transform,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constellation_engine_creation() {
        let result = ConstellationEngine::new();
        // Note: This may fail in CI environments without Vulkan drivers
        if result.is_err() {
            println!(
                "Vulkan initialization failed (expected in CI): {:?}",
                result.err()
            );
        }
    }

    #[test]
    fn test_node_graph_operations() {
        let mut graph = NodeGraph::new();
        let node_id = Uuid::new_v4();
        let node = Node::new(
            node_id,
            NodeType::Input(InputType::Camera),
            NodeConfig {
                parameters: HashMap::new(),
            },
        );

        graph.add_node(node);
        assert!(graph.get_node(&node_id).is_some());
    }

    #[test]
    fn test_frame_processor() {
        let node_id = Uuid::new_v4();
        let mut processor = FrameProcessor::new(node_id, ProcessorType::PassThrough);

        let input_frame = FrameData {
            render_data: None,
            audio_data: None,
            control_data: None,
            tally_metadata: TallyMetadata::new(),
        };

        let result = processor.process(&input_frame);
        assert!(result.is_ok());
    }
}
