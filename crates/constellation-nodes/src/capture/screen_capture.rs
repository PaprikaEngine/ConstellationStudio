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

use crate::{NodeProcessor, NodeProperties, ParameterDefinition, ParameterType};
use anyhow::Result;
use constellation_core::*;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

#[cfg(target_os = "linux")]
use super::linux::LinuxScreenCapture as PlatformScreenCapture;
#[cfg(target_os = "macos")]
use super::macos::MacOSScreenCapture as PlatformScreenCapture;
#[cfg(target_os = "windows")]
use super::windows::WindowsScreenCapture as PlatformScreenCapture;

use super::ScreenCaptureBackend;

pub struct ScreenCaptureNode {
    id: Uuid,
    config: NodeConfig,
    properties: NodeProperties,
    capture_context: Option<PlatformScreenCapture>,
}

impl ScreenCaptureNode {
    pub fn new(id: Uuid, config: NodeConfig) -> Result<Self> {
        let mut parameters = HashMap::new();
        parameters.insert(
            "display_id".to_string(),
            ParameterDefinition {
                name: "Display ID".to_string(),
                parameter_type: ParameterType::Integer,
                default_value: Value::from(0),
                min_value: Some(Value::from(0)),
                max_value: Some(Value::from(10)),
                description: "Display to capture (0 = primary)".to_string(),
            },
        );
        parameters.insert(
            "capture_cursor".to_string(),
            ParameterDefinition {
                name: "Capture Cursor".to_string(),
                parameter_type: ParameterType::Boolean,
                default_value: Value::Bool(true),
                min_value: None,
                max_value: None,
                description: "Include cursor in capture".to_string(),
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
                description: "Capture frame rate".to_string(),
            },
        );
        parameters.insert(
            "region_x".to_string(),
            ParameterDefinition {
                name: "Region X".to_string(),
                parameter_type: ParameterType::Integer,
                default_value: Value::from(0),
                min_value: Some(Value::from(0)),
                max_value: Some(Value::from(7680)),
                description: "Capture region X offset".to_string(),
            },
        );
        parameters.insert(
            "region_y".to_string(),
            ParameterDefinition {
                name: "Region Y".to_string(),
                parameter_type: ParameterType::Integer,
                default_value: Value::from(0),
                min_value: Some(Value::from(0)),
                max_value: Some(Value::from(4320)),
                description: "Capture region Y offset".to_string(),
            },
        );
        parameters.insert(
            "region_width".to_string(),
            ParameterDefinition {
                name: "Region Width".to_string(),
                parameter_type: ParameterType::Integer,
                default_value: Value::from(0), // 0 = full screen
                min_value: Some(Value::from(0)),
                max_value: Some(Value::from(7680)),
                description: "Capture region width (0 = full screen)".to_string(),
            },
        );
        parameters.insert(
            "region_height".to_string(),
            ParameterDefinition {
                name: "Region Height".to_string(),
                parameter_type: ParameterType::Integer,
                default_value: Value::from(0), // 0 = full screen
                min_value: Some(Value::from(0)),
                max_value: Some(Value::from(4320)),
                description: "Capture region height (0 = full screen)".to_string(),
            },
        );

        let properties = NodeProperties {
            id,
            name: "Screen Capture".to_string(),
            node_type: NodeType::Input(InputType::ScreenCapture),
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
        let display_id = self
            .get_parameter("display_id")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as u32;

        let capture_cursor = self
            .get_parameter("capture_cursor")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        self.capture_context = Some(PlatformScreenCapture::new(display_id, capture_cursor)?);
        Ok(())
    }
}

impl NodeProcessor for ScreenCaptureNode {
    fn process(&mut self, _input: FrameData) -> Result<FrameData> {
        if self.capture_context.is_none() {
            self.initialize_capture()?;
        }

        let frame = if let Some(ref mut context) = self.capture_context {
            context.capture_frame()?
        } else {
            return Err(anyhow::anyhow!("Failed to initialize screen capture"));
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
