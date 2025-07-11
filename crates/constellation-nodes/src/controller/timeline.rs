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

use crate::controller::{apply_mappings, ControllerConfig, ControllerNode};
use crate::{NodeProcessor, NodeProperties, ParameterDefinition, ParameterType};
use anyhow::Result;
use constellation_core::*;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;
use uuid::Uuid;

/// タイムラインコントローラ - キーフレーム制御
pub struct TimelineController {
    id: Uuid,
    config: NodeConfig,
    properties: NodeProperties,
    controller_config: ControllerConfig,

    // タイムライン設定
    keyframes: Vec<Keyframe>,
    current_time: f32,
    duration: f32,
    is_playing: bool,
    loop_enabled: bool,
    playback_speed: f32,

    // 現在の値
    current_value: f32,

    // 時間管理
    start_time: Instant,
    last_update: Instant,
}

impl TimelineController {
    pub fn new(id: Uuid, config: NodeConfig) -> Result<Self> {
        let mut parameters = HashMap::new();

        // タイムライン制御パラメータ
        parameters.insert(
            "play".to_string(),
            ParameterDefinition {
                name: "Play".to_string(),
                parameter_type: ParameterType::Boolean,
                default_value: Value::Bool(false),
                min_value: None,
                max_value: None,
                description: "Play/pause timeline".to_string(),
            },
        );

        parameters.insert(
            "time".to_string(),
            ParameterDefinition {
                name: "Time".to_string(),
                parameter_type: ParameterType::Float,
                default_value: Value::from(0.0),
                min_value: Some(Value::from(0.0)),
                max_value: Some(Value::from(60.0)),
                description: "Current timeline time in seconds".to_string(),
            },
        );

        parameters.insert(
            "duration".to_string(),
            ParameterDefinition {
                name: "Duration".to_string(),
                parameter_type: ParameterType::Float,
                default_value: Value::from(10.0),
                min_value: Some(Value::from(0.1)),
                max_value: Some(Value::from(300.0)),
                description: "Timeline duration in seconds".to_string(),
            },
        );

        parameters.insert(
            "loop".to_string(),
            ParameterDefinition {
                name: "Loop".to_string(),
                parameter_type: ParameterType::Boolean,
                default_value: Value::Bool(true),
                min_value: None,
                max_value: None,
                description: "Loop timeline playback".to_string(),
            },
        );

        parameters.insert(
            "speed".to_string(),
            ParameterDefinition {
                name: "Speed".to_string(),
                parameter_type: ParameterType::Float,
                default_value: Value::from(1.0),
                min_value: Some(Value::from(0.1)),
                max_value: Some(Value::from(10.0)),
                description: "Playback speed multiplier".to_string(),
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
                description: "Enable/disable timeline controller".to_string(),
            },
        );

        let properties = NodeProperties {
            id,
            name: "Timeline Controller".to_string(),
            node_type: NodeType::Control(ControlType::Timeline),
            input_types: vec![],
            output_types: vec![ConnectionType::Control],
            parameters,
        };

        let now = Instant::now();

        Ok(Self {
            id,
            config,
            properties,
            controller_config: ControllerConfig::default(),
            keyframes: Vec::new(),
            current_time: 0.0,
            duration: 10.0,
            is_playing: false,
            loop_enabled: true,
            playback_speed: 1.0,
            current_value: 0.0,
            start_time: now,
            last_update: now,
        })
    }

    /// キーフレームを追加
    pub fn add_keyframe(&mut self, keyframe: Keyframe) {
        self.keyframes.push(keyframe);
        // 時間でソート
        self.keyframes
            .sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    }

    /// キーフレームをクリア
    pub fn clear_keyframes(&mut self) {
        self.keyframes.clear();
    }

    /// 指定時間での値を補間
    fn interpolate_value_at_time(&self, time: f32) -> f32 {
        if self.keyframes.is_empty() {
            return 0.0;
        }

        // 時間をクランプ
        let clamped_time = time.max(0.0).min(self.duration);

        // 周囲のキーフレームを検索
        let mut before_keyframe = None;
        let mut after_keyframe = None;

        for keyframe in &self.keyframes {
            if keyframe.time <= clamped_time {
                before_keyframe = Some(keyframe);
            } else {
                after_keyframe = Some(keyframe);
                break;
            }
        }

        match (before_keyframe, after_keyframe) {
            (Some(before), Some(after)) => {
                // 2つのキーフレーム間で補間
                let t = (clamped_time - before.time) / (after.time - before.time);
                let smooth_t = self.apply_interpolation(t, &before.interpolation);

                match (&before.value, &after.value) {
                    (ParameterValue::Float(f1), ParameterValue::Float(f2)) => {
                        f1 + (f2 - f1) * smooth_t
                    }
                    _ => 0.0, // 現在はFloatのみサポート
                }
            }
            (Some(keyframe), None) => {
                // 最後のキーフレーム
                match &keyframe.value {
                    ParameterValue::Float(f) => *f,
                    _ => 0.0,
                }
            }
            (None, Some(keyframe)) => {
                // 最初のキーフレーム
                match &keyframe.value {
                    ParameterValue::Float(f) => *f,
                    _ => 0.0,
                }
            }
            (None, None) => 0.0,
        }
    }

    /// 補間カーブを適用
    fn apply_interpolation(&self, t: f32, interpolation: &InterpolationType) -> f32 {
        match interpolation {
            InterpolationType::Linear => t,
            InterpolationType::EaseIn => t * t,
            InterpolationType::EaseOut => 1.0 - (1.0 - t).powi(2),
            InterpolationType::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - 2.0 * (1.0 - t).powi(2)
                }
            }
            InterpolationType::Bezier(p1, p2, p3, p4) => {
                // 簡略化されたベジェ補間
                let t2 = t * t;
                let t3 = t2 * t;
                let mt = 1.0 - t;
                let mt2 = mt * mt;
                let mt3 = mt2 * mt;

                mt3 * p1 + 3.0 * mt2 * t * p2 + 3.0 * mt * t2 * p3 + t3 * p4
            }
        }
    }

    /// 時間を更新
    fn update_time(&mut self, delta_time: f32) {
        if self.is_playing {
            self.current_time += delta_time * self.playback_speed;

            if self.current_time >= self.duration {
                if self.loop_enabled {
                    self.current_time %= self.duration;
                } else {
                    self.current_time = self.duration;
                    self.is_playing = false;
                }
            }
        }
    }

    /// パラメータを更新
    fn update_parameters(&mut self) {
        self.is_playing = self
            .get_parameter("play")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        self.loop_enabled = self
            .get_parameter("loop")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        self.playback_speed = self
            .get_parameter("speed")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32;

        self.duration = self
            .get_parameter("duration")
            .and_then(|v| v.as_f64())
            .unwrap_or(10.0) as f32;

        // 手動時間オーバーライド
        if let Some(manual_time) = self.get_parameter("time").and_then(|v| v.as_f64()) {
            self.current_time = manual_time as f32;
        }

        self.controller_config.enabled = self
            .get_parameter("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
    }
}

impl NodeProcessor for TimelineController {
    fn process(&mut self, input: FrameData) -> Result<FrameData> {
        // パラメータを更新
        self.update_parameters();

        // 無効なら入力をそのまま通す
        if !self.controller_config.enabled {
            return Ok(input);
        }

        // 時間を更新
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_update).as_secs_f32();
        self.update_time(delta_time);

        // 現在の値を補間
        self.current_value = self.interpolate_value_at_time(self.current_time);

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

impl ControllerNode for TimelineController {
    fn add_mapping(&mut self, mapping: ControlMapping) {
        self.controller_config.mappings.push(mapping);
    }

    fn remove_mapping(&mut self, source_parameter: &str) {
        self.controller_config
            .mappings
            .retain(|m| m.source_parameter != source_parameter);
    }

    fn get_control_value(&self, parameter: &str) -> Option<f32> {
        match parameter {
            "output" | "value" => Some(self.current_value),
            "time" => Some(self.current_time),
            "progress" => Some(self.current_time / self.duration),
            _ => None,
        }
    }

    fn generate_control_commands(&self) -> Vec<ControlCommand> {
        let mut control_values = HashMap::new();
        control_values.insert("output".to_string(), self.current_value);
        control_values.insert("value".to_string(), self.current_value);
        control_values.insert("time".to_string(), self.current_time);
        control_values.insert("progress".to_string(), self.current_time / self.duration);

        apply_mappings(&self.controller_config.mappings, &control_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_controller_creation() {
        let id = Uuid::new_v4();
        let config = NodeConfig {
            parameters: HashMap::new(),
        };

        let controller = TimelineController::new(id, config);
        assert!(controller.is_ok());

        let controller = controller.unwrap();
        assert_eq!(controller.id, id);
        assert_eq!(controller.duration, 10.0);
        assert!(!controller.is_playing);
    }

    #[test]
    fn test_timeline_keyframe_interpolation() {
        let id = Uuid::new_v4();
        let config = NodeConfig {
            parameters: HashMap::new(),
        };

        let mut controller = TimelineController::new(id, config).unwrap();

        // キーフレームを追加
        controller.add_keyframe(Keyframe {
            time: 0.0,
            value: ParameterValue::Float(0.0),
            interpolation: InterpolationType::Linear,
        });

        controller.add_keyframe(Keyframe {
            time: 5.0,
            value: ParameterValue::Float(1.0),
            interpolation: InterpolationType::Linear,
        });

        // 補間をテスト
        assert_eq!(controller.interpolate_value_at_time(0.0), 0.0);
        assert_eq!(controller.interpolate_value_at_time(2.5), 0.5);
        assert_eq!(controller.interpolate_value_at_time(5.0), 1.0);
    }

    #[test]
    fn test_timeline_playback() {
        let id = Uuid::new_v4();
        let config = NodeConfig {
            parameters: HashMap::new(),
        };

        let mut controller = TimelineController::new(id, config).unwrap();
        controller.is_playing = true;
        controller.duration = 10.0;

        // 時間を進める
        controller.update_time(1.0);
        assert_eq!(controller.current_time, 1.0);

        controller.update_time(2.0);
        assert_eq!(controller.current_time, 3.0);
    }

    #[test]
    fn test_timeline_loop() {
        let id = Uuid::new_v4();
        let config = NodeConfig {
            parameters: HashMap::new(),
        };

        let mut controller = TimelineController::new(id, config).unwrap();
        controller.is_playing = true;
        controller.duration = 5.0;
        controller.loop_enabled = true;
        controller.current_time = 4.0;

        // ループテスト
        controller.update_time(2.0);
        assert_eq!(controller.current_time, 1.0); // 6.0 % 5.0 = 1.0
        assert!(controller.is_playing);
    }
}
