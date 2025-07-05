use constellation_core::*;
use constellation_nodes::input::CameraInputNode;
use constellation_nodes::camera::CameraCapture;
use constellation_nodes::{NodeProcessor, NodeConfig, ParameterType};
use std::collections::HashMap;
use uuid::Uuid;

// TiDD Test Suite: Camera Input Integration Tests

#[test]
fn test_camera_input_node_creation_and_properties() {
    let node_id = Uuid::new_v4();
    let config = NodeConfig {
        parameters: HashMap::new(),
    };
    
    let node = CameraInputNode::new(node_id, config);
    assert!(node.is_ok(), "Camera input node creation should succeed");
    
    let node = node.unwrap();
    let properties = node.get_properties();
    
    assert_eq!(properties.id, node_id);
    assert_eq!(properties.name, "Camera Input");
    assert!(matches!(properties.node_type, NodeType::Input(InputType::Camera)));
    assert!(properties.input_types.is_empty());
    assert_eq!(properties.output_types, vec![ConnectionType::Video, ConnectionType::Audio]);
    
    // Verify required parameters exist
    assert!(properties.parameters.contains_key("device_id"));
    assert!(properties.parameters.contains_key("resolution"));
    assert!(properties.parameters.contains_key("fps"));
}

#[test]
fn test_camera_input_node_parameter_defaults() {
    let node_id = Uuid::new_v4();
    let config = NodeConfig {
        parameters: HashMap::new(),
    };
    
    let node = CameraInputNode::new(node_id, config).unwrap();
    let properties = node.get_properties();
    
    // Test device_id parameter
    let device_param = &properties.parameters["device_id"];
    assert_eq!(device_param.name, "Device ID");
    assert!(matches!(device_param.parameter_type, ParameterType::String));
    assert_eq!(device_param.default_value, serde_json::Value::String("default".to_string()));
    
    // Test resolution parameter
    let resolution_param = &properties.parameters["resolution"];
    assert_eq!(resolution_param.name, "Resolution");
    assert!(matches!(resolution_param.parameter_type, ParameterType::Enum(_)));
    assert_eq!(resolution_param.default_value, serde_json::Value::String("1920x1080".to_string()));
    
    // Test fps parameter
    let fps_param = &properties.parameters["fps"];
    assert_eq!(fps_param.name, "Frame Rate");
    assert!(matches!(fps_param.parameter_type, ParameterType::Integer));
    assert_eq!(fps_param.default_value, serde_json::Value::from(30));
    assert_eq!(fps_param.min_value, Some(serde_json::Value::from(1)));
    assert_eq!(fps_param.max_value, Some(serde_json::Value::from(60)));
}

#[test] 
fn test_camera_input_node_parameter_updates() {
    let node_id = Uuid::new_v4();
    let config = NodeConfig {
        parameters: HashMap::new(),
    };
    
    let mut node = CameraInputNode::new(node_id, config).unwrap();
    
    // Test setting device ID
    let result = node.set_parameter("device_id", serde_json::Value::String("1".to_string()));
    assert!(result.is_ok());
    assert_eq!(node.get_parameter("device_id"), Some(serde_json::Value::String("1".to_string())));
    
    // Test setting resolution
    let result = node.set_parameter("resolution", serde_json::Value::String("1280x720".to_string()));
    assert!(result.is_ok());
    assert_eq!(node.get_parameter("resolution"), Some(serde_json::Value::String("1280x720".to_string())));
    
    // Test setting fps
    let result = node.set_parameter("fps", serde_json::Value::from(60));
    assert!(result.is_ok());
    assert_eq!(node.get_parameter("fps"), Some(serde_json::Value::from(60)));
}

#[test]
fn test_camera_input_node_frame_processing_without_camera() {
    let node_id = Uuid::new_v4();
    let config = NodeConfig {
        parameters: HashMap::new(),
    };
    
    let mut node = CameraInputNode::new(node_id, config).unwrap();
    
    let input_frame = FrameData {
        video_data: None,
        audio_data: None,
        tally_data: None,
        scene3d_data: None,
        spatial_audio_data: None,
        transform_data: None,
    };
    
    // Should return fallback frame when no camera is available
    let result = node.process(input_frame);
    assert!(result.is_ok());
    
    let output = result.unwrap();
    assert!(output.video_data.is_some());
    
    let video_frame = output.video_data.unwrap();
    assert_eq!(video_frame.format, VideoFormat::Rgba8);
    assert!(!video_frame.data.is_empty());
    
    // Should have red error pattern (fallback frame)
    let pixel_data = &video_frame.data[0..4];
    assert!(pixel_data[0] > 0); // Red component should be non-zero for error pattern
}

#[test]
fn test_camera_input_node_resolution_parsing() {
    let node_id = Uuid::new_v4();
    let mut config = NodeConfig {
        parameters: HashMap::new(),
    };
    
    // Set a specific resolution
    config.parameters.insert("resolution".to_string(), serde_json::Value::String("640x480".to_string()));
    
    let mut node = CameraInputNode::new(node_id, config).unwrap();
    
    let input_frame = FrameData {
        video_data: None,
        audio_data: None,
        tally_data: None,
        scene3d_data: None,
        spatial_audio_data: None,
        transform_data: None,
    };
    
    let result = node.process(input_frame);
    assert!(result.is_ok());
    
    let output = result.unwrap();
    let video_frame = output.video_data.unwrap();
    
    // Should use the specified resolution for fallback frame
    assert_eq!(video_frame.width, 640);
    assert_eq!(video_frame.height, 480);
}

#[test]
fn test_camera_capture_device_enumeration() {
    // Test camera device listing (may fail in CI without cameras)
    let result = CameraCapture::list_devices();
    
    match result {
        Ok(devices) => {
            println!("Found {} camera devices", devices.len());
            for device in devices {
                println!("  Device {}: {} - {}", device.index, device.name, device.description);
                assert!(device.index < 100); // Reasonable device index
                assert!(!device.name.is_empty());
                assert!(!device.description.is_empty());
            }
        }
        Err(e) => {
            println!("Camera enumeration failed (expected in CI): {}", e);
            // This is acceptable in CI environments
        }
    }
}

#[test]
fn test_camera_capture_creation_and_lifecycle() {
    let result = CameraCapture::new(0, 640, 480, 30);
    
    match result {
        Ok(mut capture) => {
            println!("Camera capture created successfully");
            assert!(!capture.is_running());
            assert_eq!(capture.fps(), 30.0);
            
            // Test start capture (may fail without camera)
            let start_result = capture.start_capture();
            match start_result {
                Ok(_) => {
                    println!("Camera started successfully");
                    assert!(capture.is_running());
                    
                    // Test frame capture
                    let frame_result = capture.capture_frame();
                    match frame_result {
                        Ok(frame) => {
                            println!("Frame captured: {}x{}", frame.width, frame.height);
                            assert!(frame.width > 0);
                            assert!(frame.height > 0);
                            assert!(!frame.data.is_empty());
                        }
                        Err(e) => println!("Frame capture failed: {}", e),
                    }
                    
                    // Test stop capture
                    let stop_result = capture.stop_capture();
                    assert!(stop_result.is_ok());
                    assert!(!capture.is_running());
                }
                Err(e) => {
                    println!("Camera start failed (expected in CI): {}", e);
                }
            }
        }
        Err(e) => {
            println!("Camera creation failed (expected in CI): {}", e);
        }
    }
}

#[test]
fn test_camera_input_with_valid_parameters() {
    let node_id = Uuid::new_v4();
    let mut config = NodeConfig {
        parameters: HashMap::new(),
    };
    
    // Set realistic camera parameters
    config.parameters.insert("device_id".to_string(), serde_json::Value::String("0".to_string()));
    config.parameters.insert("resolution".to_string(), serde_json::Value::String("1280x720".to_string()));
    config.parameters.insert("fps".to_string(), serde_json::Value::from(30));
    
    let mut node = CameraInputNode::new(node_id, config).unwrap();
    
    let input_frame = FrameData {
        video_data: None,
        audio_data: None,
        tally_data: None,
        scene3d_data: None,
        spatial_audio_data: None,
        transform_data: None,
    };
    
    let result = node.process(input_frame);
    assert!(result.is_ok());
    
    let output = result.unwrap();
    assert!(output.video_data.is_some());
    
    let video_frame = output.video_data.unwrap();
    assert_eq!(video_frame.width, 1280);
    assert_eq!(video_frame.height, 720);
    assert_eq!(video_frame.format, VideoFormat::Rgba8);
    assert_eq!(video_frame.data.len(), 1280 * 720 * 4);
}

#[test]
fn test_camera_input_parameter_reset_behavior() {
    let node_id = Uuid::new_v4();
    let config = NodeConfig {
        parameters: HashMap::new(),
    };
    
    let mut node = CameraInputNode::new(node_id, config).unwrap();
    
    // Process once to initialize camera
    let input_frame = FrameData {
        video_data: None,
        audio_data: None,
        tally_data: None,
        scene3d_data: None,
        spatial_audio_data: None,
        transform_data: None,
    };
    
    let _ = node.process(input_frame.clone());
    
    // Change parameters - should reset camera
    let result = node.set_parameter("resolution", serde_json::Value::String("640x480".to_string()));
    assert!(result.is_ok());
    
    // Process again - should use new parameters
    let result = node.process(input_frame);
    assert!(result.is_ok());
    
    let output = result.unwrap();
    let video_frame = output.video_data.unwrap();
    assert_eq!(video_frame.width, 640);
    assert_eq!(video_frame.height, 480);
}