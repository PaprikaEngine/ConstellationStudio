use crate::{NodeProcessor, NodeProperties, ParameterDefinition, ParameterType};
use anyhow::Result;
use constellation_core::*;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

pub struct ParameterControllerNode {
    id: Uuid,
    config: NodeConfig,
    properties: NodeProperties,
    target_mappings: HashMap<String, ControlMapping>,
}

#[derive(Debug, Clone)]
pub struct ControlMapping {
    pub target_node_id: Uuid,
    pub parameter_name: String,
    pub value_transform: ValueTransform,
}

#[derive(Debug, Clone)]
pub enum ValueTransform {
    Direct,
    Scale {
        min: f32,
        max: f32,
    },
    Invert,
    Threshold {
        threshold: f32,
        below: ParameterValue,
        above: ParameterValue,
    },
}

impl ParameterControllerNode {
    pub fn new(id: Uuid, config: NodeConfig) -> Result<Self> {
        let mut parameters = HashMap::new();
        parameters.insert(
            "control_mode".to_string(),
            ParameterDefinition {
                name: "Control Mode".to_string(),
                parameter_type: ParameterType::Enum(vec![
                    "Manual".to_string(),
                    "MIDI".to_string(),
                    "OSC".to_string(),
                    "Animation".to_string(),
                ]),
                default_value: Value::String("Manual".to_string()),
                min_value: None,
                max_value: None,
                description: "Control input mode".to_string(),
            },
        );

        parameters.insert(
            "brightness".to_string(),
            ParameterDefinition {
                name: "Brightness".to_string(),
                parameter_type: ParameterType::Float,
                default_value: Value::from(1.0),
                min_value: Some(Value::from(0.0)),
                max_value: Some(Value::from(2.0)),
                description: "Brightness control value".to_string(),
            },
        );

        parameters.insert(
            "contrast".to_string(),
            ParameterDefinition {
                name: "Contrast".to_string(),
                parameter_type: ParameterType::Float,
                default_value: Value::from(1.0),
                min_value: Some(Value::from(0.0)),
                max_value: Some(Value::from(3.0)),
                description: "Contrast control value".to_string(),
            },
        );

        parameters.insert(
            "saturation".to_string(),
            ParameterDefinition {
                name: "Saturation".to_string(),
                parameter_type: ParameterType::Float,
                default_value: Value::from(1.0),
                min_value: Some(Value::from(0.0)),
                max_value: Some(Value::from(2.0)),
                description: "Saturation control value".to_string(),
            },
        );

        let properties = NodeProperties {
            id,
            name: "Parameter Controller".to_string(),
            node_type: NodeType::Control(ControlType::ParameterController),
            input_types: vec![ConnectionType::Control],
            output_types: vec![ConnectionType::Control],
            parameters,
        };

        Ok(Self {
            id,
            config,
            properties,
            target_mappings: HashMap::new(),
        })
    }

    pub fn add_mapping(&mut self, control_name: String, mapping: ControlMapping) {
        self.target_mappings.insert(control_name, mapping);
    }

    fn generate_control_commands(&self) -> Vec<ControlCommand> {
        let mut commands = Vec::new();

        for (control_name, mapping) in &self.target_mappings {
            if let Some(value) = self.get_parameter(control_name) {
                let transformed_value = self.transform_value(value, &mapping.value_transform);

                commands.push(ControlCommand {
                    target_node_id: mapping.target_node_id,
                    parameter_name: mapping.parameter_name.clone(),
                    value: transformed_value,
                    timestamp: std::time::Instant::now(),
                });
            }
        }

        commands
    }

    fn transform_value(&self, value: Value, transform: &ValueTransform) -> ParameterValue {
        match transform {
            ValueTransform::Direct => {
                if let Some(f) = value.as_f64() {
                    ParameterValue::Float(f as f32)
                } else if let Some(i) = value.as_i64() {
                    ParameterValue::Integer(i as i32)
                } else if let Some(b) = value.as_bool() {
                    ParameterValue::Boolean(b)
                } else if let Some(s) = value.as_str() {
                    ParameterValue::String(s.to_string())
                } else {
                    ParameterValue::Float(0.0)
                }
            }
            ValueTransform::Scale { min, max } => {
                let f = value.as_f64().unwrap_or(0.0) as f32;
                let scaled = min + (f * (max - min));
                ParameterValue::Float(scaled)
            }
            ValueTransform::Invert => {
                let f = value.as_f64().unwrap_or(0.0) as f32;
                ParameterValue::Float(1.0 - f)
            }
            ValueTransform::Threshold {
                threshold,
                below,
                above,
            } => {
                let f = value.as_f64().unwrap_or(0.0) as f32;
                if f < *threshold {
                    below.clone()
                } else {
                    above.clone()
                }
            }
        }
    }
}

impl NodeProcessor for ParameterControllerNode {
    fn process(&mut self, input: FrameData) -> Result<FrameData> {
        let commands = self.generate_control_commands();

        let control_data = if !commands.is_empty() {
            Some(ControlData::MultiControl { commands })
        } else {
            input.control_data
        };

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

pub struct AnimationControllerNode {
    id: Uuid,
    config: NodeConfig,
    properties: NodeProperties,
    keyframes: Vec<Keyframe>,
    current_time: f32,
    is_playing: bool,
    loop_animation: bool,
}

impl AnimationControllerNode {
    pub fn new(id: Uuid, config: NodeConfig) -> Result<Self> {
        let mut parameters = HashMap::new();
        parameters.insert(
            "play".to_string(),
            ParameterDefinition {
                name: "Play".to_string(),
                parameter_type: ParameterType::Boolean,
                default_value: Value::Bool(false),
                min_value: None,
                max_value: None,
                description: "Play animation".to_string(),
            },
        );

        parameters.insert(
            "time".to_string(),
            ParameterDefinition {
                name: "Time".to_string(),
                parameter_type: ParameterType::Float,
                default_value: Value::from(0.0),
                min_value: Some(Value::from(0.0)),
                max_value: Some(Value::from(10.0)),
                description: "Animation time in seconds".to_string(),
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
                description: "Loop animation".to_string(),
            },
        );

        let properties = NodeProperties {
            id,
            name: "Animation Controller".to_string(),
            node_type: NodeType::Control(ControlType::AnimationController),
            input_types: vec![],
            output_types: vec![ConnectionType::Control],
            parameters,
        };

        Ok(Self {
            id,
            config,
            properties,
            keyframes: Vec::new(),
            current_time: 0.0,
            is_playing: false,
            loop_animation: true,
        })
    }

    pub fn add_keyframe(&mut self, keyframe: Keyframe) {
        self.keyframes.push(keyframe);
        // Sort keyframes by time
        self.keyframes.sort_by(|a, b| a.time.total_cmp(&b.time));
    }

    fn update_time(&mut self, delta_time: f32) {
        if self.is_playing {
            self.current_time += delta_time;

            if let Some(last_keyframe) = self.keyframes.last() {
                if self.current_time > last_keyframe.time {
                    if self.loop_animation {
                        self.current_time = 0.0;
                    } else {
                        self.current_time = last_keyframe.time;
                        self.is_playing = false;
                    }
                }
            }
        }
    }

    fn interpolate_value_at_time(&self, time: f32) -> Option<ParameterValue> {
        if self.keyframes.is_empty() {
            return None;
        }

        // Find surrounding keyframes
        let mut before_keyframe = None;
        let mut after_keyframe = None;

        for keyframe in &self.keyframes {
            if keyframe.time <= time {
                before_keyframe = Some(keyframe);
            } else {
                after_keyframe = Some(keyframe);
                break;
            }
        }

        match (before_keyframe, after_keyframe) {
            (Some(before), Some(after)) => {
                // Interpolate between keyframes
                let t = (time - before.time) / (after.time - before.time);
                Some(self.interpolate_values(&before.value, &after.value, t, &before.interpolation))
            }
            (Some(keyframe), None) => {
                // Use last keyframe
                Some(keyframe.value.clone())
            }
            (None, Some(keyframe)) => {
                // Use first keyframe
                Some(keyframe.value.clone())
            }
            (None, None) => None,
        }
    }

    fn interpolate_values(
        &self,
        from: &ParameterValue,
        to: &ParameterValue,
        t: f32,
        interpolation: &InterpolationType,
    ) -> ParameterValue {
        let smooth_t = match interpolation {
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
                // Simplified cubic bezier interpolation
                let t2 = t * t;
                let t3 = t2 * t;
                let mt = 1.0 - t;
                let mt2 = mt * mt;
                let mt3 = mt2 * mt;

                mt3 * p1 + 3.0 * mt2 * t * p2 + 3.0 * mt * t2 * p3 + t3 * p4
            }
        };

        match (from, to) {
            (ParameterValue::Float(f1), ParameterValue::Float(f2)) => {
                ParameterValue::Float(f1 + (f2 - f1) * smooth_t)
            }
            (ParameterValue::Integer(i1), ParameterValue::Integer(i2)) => {
                let interpolated = *i1 as f32 + (*i2 as f32 - *i1 as f32) * smooth_t;
                ParameterValue::Integer(interpolated.round() as i32)
            }
            (ParameterValue::Vector3(v1), ParameterValue::Vector3(v2)) => {
                ParameterValue::Vector3(Vector3 {
                    x: v1.x + (v2.x - v1.x) * smooth_t,
                    y: v1.y + (v2.y - v1.y) * smooth_t,
                    z: v1.z + (v2.z - v1.z) * smooth_t,
                })
            }
            (ParameterValue::Color(c1), ParameterValue::Color(c2)) => ParameterValue::Color([
                c1[0] + (c2[0] - c1[0]) * smooth_t,
                c1[1] + (c2[1] - c1[1]) * smooth_t,
                c1[2] + (c2[2] - c1[2]) * smooth_t,
                c1[3] + (c2[3] - c1[3]) * smooth_t,
            ]),
            _ => from.clone(), // Fallback for non-interpolable types
        }
    }
}

impl NodeProcessor for AnimationControllerNode {
    fn process(&mut self, input: FrameData) -> Result<FrameData> {
        // Update animation state from parameters
        self.is_playing = self
            .get_parameter("play")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        self.loop_animation = self
            .get_parameter("loop")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Manual time override
        if let Some(manual_time) = self.get_parameter("time").and_then(|v| v.as_f64()) {
            self.current_time = manual_time as f32;
        } else {
            // Auto-advance time based on actual frame timing
            let now = std::time::Instant::now();
            static mut LAST_FRAME_TIME: Option<std::time::Instant> = None;
            let delta_time = unsafe {
                if let Some(last_time) = LAST_FRAME_TIME {
                    let delta = now.duration_since(last_time).as_secs_f32();
                    LAST_FRAME_TIME = Some(now);
                    delta.min(0.1) // Cap at 100ms to prevent large jumps
                } else {
                    LAST_FRAME_TIME = Some(now);
                    1.0 / 60.0 // Default for first frame
                }
            };
            self.update_time(delta_time);
        }

        let control_data = if let Some(value) = self.interpolate_value_at_time(self.current_time) {
            Some(ControlData::Animation {
                keyframes: self.keyframes.clone(),
                time: self.current_time,
                interpolation: InterpolationType::Linear,
            })
        } else {
            input.control_data
        };

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
