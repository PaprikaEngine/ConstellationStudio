use crate::{NodeProcessor, NodeProperties, ParameterDefinition, ParameterType};
use anyhow::Result;
use constellation_core::*;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

pub struct VirtualWebcamNode {
    id: Uuid,
    config: NodeConfig,
    properties: NodeProperties,
    output_context: Option<VirtualWebcamContext>,
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
            input_types: vec![ConnectionType::Video, ConnectionType::Audio],
            output_types: vec![],
            parameters,
        };

        Ok(Self {
            id,
            config,
            properties,
            output_context: None,
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

        self.output_context = Some(VirtualWebcamContext::new(device_name, resolution, fps)?);
        Ok(())
    }
}

impl NodeProcessor for VirtualWebcamNode {
    fn process(&mut self, input: FrameData) -> Result<FrameData> {
        if self.output_context.is_none() {
            self.initialize_output()?;
        }

        if let Some(ref mut context) = self.output_context {
            if let Some(ref video_frame) = input.video_data {
                context.send_frame(video_frame)?;
            }
        }

        Ok(input)
    }

    fn get_properties(&self) -> NodeProperties {
        self.properties.clone()
    }

    fn set_parameter(&mut self, key: &str, value: Value) -> Result<()> {
        self.config.parameters.insert(key.to_string(), value);
        self.output_context = None;
        Ok(())
    }

    fn get_parameter(&self, key: &str) -> Option<Value> {
        self.config.parameters.get(key).cloned()
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
            input_types: vec![ConnectionType::Video, ConnectionType::Audio],
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
            video_data: None,
            audio_data: Some(AudioFrame {
                sample_rate: 48000,
                channels: 2,
                samples: vec![0.0; 1024],
            }),
            tally_data: None,
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
            output_types: vec![ConnectionType::Tally],
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
            video_data: None,
            audio_data: None,
            tally_data: Some(TallyData {
                program_tally: false,
                preview_tally: false,
                custom_tally: HashMap::new(),
            }),
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
            input_types: vec![ConnectionType::Tally],
            output_types: vec![ConnectionType::Tally],
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
            input_types: vec![ConnectionType::Tally],
            output_types: vec![ConnectionType::Tally],
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
            input_types: vec![ConnectionType::Tally],
            output_types: vec![ConnectionType::Tally],
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

#[cfg(target_os = "windows")]
pub struct VirtualWebcamContext {
    device_name: String,
    resolution: String,
    fps: u32,
}

#[cfg(target_os = "windows")]
impl VirtualWebcamContext {
    pub fn new(device_name: String, resolution: String, fps: u32) -> Result<Self> {
        Ok(Self {
            device_name,
            resolution,
            fps,
        })
    }

    pub fn send_frame(&mut self, frame: &VideoFrame) -> Result<()> {
        Ok(())
    }
}

#[cfg(target_os = "macos")]
pub struct VirtualWebcamContext {
    device_name: String,
    resolution: String,
    fps: u32,
}

#[cfg(target_os = "macos")]
impl VirtualWebcamContext {
    pub fn new(device_name: String, resolution: String, fps: u32) -> Result<Self> {
        Ok(Self {
            device_name,
            resolution,
            fps,
        })
    }

    pub fn send_frame(&mut self, frame: &VideoFrame) -> Result<()> {
        Ok(())
    }
}

#[cfg(target_os = "linux")]
pub struct VirtualWebcamContext {
    device_name: String,
    resolution: String,
    fps: u32,
}

#[cfg(target_os = "linux")]
impl VirtualWebcamContext {
    pub fn new(device_name: String, resolution: String, fps: u32) -> Result<Self> {
        Ok(Self {
            device_name,
            resolution,
            fps,
        })
    }

    pub fn send_frame(&mut self, frame: &VideoFrame) -> Result<()> {
        Ok(())
    }
}
