use crate::controller::{apply_mappings, ControllerConfig, ControllerNode};
use crate::{NodeProcessor, NodeProperties, ParameterDefinition, ParameterType};
use anyhow::Result;
use constellation_core::*;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;
use uuid::Uuid;

/// LFO波形タイプ
#[derive(Debug, Clone)]
pub enum Waveform {
    Sine,
    Square,
    Triangle,
    Sawtooth,
    Noise,
    Custom(Vec<f32>), // カスタム波形テーブル
}

/// LFO (Low Frequency Oscillator) コントローラ
pub struct LFOController {
    id: Uuid,
    config: NodeConfig,
    properties: NodeProperties,
    controller_config: ControllerConfig,

    // LFO設定
    frequency: f32,     // 周波数 (Hz)
    amplitude: f32,     // 振幅 (0.0-1.0)
    offset: f32,        // DCオフセット
    waveform: Waveform, // 波形タイプ
    phase: f32,         // 位相オフセット (0.0-1.0)

    // 時間管理
    start_time: Instant,
    last_update: Instant,

    // 現在の値
    current_value: f32,

    // ランダムノイズ用
    noise_seed: u64,
}

impl LFOController {
    pub fn new(id: Uuid, config: NodeConfig) -> Result<Self> {
        let mut parameters = HashMap::new();

        // 基本LFOパラメータ
        parameters.insert(
            "frequency".to_string(),
            ParameterDefinition {
                name: "Frequency".to_string(),
                parameter_type: ParameterType::Float,
                default_value: Value::from(1.0),
                min_value: Some(Value::from(0.001)),
                max_value: Some(Value::from(100.0)),
                description: "LFO frequency in Hz".to_string(),
            },
        );

        parameters.insert(
            "amplitude".to_string(),
            ParameterDefinition {
                name: "Amplitude".to_string(),
                parameter_type: ParameterType::Float,
                default_value: Value::from(1.0),
                min_value: Some(Value::from(0.0)),
                max_value: Some(Value::from(1.0)),
                description: "LFO amplitude".to_string(),
            },
        );

        parameters.insert(
            "offset".to_string(),
            ParameterDefinition {
                name: "Offset".to_string(),
                parameter_type: ParameterType::Float,
                default_value: Value::from(0.0),
                min_value: Some(Value::from(-1.0)),
                max_value: Some(Value::from(1.0)),
                description: "DC offset".to_string(),
            },
        );

        parameters.insert(
            "waveform".to_string(),
            ParameterDefinition {
                name: "Waveform".to_string(),
                parameter_type: ParameterType::Enum(vec![
                    "Sine".to_string(),
                    "Square".to_string(),
                    "Triangle".to_string(),
                    "Sawtooth".to_string(),
                    "Noise".to_string(),
                ]),
                default_value: Value::String("Sine".to_string()),
                min_value: None,
                max_value: None,
                description: "LFO waveform type".to_string(),
            },
        );

        parameters.insert(
            "phase".to_string(),
            ParameterDefinition {
                name: "Phase".to_string(),
                parameter_type: ParameterType::Float,
                default_value: Value::from(0.0),
                min_value: Some(Value::from(0.0)),
                max_value: Some(Value::from(1.0)),
                description: "Phase offset (0.0-1.0)".to_string(),
            },
        );

        parameters.insert(
            "enabled".to_string(),
            ParameterDefinition {
                name: "Enabled".to_string(),
                parameter_type: ParameterType::Boolean,
                default_value: Value::Bool(true),
                min_value: None,
                max_value: None,
                description: "Enable/disable LFO".to_string(),
            },
        );

        let properties = NodeProperties {
            id,
            name: "LFO Controller".to_string(),
            node_type: NodeType::Control(ControlType::LFO),
            input_types: vec![], // LFOは入力なし
            output_types: vec![ConnectionType::Control],
            parameters,
        };

        let now = Instant::now();

        Ok(Self {
            id,
            config,
            properties,
            controller_config: ControllerConfig::default(),
            frequency: 1.0,
            amplitude: 1.0,
            offset: 0.0,
            waveform: Waveform::Sine,
            phase: 0.0,
            start_time: now,
            last_update: now,
            current_value: 0.0,
            noise_seed: 12345,
        })
    }

    /// 現在の時間に基づいてLFO値を計算
    fn calculate_lfo_value(&mut self, elapsed_time: f32) -> f32 {
        // フェーズ調整された時間を計算
        let phase_adjusted_time = elapsed_time + (self.phase * (1.0 / self.frequency));

        // 基本波形値を計算
        let base_value = match &self.waveform {
            Waveform::Sine => {
                let angle = phase_adjusted_time * self.frequency * 2.0 * std::f32::consts::PI;
                angle.sin()
            }
            Waveform::Square => {
                let phase = (phase_adjusted_time * self.frequency) % 1.0;
                if phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Waveform::Triangle => {
                let phase = (phase_adjusted_time * self.frequency) % 1.0;
                if phase < 0.5 {
                    4.0 * phase - 1.0
                } else {
                    3.0 - 4.0 * phase
                }
            }
            Waveform::Sawtooth => {
                let phase = (phase_adjusted_time * self.frequency) % 1.0;
                2.0 * phase - 1.0
            }
            Waveform::Noise => {
                // Simple pseudo-random noise
                self.noise_seed = self.noise_seed.wrapping_mul(1103515245).wrapping_add(12345);
                let noise = (self.noise_seed as f32 / u64::MAX as f32) * 2.0 - 1.0;
                noise
            }
            Waveform::Custom(samples) => {
                if samples.is_empty() {
                    0.0
                } else {
                    let phase = (phase_adjusted_time * self.frequency) % 1.0;
                    let index = (phase * samples.len() as f32) as usize;
                    samples[index.min(samples.len() - 1)]
                }
            }
        };

        // 振幅とオフセットを適用
        let scaled_value = base_value * self.amplitude + self.offset;

        // -1.0から1.0の範囲にクランプ
        scaled_value.clamp(-1.0, 1.0)
    }

    /// パラメータを更新
    fn update_parameters(&mut self) {
        self.frequency = self
            .get_parameter("frequency")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32;

        self.amplitude = self
            .get_parameter("amplitude")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32;

        self.offset = self
            .get_parameter("offset")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;

        self.phase = self
            .get_parameter("phase")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;

        // 波形タイプを更新
        if let Some(waveform_value) = self.get_parameter("waveform") {
            if let Some(waveform_str) = waveform_value.as_str() {
                self.waveform = match waveform_str {
                    "Sine" => Waveform::Sine,
                    "Square" => Waveform::Square,
                    "Triangle" => Waveform::Triangle,
                    "Sawtooth" => Waveform::Sawtooth,
                    "Noise" => Waveform::Noise,
                    _ => Waveform::Sine,
                };
            }
        }

        // コントローラ有効状態を更新
        self.controller_config.enabled = self
            .get_parameter("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
    }
}

impl NodeProcessor for LFOController {
    fn process(&mut self, input: FrameData) -> Result<FrameData> {
        // パラメータを更新
        self.update_parameters();

        // 無効なら入力をそのまま通す
        if !self.controller_config.enabled {
            return Ok(input);
        }

        // 経過時間を計算
        let now = Instant::now();
        let elapsed = now.duration_since(self.start_time).as_secs_f32();

        // LFO値を計算
        self.current_value = self.calculate_lfo_value(elapsed);

        // 制御コマンドを生成
        let control_commands = self.generate_control_commands();

        let control_data = if !control_commands.is_empty() {
            Some(ControlData::MultiControl {
                commands: control_commands,
            })
        } else {
            input.control_data
        };

        self.last_update = now;

        Ok(FrameData {
            render_data: input.render_data,
            audio_data: input.audio_data,
            control_data,
            tally_metadata: input.tally_metadata,
        })
    }

    fn get_properties(&self) -> NodeProperties {
        self.properties.clone()
    }

    fn set_parameter(&mut self, key: &str, value: Value) -> Result<()> {
        self.config.parameters.insert(key.to_string(), value);
        Ok(())
    }

    fn get_parameter(&self, key: &str) -> Option<Value> {
        self.config.parameters.get(key).cloned()
    }
}

impl ControllerNode for LFOController {
    fn add_mapping(&mut self, mapping: ControlMapping) {
        self.controller_config.mappings.push(mapping);
    }

    fn remove_mapping(&mut self, source_parameter: &str) {
        self.controller_config
            .mappings
            .retain(|m| m.source_parameter != source_parameter);
    }

    fn get_control_value(&self, parameter: &str) -> Option<f32> {
        if parameter == "output" || parameter == "lfo" {
            Some(self.current_value)
        } else {
            None
        }
    }

    fn generate_control_commands(&self) -> Vec<ControlCommand> {
        let mut control_values = HashMap::new();
        control_values.insert("output".to_string(), self.current_value);
        control_values.insert("lfo".to_string(), self.current_value);

        apply_mappings(&self.controller_config.mappings, &control_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lfo_controller_creation() {
        let id = Uuid::new_v4();
        let config = NodeConfig {
            parameters: HashMap::new(),
        };

        let controller = LFOController::new(id, config);
        assert!(controller.is_ok());

        let controller = controller.unwrap();
        assert_eq!(controller.id, id);
        assert_eq!(controller.frequency, 1.0);
        assert_eq!(controller.amplitude, 1.0);
    }

    #[test]
    fn test_lfo_sine_waveform() {
        let id = Uuid::new_v4();
        let config = NodeConfig {
            parameters: HashMap::new(),
        };

        let mut controller = LFOController::new(id, config).unwrap();

        // Test sine wave at different time points
        let value_0 = controller.calculate_lfo_value(0.0);
        let value_quarter = controller.calculate_lfo_value(0.25);
        let value_half = controller.calculate_lfo_value(0.5);

        assert!((value_0 - 0.0).abs() < 0.01); // sin(0) = 0
        assert!((value_quarter - 1.0).abs() < 0.01); // sin(π/2) = 1
        assert!((value_half - 0.0).abs() < 0.01); // sin(π) = 0
    }

    #[test]
    fn test_lfo_square_waveform() {
        let id = Uuid::new_v4();
        let config = NodeConfig {
            parameters: HashMap::new(),
        };

        let mut controller = LFOController::new(id, config).unwrap();
        controller.waveform = Waveform::Square;

        let value_0 = controller.calculate_lfo_value(0.0);
        let value_quarter = controller.calculate_lfo_value(0.25);
        let value_half = controller.calculate_lfo_value(0.5);

        assert_eq!(value_0, 1.0); // First half of square wave
        assert_eq!(value_quarter, 1.0); // Still first half
        assert_eq!(value_half, -1.0); // Second half of square wave
    }

    #[test]
    fn test_lfo_amplitude_scaling() {
        let id = Uuid::new_v4();
        let config = NodeConfig {
            parameters: HashMap::new(),
        };

        let mut controller = LFOController::new(id, config).unwrap();
        controller.amplitude = 0.5;

        let value = controller.calculate_lfo_value(0.25); // Should be at peak
        assert!((value - 0.5).abs() < 0.01); // Peak scaled by amplitude
    }

    #[test]
    fn test_lfo_offset() {
        let id = Uuid::new_v4();
        let config = NodeConfig {
            parameters: HashMap::new(),
        };

        let mut controller = LFOController::new(id, config).unwrap();
        controller.offset = 0.5;

        let value = controller.calculate_lfo_value(0.0); // Should be at zero crossing
        assert!((value - 0.5).abs() < 0.01); // Zero crossing + offset
    }
}
