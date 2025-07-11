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

use constellation_core::*;
use constellation_nodes::effects::{BlurNode, ColorCorrectionNode, SharpenNode};
use constellation_nodes::{NodeConfig, NodeProcessor, ParameterType};
use std::collections::HashMap;
use uuid::Uuid;

// TiDD Test Suite: Effects Processing Integration Tests

fn create_test_video_frame(width: u32, height: u32) -> VideoFrame {
    let mut data = vec![0u8; (width * height * 4) as usize];

    // Create a test pattern with different colors
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            data[idx] = (x * 255 / width) as u8; // Red gradient
            data[idx + 1] = (y * 255 / height) as u8; // Green gradient
            data[idx + 2] = 128; // Constant blue
            data[idx + 3] = 255; // Full alpha
        }
    }

    VideoFrame {
        width,
        height,
        format: VideoFormat::Rgba8,
        data,
    }
}

fn create_test_frame_data(width: u32, height: u32) -> FrameData {
    FrameData {
        render_data: Some(RenderData::Raster2D(create_test_video_frame(width, height))),
        audio_data: None,
        control_data: None,
        tally_metadata: TallyMetadata::new(),
    }
}

#[test]
fn test_color_correction_node_creation_and_properties() {
    let node_id = Uuid::new_v4();
    let config = NodeConfig {
        parameters: HashMap::new(),
    };

    let node = ColorCorrectionNode::new(node_id, config);
    assert!(
        node.is_ok(),
        "Color correction node creation should succeed"
    );

    let node = node.unwrap();
    let properties = node.get_properties();

    assert_eq!(properties.id, node_id);
    assert_eq!(properties.name, "Color Correction");
    assert!(matches!(
        properties.node_type,
        NodeType::Effect(EffectType::ColorCorrection)
    ));
    assert_eq!(properties.input_types, vec![ConnectionType::RenderData]);
    assert_eq!(properties.output_types, vec![ConnectionType::RenderData]);

    // Verify required parameters exist
    assert!(properties.parameters.contains_key("brightness"));
    assert!(properties.parameters.contains_key("contrast"));
    assert!(properties.parameters.contains_key("saturation"));
    assert!(properties.parameters.contains_key("hue"));
}

#[test]
fn test_color_correction_node_parameter_defaults() {
    let node_id = Uuid::new_v4();
    let config = NodeConfig {
        parameters: HashMap::new(),
    };

    let node = ColorCorrectionNode::new(node_id, config).unwrap();
    let properties = node.get_properties();

    // Test brightness parameter
    let brightness_param = &properties.parameters["brightness"];
    assert_eq!(brightness_param.name, "Brightness");
    assert!(matches!(
        brightness_param.parameter_type,
        ParameterType::Float
    ));
    assert_eq!(brightness_param.default_value, serde_json::Value::from(1.0));
    assert_eq!(
        brightness_param.min_value,
        Some(serde_json::Value::from(0.0))
    );
    assert_eq!(
        brightness_param.max_value,
        Some(serde_json::Value::from(3.0))
    );

    // Test contrast parameter
    let contrast_param = &properties.parameters["contrast"];
    assert_eq!(contrast_param.name, "Contrast");
    assert_eq!(contrast_param.default_value, serde_json::Value::from(1.0));

    // Test saturation parameter
    let saturation_param = &properties.parameters["saturation"];
    assert_eq!(saturation_param.name, "Saturation");
    assert_eq!(saturation_param.default_value, serde_json::Value::from(1.0));

    // Test hue parameter
    let hue_param = &properties.parameters["hue"];
    assert_eq!(hue_param.name, "Hue");
    assert_eq!(hue_param.default_value, serde_json::Value::from(0.0));
}

#[test]
fn test_color_correction_node_processing() {
    let node_id = Uuid::new_v4();
    let config = NodeConfig {
        parameters: HashMap::new(),
    };

    let mut node = ColorCorrectionNode::new(node_id, config).unwrap();
    let input_frame = create_test_frame_data(64, 64);

    let result = node.process(input_frame);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.render_data.is_some());

    let video_frame = match output.render_data.unwrap() {
        RenderData::Raster2D(frame) => frame,
        _ => panic!("Expected Raster2D render data"),
    };
    assert_eq!(video_frame.width, 64);
    assert_eq!(video_frame.height, 64);
    assert_eq!(video_frame.format, VideoFormat::Rgba8);
    assert_eq!(video_frame.data.len(), 64 * 64 * 4);
}

#[test]
fn test_color_correction_brightness_adjustment() {
    let node_id = Uuid::new_v4();
    let mut config = NodeConfig {
        parameters: HashMap::new(),
    };

    // Set brightness to 2.0 (double brightness)
    config
        .parameters
        .insert("brightness".to_string(), serde_json::Value::from(2.0));

    let mut node = ColorCorrectionNode::new(node_id, config).unwrap();
    let input_frame = create_test_frame_data(4, 4);

    // Get original pixel for comparison
    let original_pixel = match input_frame.render_data.as_ref().unwrap() {
        RenderData::Raster2D(frame) => frame.data[0],
        _ => panic!("Expected Raster2D render data"),
    };

    let result = node.process(input_frame);
    assert!(result.is_ok());

    let output = result.unwrap();
    let video_frame = match output.render_data.unwrap() {
        RenderData::Raster2D(frame) => frame,
        _ => panic!("Expected Raster2D render data"),
    };
    let adjusted_pixel = video_frame.data[0];

    // Brightness should increase pixel values (clamped to 255)
    assert!(adjusted_pixel >= original_pixel);
}

#[test]
fn test_blur_node_creation_and_properties() {
    let node_id = Uuid::new_v4();
    let config = NodeConfig {
        parameters: HashMap::new(),
    };

    let node = BlurNode::new(node_id, config);
    assert!(node.is_ok(), "Blur node creation should succeed");

    let node = node.unwrap();
    let properties = node.get_properties();

    assert_eq!(properties.id, node_id);
    assert_eq!(properties.name, "Blur");
    assert!(matches!(
        properties.node_type,
        NodeType::Effect(EffectType::Blur)
    ));
    assert_eq!(properties.input_types, vec![ConnectionType::RenderData]);
    assert_eq!(properties.output_types, vec![ConnectionType::RenderData]);

    // Verify radius parameter exists
    assert!(properties.parameters.contains_key("radius"));

    let radius_param = &properties.parameters["radius"];
    assert_eq!(radius_param.name, "Radius");
    assert!(matches!(radius_param.parameter_type, ParameterType::Float));
    assert_eq!(radius_param.default_value, serde_json::Value::from(1.0));
    assert_eq!(radius_param.min_value, Some(serde_json::Value::from(0.0)));
    assert_eq!(radius_param.max_value, Some(serde_json::Value::from(50.0)));
}

#[test]
fn test_blur_node_processing() {
    let node_id = Uuid::new_v4();
    let mut config = NodeConfig {
        parameters: HashMap::new(),
    };

    // Set blur radius
    config
        .parameters
        .insert("radius".to_string(), serde_json::Value::from(2.0));

    let mut node = BlurNode::new(node_id, config).unwrap();
    let input_frame = create_test_frame_data(32, 32);

    let result = node.process(input_frame);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.render_data.is_some());

    let video_frame = match output.render_data.unwrap() {
        RenderData::Raster2D(frame) => frame,
        _ => panic!("Expected Raster2D render data"),
    };
    assert_eq!(video_frame.width, 32);
    assert_eq!(video_frame.height, 32);
    assert_eq!(video_frame.format, VideoFormat::Rgba8);
    assert_eq!(video_frame.data.len(), 32 * 32 * 4);
}

#[test]
fn test_blur_node_zero_radius() {
    let node_id = Uuid::new_v4();
    let mut config = NodeConfig {
        parameters: HashMap::new(),
    };

    // Set zero blur radius - should not change image
    config
        .parameters
        .insert("radius".to_string(), serde_json::Value::from(0.0));

    let mut node = BlurNode::new(node_id, config).unwrap();
    let input_frame = create_test_frame_data(16, 16);

    let original_data = match input_frame.render_data.as_ref().unwrap() {
        RenderData::Raster2D(frame) => frame.data.clone(),
        _ => panic!("Expected Raster2D render data"),
    };

    let result = node.process(input_frame);
    assert!(result.is_ok());

    let output = result.unwrap();
    let video_frame = match output.render_data.unwrap() {
        RenderData::Raster2D(frame) => frame,
        _ => panic!("Expected Raster2D render data"),
    };

    // With zero radius, data should be unchanged
    assert_eq!(video_frame.data, original_data);
}

#[test]
fn test_sharpen_node_creation_and_properties() {
    let node_id = Uuid::new_v4();
    let config = NodeConfig {
        parameters: HashMap::new(),
    };

    let node = SharpenNode::new(node_id, config);
    assert!(node.is_ok(), "Sharpen node creation should succeed");

    let node = node.unwrap();
    let properties = node.get_properties();

    assert_eq!(properties.id, node_id);
    assert_eq!(properties.name, "Sharpen");
    assert!(matches!(
        properties.node_type,
        NodeType::Effect(EffectType::Sharpen)
    ));
    assert_eq!(properties.input_types, vec![ConnectionType::RenderData]);
    assert_eq!(properties.output_types, vec![ConnectionType::RenderData]);

    // Verify strength parameter exists
    assert!(properties.parameters.contains_key("strength"));

    let strength_param = &properties.parameters["strength"];
    assert_eq!(strength_param.name, "Strength");
    assert!(matches!(
        strength_param.parameter_type,
        ParameterType::Float
    ));
    assert_eq!(strength_param.default_value, serde_json::Value::from(1.0));
    assert_eq!(strength_param.min_value, Some(serde_json::Value::from(0.0)));
    assert_eq!(strength_param.max_value, Some(serde_json::Value::from(5.0)));
}

#[test]
fn test_sharpen_node_processing() {
    let node_id = Uuid::new_v4();
    let mut config = NodeConfig {
        parameters: HashMap::new(),
    };

    // Set sharpen strength
    config
        .parameters
        .insert("strength".to_string(), serde_json::Value::from(1.5));

    let mut node = SharpenNode::new(node_id, config).unwrap();
    let input_frame = create_test_frame_data(32, 32);

    let result = node.process(input_frame);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.render_data.is_some());

    let video_frame = match output.render_data.unwrap() {
        RenderData::Raster2D(frame) => frame,
        _ => panic!("Expected Raster2D render data"),
    };
    assert_eq!(video_frame.width, 32);
    assert_eq!(video_frame.height, 32);
    assert_eq!(video_frame.format, VideoFormat::Rgba8);
    assert_eq!(video_frame.data.len(), 32 * 32 * 4);
}

#[test]
fn test_sharpen_node_zero_strength() {
    let node_id = Uuid::new_v4();
    let mut config = NodeConfig {
        parameters: HashMap::new(),
    };

    // Set zero sharpen strength - should not change image
    config
        .parameters
        .insert("strength".to_string(), serde_json::Value::from(0.0));

    let mut node = SharpenNode::new(node_id, config).unwrap();
    let input_frame = create_test_frame_data(16, 16);

    let original_data = match input_frame.render_data.as_ref().unwrap() {
        RenderData::Raster2D(frame) => frame.data.clone(),
        _ => panic!("Expected Raster2D render data"),
    };

    let result = node.process(input_frame);
    assert!(result.is_ok());

    let output = result.unwrap();
    let video_frame = match output.render_data.unwrap() {
        RenderData::Raster2D(frame) => frame,
        _ => panic!("Expected Raster2D render data"),
    };

    // With zero strength, data should be unchanged
    assert_eq!(video_frame.data, original_data);
}

#[test]
fn test_effects_chain_processing() {
    // Test chaining multiple effects together
    let node_id1 = Uuid::new_v4();
    let node_id2 = Uuid::new_v4();
    let node_id3 = Uuid::new_v4();

    let mut config1 = NodeConfig {
        parameters: HashMap::new(),
    };
    config1
        .parameters
        .insert("brightness".to_string(), serde_json::Value::from(1.2));
    let mut color_node = ColorCorrectionNode::new(node_id1, config1).unwrap();

    let mut config2 = NodeConfig {
        parameters: HashMap::new(),
    };
    config2
        .parameters
        .insert("radius".to_string(), serde_json::Value::from(1.0));
    let mut blur_node = BlurNode::new(node_id2, config2).unwrap();

    let mut config3 = NodeConfig {
        parameters: HashMap::new(),
    };
    config3
        .parameters
        .insert("strength".to_string(), serde_json::Value::from(0.5));
    let mut sharpen_node = SharpenNode::new(node_id3, config3).unwrap();

    let input_frame = create_test_frame_data(64, 64);

    // Process through effects chain: Color Correction -> Blur -> Sharpen
    let result1 = color_node.process(input_frame);
    assert!(result1.is_ok());

    let result2 = blur_node.process(result1.unwrap());
    assert!(result2.is_ok());

    let result3 = sharpen_node.process(result2.unwrap());
    assert!(result3.is_ok());

    let final_output = result3.unwrap();
    assert!(final_output.render_data.is_some());

    let video_frame = match final_output.render_data.unwrap() {
        RenderData::Raster2D(frame) => frame,
        _ => panic!("Expected Raster2D render data"),
    };
    assert_eq!(video_frame.width, 64);
    assert_eq!(video_frame.height, 64);
    assert_eq!(video_frame.format, VideoFormat::Rgba8);
}

#[test]
fn test_effects_parameter_updates() {
    let node_id = Uuid::new_v4();
    let config = NodeConfig {
        parameters: HashMap::new(),
    };

    let mut node = ColorCorrectionNode::new(node_id, config).unwrap();

    // Test parameter updates
    let result = node.set_parameter("brightness", serde_json::Value::from(2.5));
    assert!(result.is_ok());
    assert_eq!(
        node.get_parameter("brightness"),
        Some(serde_json::Value::from(2.5))
    );

    let result = node.set_parameter("contrast", serde_json::Value::from(1.8));
    assert!(result.is_ok());
    assert_eq!(
        node.get_parameter("contrast"),
        Some(serde_json::Value::from(1.8))
    );

    // Process with new parameters
    let input_frame = create_test_frame_data(32, 32);
    let result = node.process(input_frame);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.render_data.is_some());
}

#[test]
fn test_effects_preserve_non_video_data() {
    let node_id = Uuid::new_v4();
    let config = NodeConfig {
        parameters: HashMap::new(),
    };

    let mut node = ColorCorrectionNode::new(node_id, config).unwrap();

    let input_frame = FrameData {
        render_data: Some(RenderData::Raster2D(create_test_video_frame(32, 32))),
        audio_data: Some(UnifiedAudioData::Stereo {
            sample_rate: 48000,
            channels: 2,
            samples: vec![0.5; 1024],
        }),
        control_data: None,
        tally_metadata: TallyMetadata::new().with_program_tally(true),
    };

    let result = node.process(input_frame);
    assert!(result.is_ok());

    let output = result.unwrap();

    // Video should be processed
    assert!(output.render_data.is_some());

    // Audio should be preserved
    assert!(output.audio_data.is_some());

    let audio = output.audio_data.unwrap();
    match audio {
        UnifiedAudioData::Stereo {
            sample_rate,
            channels,
            samples,
        } => {
            assert_eq!(sample_rate, 48000);
            assert_eq!(channels, 2);
            assert_eq!(samples.len(), 1024);
        }
        _ => panic!("Expected stereo audio data"),
    }

    // Tally metadata should be preserved
    assert!(output.tally_metadata.program_tally);
}

#[test]
fn test_effects_with_no_video_data() {
    let node_id = Uuid::new_v4();
    let config = NodeConfig {
        parameters: HashMap::new(),
    };

    let mut node = BlurNode::new(node_id, config).unwrap();

    let input_frame = FrameData {
        render_data: None,
        audio_data: Some(UnifiedAudioData::Stereo {
            sample_rate: 44100,
            channels: 1,
            samples: vec![0.0; 512],
        }),
        control_data: None,
        tally_metadata: TallyMetadata::new(),
    };

    let result = node.process(input_frame);
    assert!(result.is_ok());

    let output = result.unwrap();

    // No render data should remain None
    assert!(output.render_data.is_none());

    // Audio should be preserved
    assert!(output.audio_data.is_some());
    let audio = output.audio_data.unwrap();
    match audio {
        UnifiedAudioData::Stereo {
            sample_rate,
            channels,
            samples: _,
        } => {
            assert_eq!(sample_rate, 44100);
            assert_eq!(channels, 1);
        }
        _ => panic!("Expected stereo audio data"),
    }
}
