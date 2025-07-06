use crate::{NodeProcessor, NodeProperties, ParameterDefinition, ParameterType};
use anyhow::Result;
use constellation_core::*;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

#[cfg(target_os = "linux")]
use super::linux::LinuxWindowCapture as PlatformWindowCapture;
#[cfg(target_os = "macos")]
use super::macos::MacOSWindowCapture as PlatformWindowCapture;
#[cfg(target_os = "windows")]
use super::windows::WindowsWindowCapture as PlatformWindowCapture;

use super::WindowCaptureBackend;

pub struct WindowCaptureNode {
    id: Uuid,
    config: NodeConfig,
    properties: NodeProperties,
    capture_context: Option<PlatformWindowCapture>,
}

impl WindowCaptureNode {
    pub fn new(id: Uuid, config: NodeConfig) -> Result<Self> {
        let mut parameters = HashMap::new();
        parameters.insert(
            "window_title".to_string(),
            ParameterDefinition {
                name: "Window Title".to_string(),
                parameter_type: ParameterType::String,
                default_value: Value::String("".to_string()),
                min_value: None,
                max_value: None,
                description: "Title of window to capture".to_string(),
            },
        );
        parameters.insert(
            "window_id".to_string(),
            ParameterDefinition {
                name: "Window ID".to_string(),
                parameter_type: ParameterType::Integer,
                default_value: Value::from(0),
                min_value: Some(Value::from(0)),
                max_value: None,
                description: "Window ID to capture".to_string(),
            },
        );
        parameters.insert(
            "follow_window".to_string(),
            ParameterDefinition {
                name: "Follow Window".to_string(),
                parameter_type: ParameterType::Boolean,
                default_value: Value::Bool(true),
                min_value: None,
                max_value: None,
                description: "Follow window size changes".to_string(),
            },
        );
        parameters.insert(
            "capture_method".to_string(),
            ParameterDefinition {
                name: "Capture Method".to_string(),
                parameter_type: ParameterType::Enum(vec![
                    "Auto".to_string(),
                    "Graphics Capture".to_string(),
                    "BitBlt".to_string(),
                ]),
                default_value: Value::String("Auto".to_string()),
                min_value: None,
                max_value: None,
                description: "Window capture method".to_string(),
            },
        );

        let properties = NodeProperties {
            id,
            name: "Window Capture".to_string(),
            node_type: NodeType::Input(InputType::WindowCapture),
            input_types: vec![],
            output_types: vec![ConnectionType::RenderData],
            parameters: parameters.clone(),
        };

        // Initialize config with default values if not provided
        let mut initialized_config = config;
        for (key, param_def) in &parameters {
            if !initialized_config.parameters.contains_key(key) {
                initialized_config
                    .parameters
                    .insert(key.clone(), param_def.default_value.clone());
            }
        }

        Ok(Self {
            id,
            config: initialized_config,
            properties,
            capture_context: None,
        })
    }

    fn initialize_capture(&mut self) -> Result<()> {
        let window_title = self
            .get_parameter("window_title")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();

        let window_id = self
            .get_parameter("window_id")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as u64;

        self.capture_context = if !window_title.is_empty() {
            Some(PlatformWindowCapture::new_by_title(&window_title)?)
        } else if window_id > 0 {
            Some(PlatformWindowCapture::new(window_id)?)
        } else {
            return Err(anyhow::anyhow!(
                "Either window_title or window_id must be specified"
            ));
        };

        Ok(())
    }
}

impl NodeProcessor for WindowCaptureNode {
    fn process(&mut self, _input: FrameData) -> Result<FrameData> {
        if self.capture_context.is_none() {
            self.initialize_capture()?;
        }

        let frame = if let Some(ref mut context) = self.capture_context {
            context.capture_frame()?
        } else {
            return Err(anyhow::anyhow!("Failed to initialize window capture"));
        };

        Ok(FrameData {
            render_data: Some(RenderData::Raster2D(frame)),
            audio_data: None,
            control_data: None,
            tally_metadata: TallyMetadata::new(),
        })
    }

    fn get_properties(&self) -> NodeProperties {
        self.properties.clone()
    }

    fn set_parameter(&mut self, key: &str, value: Value) -> Result<()> {
        self.config.parameters.insert(key.to_string(), value);
        // Reset capture context to apply new parameters
        self.capture_context = None;
        Ok(())
    }

    fn get_parameter(&self, key: &str) -> Option<Value> {
        self.config.parameters.get(key).cloned()
    }
}
