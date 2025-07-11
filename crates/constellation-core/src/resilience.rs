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

use crate::error::{ConstellationError, ConstellationResult};
use crate::{ConstellationEngine, FrameData, NodeType, ProcessorType};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// システム健全性監視および自動復旧システム
pub struct ResilienceManager {
    #[allow(dead_code)]
    engine: Arc<ConstellationEngine>,
    health_monitor: HealthMonitor,
    recovery_strategies: HashMap<ErrorCategory, RecoveryStrategy>,
    fallback_modes: FallbackModeManager,
    performance_monitor: PerformanceMonitor,
}

/// システム健全性監視
#[derive(Debug)]
pub struct HealthMonitor {
    pub frame_processing_failures: AtomicU64,
    pub memory_allocation_failures: AtomicU64,
    pub gpu_processing_failures: AtomicU64,
    pub connection_failures: AtomicU64,
    pub last_successful_frame: Arc<std::sync::Mutex<Option<Instant>>>,
    pub system_status: Arc<std::sync::Mutex<SystemStatus>>,
}

#[derive(Debug, Clone)]
pub enum SystemStatus {
    Healthy,
    Degraded(Vec<String>),
    Critical(String),
    FailSafe,
}

/// エラーカテゴリ（詳細分類）
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ErrorCategory {
    FrameProcessing,
    MemoryAllocation,
    GpuProcessing,
    NetworkConnection,
    NodeProcessing,
    ResourceExhaustion,
    HardwareFailure,
}

/// 復旧戦略
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// 自動再試行
    Retry {
        max_attempts: u32,
        delay: Duration,
        backoff_multiplier: f32,
    },
    /// 品質低下モード
    QualityDegradation {
        reduced_resolution: Option<(u32, u32)>,
        reduced_framerate: Option<u32>,
        disable_effects: Vec<ProcessorType>,
    },
    /// フォールバック処理
    Fallback {
        fallback_processor: ProcessorType,
        fallback_nodes: Vec<NodeType>,
    },
    /// 段階的機能停止
    GracefulShutdown {
        preserve_data: bool,
        notify_users: bool,
        cleanup_timeout: Duration,
    },
}

/// フォールバックモード管理
#[derive(Debug)]
pub struct FallbackModeManager {
    current_mode: FallbackMode,
    original_config: Option<SystemConfiguration>,
    degradation_level: u8, // 0-10, 0が最高品質、10が最低品質
}

#[derive(Debug, Clone)]
pub enum FallbackMode {
    Normal,
    ReducedQuality,
    SafeMode,
    EmergencyMode,
}

#[derive(Debug, Clone)]
pub struct SystemConfiguration {
    pub resolution: (u32, u32),
    pub framerate: u32,
    pub enabled_effects: Vec<ProcessorType>,
    pub gpu_acceleration: bool,
    pub memory_limit: u64,
}

/// パフォーマンス監視
#[derive(Debug)]
pub struct PerformanceMonitor {
    pub frame_processing_times: Vec<Duration>,
    pub memory_usage_history: Vec<u64>,
    pub gpu_utilization_history: Vec<f32>,
    pub last_performance_check: Instant,
}

impl ResilienceManager {
    pub fn new(engine: Arc<ConstellationEngine>) -> Self {
        let mut recovery_strategies = HashMap::new();

        // デフォルト復旧戦略を設定
        recovery_strategies.insert(
            ErrorCategory::FrameProcessing,
            RecoveryStrategy::Retry {
                max_attempts: 3,
                delay: Duration::from_millis(100),
                backoff_multiplier: 2.0,
            },
        );

        recovery_strategies.insert(
            ErrorCategory::MemoryAllocation,
            RecoveryStrategy::QualityDegradation {
                reduced_resolution: Some((1280, 720)),
                reduced_framerate: Some(30),
                disable_effects: vec![ProcessorType::Blur],
            },
        );

        recovery_strategies.insert(
            ErrorCategory::GpuProcessing,
            RecoveryStrategy::Fallback {
                fallback_processor: ProcessorType::PassThrough,
                fallback_nodes: vec![],
            },
        );

        recovery_strategies.insert(
            ErrorCategory::HardwareFailure,
            RecoveryStrategy::GracefulShutdown {
                preserve_data: true,
                notify_users: true,
                cleanup_timeout: Duration::from_secs(30),
            },
        );

        Self {
            engine,
            health_monitor: HealthMonitor::new(),
            recovery_strategies,
            fallback_modes: FallbackModeManager::new(),
            performance_monitor: PerformanceMonitor::new(),
        }
    }

    /// エラー処理とリカバリー実行
    pub fn handle_error(
        &mut self,
        error: &ConstellationError,
    ) -> ConstellationResult<RecoveryAction> {
        // エラーの分類
        let error_category = self.classify_error(error);

        // エラー統計を更新
        self.update_error_statistics(&error_category);

        // システム状態を評価
        let system_status = self.evaluate_system_health();
        self.health_monitor.update_status(system_status.clone());

        // 復旧戦略を実行
        if let Some(strategy) = self.recovery_strategies.get(&error_category) {
            let strategy = strategy.clone(); // Clone to avoid borrowing issues
            let action = self.execute_recovery_strategy(&strategy, error)?;
            Ok(action)
        } else {
            // デフォルト戦略: エラーをログに記録し、継続
            tracing::warn!(
                "No recovery strategy for error category: {:?}",
                error_category
            );
            Ok(RecoveryAction::LogAndContinue)
        }
    }

    /// エラーの分類
    fn classify_error(&self, error: &ConstellationError) -> ErrorCategory {
        match error {
            ConstellationError::FrameProcessingFailed { .. }
            | ConstellationError::FrameDataCorrupted { .. }
            | ConstellationError::FrameProcessingTimeout { .. } => ErrorCategory::FrameProcessing,

            ConstellationError::InsufficientMemory { .. }
            | ConstellationError::ResourceAllocationFailed { .. } => {
                ErrorCategory::MemoryAllocation
            }

            ConstellationError::GpuProcessingFailed { .. } => ErrorCategory::GpuProcessing,

            ConstellationError::NetworkConnectionFailed { .. }
            | ConstellationError::DataTransmissionFailed { .. } => ErrorCategory::NetworkConnection,

            ConstellationError::NodeProcessingFailed { .. }
            | ConstellationError::NodeNotFound { .. } => ErrorCategory::NodeProcessing,

            ConstellationError::ResourceLimitExceeded { .. } => ErrorCategory::ResourceExhaustion,

            ConstellationError::HardwareNotSupported { .. }
            | ConstellationError::DriverIncompatible { .. }
            | ConstellationError::DeviceAccessFailed { .. } => ErrorCategory::HardwareFailure,

            _ => ErrorCategory::NodeProcessing, // デフォルト
        }
    }

    /// エラー統計を更新
    fn update_error_statistics(&self, category: &ErrorCategory) {
        match category {
            ErrorCategory::FrameProcessing => {
                self.health_monitor
                    .frame_processing_failures
                    .fetch_add(1, Ordering::Relaxed);
            }
            ErrorCategory::MemoryAllocation => {
                self.health_monitor
                    .memory_allocation_failures
                    .fetch_add(1, Ordering::Relaxed);
            }
            ErrorCategory::GpuProcessing => {
                self.health_monitor
                    .gpu_processing_failures
                    .fetch_add(1, Ordering::Relaxed);
            }
            ErrorCategory::NetworkConnection => {
                self.health_monitor
                    .connection_failures
                    .fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }
    }

    /// システム健全性評価
    fn evaluate_system_health(&self) -> SystemStatus {
        let frame_failures = self
            .health_monitor
            .frame_processing_failures
            .load(Ordering::Relaxed);
        let memory_failures = self
            .health_monitor
            .memory_allocation_failures
            .load(Ordering::Relaxed);
        let gpu_failures = self
            .health_monitor
            .gpu_processing_failures
            .load(Ordering::Relaxed);

        let total_failures = frame_failures + memory_failures + gpu_failures;

        if total_failures == 0 {
            SystemStatus::Healthy
        } else if total_failures < 5 {
            SystemStatus::Degraded(vec!["Minor errors detected".to_string()])
        } else if total_failures < 20 {
            SystemStatus::Critical("Multiple system errors".to_string())
        } else {
            SystemStatus::FailSafe
        }
    }

    /// 復旧戦略実行
    fn execute_recovery_strategy(
        &mut self,
        strategy: &RecoveryStrategy,
        error: &ConstellationError,
    ) -> ConstellationResult<RecoveryAction> {
        match strategy {
            RecoveryStrategy::Retry {
                max_attempts,
                delay,
                backoff_multiplier,
            } => Ok(RecoveryAction::Retry {
                max_attempts: *max_attempts,
                delay: *delay,
                backoff_multiplier: *backoff_multiplier,
            }),
            RecoveryStrategy::QualityDegradation {
                reduced_resolution,
                reduced_framerate,
                disable_effects,
            } => {
                self.fallback_modes.activate_degraded_mode(
                    *reduced_resolution,
                    *reduced_framerate,
                    disable_effects.clone(),
                )?;
                Ok(RecoveryAction::QualityReduced)
            }
            RecoveryStrategy::Fallback {
                fallback_processor,
                fallback_nodes,
            } => Ok(RecoveryAction::Fallback {
                processor: fallback_processor.clone(),
                nodes: fallback_nodes.clone(),
            }),
            RecoveryStrategy::GracefulShutdown {
                preserve_data,
                notify_users,
                cleanup_timeout,
            } => {
                if *notify_users {
                    tracing::error!("System entering graceful shutdown due to: {}", error);
                }
                Ok(RecoveryAction::GracefulShutdown {
                    preserve_data: *preserve_data,
                    cleanup_timeout: *cleanup_timeout,
                })
            }
        }
    }

    /// パフォーマンス監視
    pub fn monitor_performance(&mut self, _frame_data: &FrameData, processing_time: Duration) {
        self.performance_monitor.record_frame_time(processing_time);

        // パフォーマンス低下検出
        if self.performance_monitor.is_performance_degraded() {
            let _recovery_action = self.handle_performance_degradation();
        }
    }

    fn handle_performance_degradation(&mut self) -> ConstellationResult<()> {
        // パフォーマンス低下時の自動対応
        self.fallback_modes.increase_degradation_level()?;
        tracing::warn!("Performance degradation detected, reducing quality");
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum RecoveryAction {
    Retry {
        max_attempts: u32,
        delay: Duration,
        backoff_multiplier: f32,
    },
    QualityReduced,
    Fallback {
        processor: ProcessorType,
        nodes: Vec<NodeType>,
    },
    GracefulShutdown {
        preserve_data: bool,
        cleanup_timeout: Duration,
    },
    LogAndContinue,
}

impl HealthMonitor {
    fn new() -> Self {
        Self {
            frame_processing_failures: AtomicU64::new(0),
            memory_allocation_failures: AtomicU64::new(0),
            gpu_processing_failures: AtomicU64::new(0),
            connection_failures: AtomicU64::new(0),
            last_successful_frame: Arc::new(std::sync::Mutex::new(None)),
            system_status: Arc::new(std::sync::Mutex::new(SystemStatus::Healthy)),
        }
    }

    fn update_status(&self, status: SystemStatus) {
        if let Ok(mut current_status) = self.system_status.lock() {
            *current_status = status;
        }
    }
}

impl FallbackModeManager {
    fn new() -> Self {
        Self {
            current_mode: FallbackMode::Normal,
            original_config: None,
            degradation_level: 0,
        }
    }

    fn activate_degraded_mode(
        &mut self,
        _reduced_resolution: Option<(u32, u32)>,
        _reduced_framerate: Option<u32>,
        _disable_effects: Vec<ProcessorType>,
    ) -> ConstellationResult<()> {
        if self.original_config.is_none() {
            // 現在の設定を保存
            self.original_config = Some(SystemConfiguration {
                resolution: (1920, 1080), // デフォルト値
                framerate: 60,
                enabled_effects: vec![],
                gpu_acceleration: true,
                memory_limit: 1024 * 1024 * 1024, // 1GB
            });
        }

        self.current_mode = FallbackMode::ReducedQuality;
        self.degradation_level = (self.degradation_level + 1).min(10);

        tracing::info!("Activated degraded mode: level {}", self.degradation_level);
        Ok(())
    }

    fn increase_degradation_level(&mut self) -> ConstellationResult<()> {
        self.degradation_level = (self.degradation_level + 1).min(10);

        match self.degradation_level {
            1..=3 => self.current_mode = FallbackMode::ReducedQuality,
            4..=7 => self.current_mode = FallbackMode::SafeMode,
            8..=10 => self.current_mode = FallbackMode::EmergencyMode,
            _ => {}
        }

        tracing::info!("Increased degradation level to: {}", self.degradation_level);
        Ok(())
    }
}

impl PerformanceMonitor {
    fn new() -> Self {
        Self {
            frame_processing_times: Vec::new(),
            memory_usage_history: Vec::new(),
            gpu_utilization_history: Vec::new(),
            last_performance_check: Instant::now(),
        }
    }

    fn record_frame_time(&mut self, processing_time: Duration) {
        self.frame_processing_times.push(processing_time);

        // 過去100フレームのみ保持
        if self.frame_processing_times.len() > 100 {
            self.frame_processing_times.remove(0);
        }
    }

    fn is_performance_degraded(&self) -> bool {
        if self.frame_processing_times.len() < 10 {
            return false;
        }

        let recent_times: Vec<_> = self.frame_processing_times.iter().rev().take(10).collect();

        let total_time: Duration = recent_times.iter().copied().sum();
        let avg_time = total_time / recent_times.len() as u32;

        // 33ms (30fps) を超えている場合はパフォーマンス低下
        avg_time > Duration::from_millis(33)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_classification() {
        // テスト用にConstellationEngineのモックを作成する必要があります
        // 実際の実装では、mock frameworkまたはtest doubleを使用します
    }

    #[test]
    fn test_health_monitor() {
        let monitor = HealthMonitor::new();
        assert_eq!(monitor.frame_processing_failures.load(Ordering::Relaxed), 0);

        monitor
            .frame_processing_failures
            .fetch_add(1, Ordering::Relaxed);
        assert_eq!(monitor.frame_processing_failures.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new();

        // 正常な処理時間
        monitor.record_frame_time(Duration::from_millis(16));
        assert!(!monitor.is_performance_degraded());

        // 劣化した処理時間を複数回記録
        for _ in 0..15 {
            monitor.record_frame_time(Duration::from_millis(50));
        }
        assert!(monitor.is_performance_degraded());
    }
}
