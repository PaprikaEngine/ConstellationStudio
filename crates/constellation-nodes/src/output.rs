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


use crate::virtual_camera::VirtualWebcamBackend;
use crate::{NodeProcessor, NodeProperties, ParameterDefinition, ParameterType};
use anyhow::Result;
use constellation_core::*;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

#[cfg(target_os = "linux")]
use crate::virtual_camera::LinuxVirtualWebcam as PlatformWebcam;
#[cfg(target_os = "macos")]
use crate::virtual_camera::MacOSVirtualWebcam as PlatformWebcam;
#[cfg(target_os = "windows")]
use crate::virtual_camera::WindowsVirtualWebcam as PlatformWebcam;

pub struct VirtualWebcamNode {
    id: Uuid,
    config: NodeConfig,
    properties: NodeProperties,
    webcam_backend: Option<PlatformWebcam>,
}

impl VirtualWebcamNode {
    pub fn new(id: Uuid, config: NodeConfig) -> Result<Self> {
        let mut parameters = HashMap::new();
        parameters.insert(
            "device_name".to_string(),
            ParameterDefinition {
                name: "Device Name".to_string(),
                parameter_type: ParameterType::String,
                default_value: Value::String("Constellation Studio".to_string()),
                min_value: None,
                max_value: None,
                description: "Virtual camera device name".to_string(),
            },
        );
        parameters.insert(
            "resolution".to_string(),
            ParameterDefinition {
                name: "Resolution".to_string(),
                parameter_type: ParameterType::Enum(vec![
                    "1920x1080".to_string(),
                    "1280x720".to_string(),
                    "640x480".to_string(),
                ]),
                default_value: Value::String("1920x1080".to_string()),
                min_value: None,
                max_value: None,
                description: "Output resolution".to_string(),
            },
        );
        parameters.insert(
            "fps".to_string(),
            ParameterDefinition {
                name: "Frame Rate".to_string(),
                parameter_type: ParameterType::Integer,
                default_value: Value::from(30),
                min_value: Some(Value::from(1)),
                max_value: Some(Value::from(60)),
                description: "Output frame rate".to_string(),
            },
        );

        let properties = NodeProperties {
            id,
            name: "Virtual Webcam".to_string(),
            node_type: NodeType::Output(OutputType::VirtualWebcam),
            input_types: vec![ConnectionType::RenderData, ConnectionType::Audio],
            output_types: vec![],
            parameters,
        };

        Ok(Self {
            id,
            config,
            properties,
            webcam_backend: None,
        })
    }

    fn initialize_output(&mut self) -> Result<()> {
        let device_name = self
            .get_parameter("device_name")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "Constellation Studio".to_string());

        let resolution = self
            .get_parameter("resolution")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "1920x1080".to_string());

        let fps = self
            .get_parameter("fps")
            .and_then(|v| v.as_i64())
            .unwrap_or(30) as u32;

        // Parse resolution string
        let (width, height) = self.parse_resolution(&resolution)?;

        // Create platform-specific webcam backend
        let mut webcam = PlatformWebcam::new(device_name, width, height, fps)?;
        webcam.start()?;

        self.webcam_backend = Some(webcam);
        Ok(())
    }

    pub fn parse_resolution(&self, resolution: &str) -> Result<(u32, u32)> {
        let parts: Vec<&str> = resolution.split('x').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid resolution format: {}", resolution));
        }

        let width = parts[0].parse::<u32>()?;
        let height = parts[1].parse::<u32>()?;
        Ok((width, height))
    }
}

impl NodeProcessor for VirtualWebcamNode {
    fn process(&mut self, input: FrameData) -> Result<FrameData> {
        if self.webcam_backend.is_none() {
            self.initialize_output()?;
        }

        if let Some(ref mut webcam) = self.webcam_backend {
            if let Some(RenderData::Raster2D(ref video_frame)) = input.render_data {
                webcam.send_frame(video_frame)?;
            }
        }

        Ok(input)
    }

    fn generate_tally_state(&self) -> TallyMetadata {
        // Virtual Webcamは出力中はProgram Tallyを生成
        if self.webcam_backend.is_some() {
            TallyMetadata::new().with_program_tally(true)
        } else {
            TallyMetadata::new()
        }
    }

    fn get_properties(&self) -> NodeProperties {
        self.properties.clone()
    }

    fn set_parameter(&mut self, key: &str, value: Value) -> Result<()> {
        self.config.parameters.insert(key.to_string(), value);
        // Stop current webcam when parameters change
        if let Some(ref mut webcam) = self.webcam_backend {
            if let Err(e) = webcam.stop() {
                tracing::warn!("Failed to stop webcam on parameter change: {}", e);
            }
        }
        self.webcam_backend = None;
        Ok(())
    }

    fn get_parameter(&self, key: &str) -> Option<Value> {
        self.config.parameters.get(key).cloned()
    }
}

impl Drop for VirtualWebcamNode {
    fn drop(&mut self) {
        if let Some(ref mut webcam) = self.webcam_backend {
            if let Err(e) = webcam.stop() {
                tracing::error!("Failed to stop webcam on drop: {}", e);
            }
        }
    }
}

pub struct PreviewNode {
    id: Uuid,
    config: NodeConfig,
    properties: NodeProperties,
}

impl PreviewNode {
    pub fn new(id: Uuid, config: NodeConfig) -> Result<Self> {
        let mut parameters = HashMap::new();
        parameters.insert(
            "window_title".to_string(),
            ParameterDefinition {
                name: "Window Title".to_string(),
                parameter_type: ParameterType::String,
                default_value: Value::String("Preview".to_string()),
                min_value: None,
                max_value: None,
                description: "Preview window title".to_string(),
            },
        );
        parameters.insert(
            "show_stats".to_string(),
            ParameterDefinition {
                name: "Show Stats".to_string(),
                parameter_type: ParameterType::Boolean,
                default_value: Value::Bool(true),
                min_value: None,
                max_value: None,
                description: "Show performance statistics".to_string(),
            },
        );

        let properties = NodeProperties {
            id,
            name: "Preview".to_string(),
            node_type: NodeType::Output(OutputType::Preview),
            input_types: vec![ConnectionType::RenderData, ConnectionType::Audio],
            output_types: vec![],
            parameters,
        };

        Ok(Self {
            id,
            config,
            properties,
        })
    }
}

impl NodeProcessor for PreviewNode {
    fn process(&mut self, input: FrameData) -> Result<FrameData> {
        Ok(input)
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

pub struct AudioInputNode {
    id: Uuid,
    config: NodeConfig,
    properties: NodeProperties,
}

impl AudioInputNode {
    pub fn new(id: Uuid, config: NodeConfig) -> Result<Self> {
        let mut parameters = HashMap::new();
        parameters.insert(
            "device_id".to_string(),
            ParameterDefinition {
                name: "Device ID".to_string(),
                parameter_type: ParameterType::String,
                default_value: Value::String("default".to_string()),
                min_value: None,
                max_value: None,
                description: "Audio input device".to_string(),
            },
        );

        let properties = NodeProperties {
            id,
            name: "Audio Input".to_string(),
            node_type: NodeType::Audio(AudioType::Input),
            input_types: vec![],
            output_types: vec![ConnectionType::Audio],
            parameters,
        };

        Ok(Self {
            id,
            config,
            properties,
        })
    }
}

impl NodeProcessor for AudioInputNode {
    fn process(&mut self, _input: FrameData) -> Result<FrameData> {
        Ok(FrameData {
            render_data: None,
            audio_data: Some(UnifiedAudioData::Stereo {
                sample_rate: 48000,
                channels: 2,
                samples: vec![0.0; 1024],
            }),
            control_data: None,
            tally_metadata: TallyMetadata::new(),
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

pub struct AudioMixerNode {
    id: Uuid,
    config: NodeConfig,
    properties: NodeProperties,
}

impl AudioMixerNode {
    pub fn new(id: Uuid, config: NodeConfig) -> Result<Self> {
        let mut parameters = HashMap::new();
        parameters.insert(
            "master_volume".to_string(),
            ParameterDefinition {
                name: "Master Volume".to_string(),
                parameter_type: ParameterType::Float,
                default_value: Value::from(1.0),
                min_value: Some(Value::from(0.0)),
                max_value: Some(Value::from(2.0)),
                description: "Master volume level".to_string(),
            },
        );

        let properties = NodeProperties {
            id,
            name: "Audio Mixer".to_string(),
            node_type: NodeType::Audio(AudioType::Mixer),
            input_types: vec![ConnectionType::Audio],
            output_types: vec![ConnectionType::Audio],
            parameters,
        };

        Ok(Self {
            id,
            config,
            properties,
        })
    }
}

impl NodeProcessor for AudioMixerNode {
    fn process(&mut self, input: FrameData) -> Result<FrameData> {
        Ok(input)
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

pub struct AudioEffectNode {
    id: Uuid,
    config: NodeConfig,
    properties: NodeProperties,
}

impl AudioEffectNode {
    pub fn new(id: Uuid, config: NodeConfig) -> Result<Self> {
        let properties = NodeProperties {
            id,
            name: "Audio Effect".to_string(),
            node_type: NodeType::Audio(AudioType::Effect),
            input_types: vec![ConnectionType::Audio],
            output_types: vec![ConnectionType::Audio],
            parameters: HashMap::new(),
        };

        Ok(Self {
            id,
            config,
            properties,
        })
    }
}

impl NodeProcessor for AudioEffectNode {
    fn process(&mut self, input: FrameData) -> Result<FrameData> {
        Ok(input)
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

pub struct AudioOutputNode {
    id: Uuid,
    config: NodeConfig,
    properties: NodeProperties,
}

impl AudioOutputNode {
    pub fn new(id: Uuid, config: NodeConfig) -> Result<Self> {
        let properties = NodeProperties {
            id,
            name: "Audio Output".to_string(),
            node_type: NodeType::Audio(AudioType::Output),
            input_types: vec![ConnectionType::Audio],
            output_types: vec![],
            parameters: HashMap::new(),
        };

        Ok(Self {
            id,
            config,
            properties,
        })
    }
}

impl NodeProcessor for AudioOutputNode {
    fn process(&mut self, input: FrameData) -> Result<FrameData> {
        Ok(input)
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

pub struct TallyGeneratorNode {
    id: Uuid,
    config: NodeConfig,
    properties: NodeProperties,
}

impl TallyGeneratorNode {
    pub fn new(id: Uuid, config: NodeConfig) -> Result<Self> {
        let properties = NodeProperties {
            id,
            name: "Tally Generator".to_string(),
            node_type: NodeType::Tally(TallyType::Generator),
            input_types: vec![],
            output_types: vec![ConnectionType::Control],
            parameters: HashMap::new(),
        };

        Ok(Self {
            id,
            config,
            properties,
        })
    }
}

impl NodeProcessor for TallyGeneratorNode {
    fn process(&mut self, _input: FrameData) -> Result<FrameData> {
        Ok(FrameData {
            render_data: None,
            audio_data: None,
            control_data: Some(ControlData::Parameter {
                target_node_id: self.id,
                parameter_name: "tally_state".to_string(),
                value: ParameterValue::Boolean(true),
            }),
            tally_metadata: TallyMetadata::new().with_program_tally(true),
        })
    }

    fn generate_tally_state(&self) -> TallyMetadata {
        // TallyGeneratorは常にProgram Tallyを生成
        TallyMetadata::new().with_program_tally(true)
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

pub struct TallyMonitorNode {
    id: Uuid,
    config: NodeConfig,
    properties: NodeProperties,
}

impl TallyMonitorNode {
    pub fn new(id: Uuid, config: NodeConfig) -> Result<Self> {
        let properties = NodeProperties {
            id,
            name: "Tally Monitor".to_string(),
            node_type: NodeType::Tally(TallyType::Monitor),
            input_types: vec![ConnectionType::Control],
            output_types: vec![ConnectionType::Control],
            parameters: HashMap::new(),
        };

        Ok(Self {
            id,
            config,
            properties,
        })
    }
}

impl NodeProcessor for TallyMonitorNode {
    fn process(&mut self, input: FrameData) -> Result<FrameData> {
        Ok(input)
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

pub struct TallyLogicNode {
    id: Uuid,
    config: NodeConfig,
    properties: NodeProperties,
}

impl TallyLogicNode {
    pub fn new(id: Uuid, config: NodeConfig) -> Result<Self> {
        let properties = NodeProperties {
            id,
            name: "Tally Logic".to_string(),
            node_type: NodeType::Tally(TallyType::Logic),
            input_types: vec![ConnectionType::Control],
            output_types: vec![ConnectionType::Control],
            parameters: HashMap::new(),
        };

        Ok(Self {
            id,
            config,
            properties,
        })
    }
}

impl NodeProcessor for TallyLogicNode {
    fn process(&mut self, input: FrameData) -> Result<FrameData> {
        Ok(input)
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

pub struct TallyRouterNode {
    id: Uuid,
    config: NodeConfig,
    properties: NodeProperties,
}

impl TallyRouterNode {
    pub fn new(id: Uuid, config: NodeConfig) -> Result<Self> {
        let properties = NodeProperties {
            id,
            name: "Tally Router".to_string(),
            node_type: NodeType::Tally(TallyType::Router),
            input_types: vec![ConnectionType::Control],
            output_types: vec![ConnectionType::Control],
            parameters: HashMap::new(),
        };

        Ok(Self {
            id,
            config,
            properties,
        })
    }
}

impl NodeProcessor for TallyRouterNode {
    fn process(&mut self, input: FrameData) -> Result<FrameData> {
        Ok(input)
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

// Virtual webcam implementation moved to virtual_camera module
