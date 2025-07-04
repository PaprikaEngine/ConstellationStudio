use constellation_core::*;
use crate::{NodeProcessor, NodeProperties, ParameterDefinition, ParameterType};
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

pub struct ColorCorrectionNode {
    id: Uuid,
    config: NodeConfig,
    properties: NodeProperties,
}

impl ColorCorrectionNode {
    pub fn new(id: Uuid, config: NodeConfig) -> Result<Self> {
        let mut parameters = HashMap::new();
        parameters.insert("brightness".to_string(), ParameterDefinition {
            name: "Brightness".to_string(),
            parameter_type: ParameterType::Float,
            default_value: Value::from(1.0),
            min_value: Some(Value::from(0.0)),
            max_value: Some(Value::from(3.0)),
            description: "Brightness adjustment".to_string(),
        });
        parameters.insert("contrast".to_string(), ParameterDefinition {
            name: "Contrast".to_string(),
            parameter_type: ParameterType::Float,
            default_value: Value::from(1.0),
            min_value: Some(Value::from(0.0)),
            max_value: Some(Value::from(3.0)),
            description: "Contrast adjustment".to_string(),
        });
        parameters.insert("saturation".to_string(), ParameterDefinition {
            name: "Saturation".to_string(),
            parameter_type: ParameterType::Float,
            default_value: Value::from(1.0),
            min_value: Some(Value::from(0.0)),
            max_value: Some(Value::from(3.0)),
            description: "Saturation adjustment".to_string(),
        });
        parameters.insert("hue".to_string(), ParameterDefinition {
            name: "Hue".to_string(),
            parameter_type: ParameterType::Float,
            default_value: Value::from(0.0),
            min_value: Some(Value::from(-180.0)),
            max_value: Some(Value::from(180.0)),
            description: "Hue adjustment in degrees".to_string(),
        });

        let properties = NodeProperties {
            id,
            name: "Color Correction".to_string(),
            node_type: NodeType::Effect(EffectType::ColorCorrection),
            input_types: vec![ConnectionType::Video],
            output_types: vec![ConnectionType::Video],
            parameters,
        };

        Ok(Self {
            id,
            config,
            properties,
        })
    }
}

impl NodeProcessor for ColorCorrectionNode {
    fn process(&mut self, input: FrameData) -> Result<FrameData> {
        let mut output = input;
        
        if let Some(ref mut video_frame) = output.video_data {
            let brightness = self.get_parameter("brightness")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0) as f32;
            let contrast = self.get_parameter("contrast")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0) as f32;
            let saturation = self.get_parameter("saturation")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0) as f32;
            let hue = self.get_parameter("hue")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32;

            self.apply_color_correction(video_frame, brightness, contrast, saturation, hue);
        }

        Ok(output)
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

impl ColorCorrectionNode {
    fn apply_color_correction(&self, frame: &mut VideoFrame, brightness: f32, contrast: f32, saturation: f32, hue: f32) {
        let pixel_count = (frame.width * frame.height) as usize;
        let bytes_per_pixel = match frame.format {
            VideoFormat::Rgba8 | VideoFormat::Bgra8 => 4,
            VideoFormat::Rgb8 | VideoFormat::Bgr8 => 3,
            _ => 4,
        };

        for i in 0..pixel_count {
            let pixel_offset = i * bytes_per_pixel;
            if pixel_offset + 2 < frame.data.len() {
                let r = frame.data[pixel_offset] as f32 / 255.0;
                let g = frame.data[pixel_offset + 1] as f32 / 255.0;
                let b = frame.data[pixel_offset + 2] as f32 / 255.0;

                let (r_adj, g_adj, b_adj) = self.adjust_pixel(r, g, b, brightness, contrast, saturation, hue);

                frame.data[pixel_offset] = (r_adj * 255.0).clamp(0.0, 255.0) as u8;
                frame.data[pixel_offset + 1] = (g_adj * 255.0).clamp(0.0, 255.0) as u8;
                frame.data[pixel_offset + 2] = (b_adj * 255.0).clamp(0.0, 255.0) as u8;
            }
        }
    }

    fn adjust_pixel(&self, r: f32, g: f32, b: f32, brightness: f32, contrast: f32, saturation: f32, hue: f32) -> (f32, f32, f32) {
        let r_adj = ((r - 0.5) * contrast + 0.5) * brightness;
        let g_adj = ((g - 0.5) * contrast + 0.5) * brightness;
        let b_adj = ((b - 0.5) * contrast + 0.5) * brightness;

        (r_adj, g_adj, b_adj)
    }
}

pub struct BlurNode {
    id: Uuid,
    config: NodeConfig,
    properties: NodeProperties,
}

impl BlurNode {
    pub fn new(id: Uuid, config: NodeConfig) -> Result<Self> {
        let mut parameters = HashMap::new();
        parameters.insert("radius".to_string(), ParameterDefinition {
            name: "Radius".to_string(),
            parameter_type: ParameterType::Float,
            default_value: Value::from(1.0),
            min_value: Some(Value::from(0.0)),
            max_value: Some(Value::from(50.0)),
            description: "Blur radius".to_string(),
        });
        parameters.insert("quality".to_string(), ParameterDefinition {
            name: "Quality".to_string(),
            parameter_type: ParameterType::Enum(vec!["Low".to_string(), "Medium".to_string(), "High".to_string()]),
            default_value: Value::String("Medium".to_string()),
            min_value: None,
            max_value: None,
            description: "Blur quality".to_string(),
        });

        let properties = NodeProperties {
            id,
            name: "Blur".to_string(),
            node_type: NodeType::Effect(EffectType::Blur),
            input_types: vec![ConnectionType::Video],
            output_types: vec![ConnectionType::Video],
            parameters,
        };

        Ok(Self {
            id,
            config,
            properties,
        })
    }
}

impl NodeProcessor for BlurNode {
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

pub struct SharpenNode {
    id: Uuid,
    config: NodeConfig,
    properties: NodeProperties,
}

impl SharpenNode {
    pub fn new(id: Uuid, config: NodeConfig) -> Result<Self> {
        let mut parameters = HashMap::new();
        parameters.insert("strength".to_string(), ParameterDefinition {
            name: "Strength".to_string(),
            parameter_type: ParameterType::Float,
            default_value: Value::from(1.0),
            min_value: Some(Value::from(0.0)),
            max_value: Some(Value::from(5.0)),
            description: "Sharpening strength".to_string(),
        });

        let properties = NodeProperties {
            id,
            name: "Sharpen".to_string(),
            node_type: NodeType::Effect(EffectType::Sharpen),
            input_types: vec![ConnectionType::Video],
            output_types: vec![ConnectionType::Video],
            parameters,
        };

        Ok(Self {
            id,
            config,
            properties,
        })
    }
}

impl NodeProcessor for SharpenNode {
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

pub struct TransformNode {
    id: Uuid,
    config: NodeConfig,
    properties: NodeProperties,
}

impl TransformNode {
    pub fn new(id: Uuid, config: NodeConfig) -> Result<Self> {
        let mut parameters = HashMap::new();
        parameters.insert("position".to_string(), ParameterDefinition {
            name: "Position".to_string(),
            parameter_type: ParameterType::Vector2,
            default_value: Value::Array(vec![Value::from(0.0), Value::from(0.0)]),
            min_value: None,
            max_value: None,
            description: "Position offset (X, Y)".to_string(),
        });
        parameters.insert("scale".to_string(), ParameterDefinition {
            name: "Scale".to_string(),
            parameter_type: ParameterType::Vector2,
            default_value: Value::Array(vec![Value::from(1.0), Value::from(1.0)]),
            min_value: Some(Value::Array(vec![Value::from(0.1), Value::from(0.1)])),
            max_value: Some(Value::Array(vec![Value::from(10.0), Value::from(10.0)])),
            description: "Scale factor (X, Y)".to_string(),
        });
        parameters.insert("rotation".to_string(), ParameterDefinition {
            name: "Rotation".to_string(),
            parameter_type: ParameterType::Float,
            default_value: Value::from(0.0),
            min_value: Some(Value::from(-360.0)),
            max_value: Some(Value::from(360.0)),
            description: "Rotation angle in degrees".to_string(),
        });

        let properties = NodeProperties {
            id,
            name: "Transform".to_string(),
            node_type: NodeType::Effect(EffectType::Transform),
            input_types: vec![ConnectionType::Video],
            output_types: vec![ConnectionType::Video],
            parameters,
        };

        Ok(Self {
            id,
            config,
            properties,
        })
    }
}

impl NodeProcessor for TransformNode {
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

pub struct CompositeNode {
    id: Uuid,
    config: NodeConfig,
    properties: NodeProperties,
}

impl CompositeNode {
    pub fn new(id: Uuid, config: NodeConfig) -> Result<Self> {
        let mut parameters = HashMap::new();
        parameters.insert("blend_mode".to_string(), ParameterDefinition {
            name: "Blend Mode".to_string(),
            parameter_type: ParameterType::Enum(vec![
                "Normal".to_string(),
                "Multiply".to_string(),
                "Screen".to_string(),
                "Overlay".to_string(),
                "Add".to_string(),
                "Subtract".to_string(),
            ]),
            default_value: Value::String("Normal".to_string()),
            min_value: None,
            max_value: None,
            description: "Blending mode".to_string(),
        });
        parameters.insert("opacity".to_string(), ParameterDefinition {
            name: "Opacity".to_string(),
            parameter_type: ParameterType::Float,
            default_value: Value::from(1.0),
            min_value: Some(Value::from(0.0)),
            max_value: Some(Value::from(1.0)),
            description: "Opacity level".to_string(),
        });

        let properties = NodeProperties {
            id,
            name: "Composite".to_string(),
            node_type: NodeType::Effect(EffectType::Composite),
            input_types: vec![ConnectionType::Video, ConnectionType::Video],
            output_types: vec![ConnectionType::Video],
            parameters,
        };

        Ok(Self {
            id,
            config,
            properties,
        })
    }
}

impl NodeProcessor for CompositeNode {
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