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

//! Constellation Studio - Node Implementation Crate
//!
//! This crate contains all node implementations for the constellation pipeline.
//! During development, many fields and functions may not be used yet but are
//! planned for future implementation phases.

#![allow(dead_code)]
#![allow(unused_variables)]

use anyhow::Result;
use constellation_core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub mod camera;
pub mod capture;
pub mod controller;
pub mod effects;
pub mod input;
pub mod output;
pub mod video_file;
pub mod virtual_camera;

pub use capture::{ScreenCaptureNode, WindowCaptureNode};
pub use controller::*;
pub use effects::*;
pub use input::*;
pub use output::*;

// Export types needed for tests
pub use constellation_core::NodeConfig;

pub trait NodeProcessor: Send {
    fn process(&mut self, input: FrameData) -> Result<FrameData>;
    fn get_properties(&self) -> NodeProperties;
    fn set_parameter(&mut self, key: &str, value: serde_json::Value) -> Result<()>;
    fn get_parameter(&self, key: &str) -> Option<serde_json::Value>;

    // Tally自動伝播システム
    fn process_tally_metadata(&mut self, metadata: &TallyMetadata) -> TallyMetadata {
        // デフォルト実装: 変更なしで伝播
        let mut result = metadata.clone();
        result.add_to_path(self.get_properties().id);
        result
    }

    fn should_propagate_tally(&self, _metadata: &TallyMetadata) -> bool {
        // デフォルト実装: 常に伝播
        true
    }

    fn generate_tally_state(&self) -> TallyMetadata {
        // デフォルト実装: Tally状態なし
        TallyMetadata::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeProperties {
    pub id: Uuid,
    pub name: String,
    pub node_type: NodeType,
    pub input_types: Vec<ConnectionType>,
    pub output_types: Vec<ConnectionType>,
    pub parameters: HashMap<String, ParameterDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDefinition {
    pub name: String,
    pub parameter_type: ParameterType,
    pub default_value: serde_json::Value,
    pub min_value: Option<serde_json::Value>,
    pub max_value: Option<serde_json::Value>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    Float,
    Integer,
    Boolean,
    String,
    Color,
    Vector2,
    Vector3,
    Vector4,
    Enum(Vec<String>),
}

pub fn create_node_processor(
    node_type: NodeType,
    id: Uuid,
    config: NodeConfig,
) -> Result<Box<dyn NodeProcessor>> {
    match node_type {
        NodeType::Input(input_type) => match input_type {
            InputType::Camera => Ok(Box::new(CameraInputNode::new(id, config)?)),
            InputType::ScreenCapture => Ok(Box::new(ScreenCaptureNode::new(id, config)?)),
            InputType::WindowCapture => Ok(Box::new(WindowCaptureNode::new(id, config)?)),
            InputType::VideoFile => Ok(Box::new(VideoFileInputNode::new(id, config)?)),
            InputType::TestPattern => Ok(Box::new(TestPatternNode::new(id, config)?)),
        },
        NodeType::Output(output_type) => match output_type {
            OutputType::VirtualWebcam => Ok(Box::new(VirtualWebcamNode::new(id, config)?)),
            OutputType::Preview => Ok(Box::new(PreviewNode::new(id, config)?)),
        },
        NodeType::Effect(effect_type) => match effect_type {
            EffectType::ColorCorrection => Ok(Box::new(ColorCorrectionNode::new(id, config)?)),
            EffectType::Blur => Ok(Box::new(BlurNode::new(id, config)?)),
            EffectType::Sharpen => Ok(Box::new(SharpenNode::new(id, config)?)),
            EffectType::Transform => Ok(Box::new(TransformNode::new(id, config)?)),
            EffectType::Composite => Ok(Box::new(CompositeNode::new(id, config)?)),
        },
        NodeType::Audio(audio_type) => match audio_type {
            AudioType::Input => Ok(Box::new(AudioInputNode::new(id, config)?)),
            AudioType::Mixer => Ok(Box::new(AudioMixerNode::new(id, config)?)),
            AudioType::Effect => Ok(Box::new(AudioEffectNode::new(id, config)?)),
            AudioType::Output => Ok(Box::new(AudioOutputNode::new(id, config)?)),
        },
        NodeType::Tally(tally_type) => match tally_type {
            TallyType::Generator => Ok(Box::new(TallyGeneratorNode::new(id, config)?)),
            TallyType::Monitor => Ok(Box::new(TallyMonitorNode::new(id, config)?)),
            TallyType::Logic => Ok(Box::new(TallyLogicNode::new(id, config)?)),
            TallyType::Router => Ok(Box::new(TallyRouterNode::new(id, config)?)),
        },
        NodeType::Control(control_type) => match control_type {
            ControlType::Lfo => Ok(Box::new(LFOController::new(id, config)?)),
            ControlType::Timeline => Ok(Box::new(TimelineController::new(id, config)?)),
            ControlType::MathController => Ok(Box::new(MathController::new(id, config)?)),
            ControlType::MidiController => {
                Err(anyhow::anyhow!("MIDI controller not yet implemented"))
            }
            _ => Err(anyhow::anyhow!(
                "Controller type not yet implemented: {:?}",
                control_type
            )),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node_id = Uuid::new_v4();
        let config = NodeConfig {
            parameters: HashMap::new(),
        };

        let result =
            create_node_processor(NodeType::Input(InputType::TestPattern), node_id, config);

        assert!(result.is_ok());
    }

    #[test]
    fn test_parameter_definition() {
        let param = ParameterDefinition {
            name: "brightness".to_string(),
            parameter_type: ParameterType::Float,
            default_value: serde_json::Value::from(1.0),
            min_value: Some(serde_json::Value::from(0.0)),
            max_value: Some(serde_json::Value::from(2.0)),
            description: "Brightness adjustment".to_string(),
        };

        assert_eq!(param.name, "brightness");
        assert_eq!(param.default_value, serde_json::Value::from(1.0));
    }
}
