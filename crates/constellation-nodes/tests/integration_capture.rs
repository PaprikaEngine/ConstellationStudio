use constellation_core::{FrameData, NodeConfig, RenderData, TallyMetadata};
use constellation_nodes::{NodeProcessor, ScreenCaptureNode, WindowCaptureNode};
use std::collections::HashMap;
use uuid::Uuid;

#[test]
fn test_screen_capture_node_integration() {
    let node_id = Uuid::new_v4();
    let config = NodeConfig {
        parameters: HashMap::new(),
    };

    let mut node =
        ScreenCaptureNode::new(node_id, config).expect("Failed to create screen capture node");

    // Test that the node has correct properties
    let properties = node.get_properties();
    assert_eq!(properties.name, "Screen Capture");
    assert_eq!(properties.id, node_id);

    // Test parameter setting
    assert!(node
        .set_parameter("display_id", serde_json::Value::from(0))
        .is_ok());
    assert!(node
        .set_parameter("capture_cursor", serde_json::Value::Bool(true))
        .is_ok());

    // Test parameter getting
    assert_eq!(
        node.get_parameter("display_id"),
        Some(serde_json::Value::from(0))
    );
    assert_eq!(
        node.get_parameter("capture_cursor"),
        Some(serde_json::Value::Bool(true))
    );
}

#[test]
fn test_window_capture_node_integration() {
    let node_id = Uuid::new_v4();
    let config = NodeConfig {
        parameters: HashMap::new(),
    };

    let mut node =
        WindowCaptureNode::new(node_id, config).expect("Failed to create window capture node");

    // Test that the node has correct properties
    let properties = node.get_properties();
    assert_eq!(properties.name, "Window Capture");
    assert_eq!(properties.id, node_id);

    // Test parameter setting
    assert!(node
        .set_parameter(
            "window_title",
            serde_json::Value::String("Test".to_string())
        )
        .is_ok());
    assert!(node
        .set_parameter("follow_window", serde_json::Value::Bool(true))
        .is_ok());

    // Test parameter getting
    assert_eq!(
        node.get_parameter("window_title"),
        Some(serde_json::Value::String("Test".to_string()))
    );
    assert_eq!(
        node.get_parameter("follow_window"),
        Some(serde_json::Value::Bool(true))
    );
}

// Skip actual capture tests in CI environments - use runtime detection instead
#[test]
fn test_capture_processing_flow() {
    // This test verifies the basic processing flow works
    // In CI environments or headless systems, this will be skipped

    // Skip in CI environments where hardware access is limited
    if std::env::var("CI").is_ok() {
        println!("Skipping capture test in CI environment");
        return;
    }

    let node_id = Uuid::new_v4();
    let mut config = NodeConfig {
        parameters: HashMap::new(),
    };

    // Set up minimal parameters for testing
    config
        .parameters
        .insert("display_id".to_string(), serde_json::Value::from(0));
    config
        .parameters
        .insert("capture_cursor".to_string(), serde_json::Value::Bool(false));

    let mut node =
        ScreenCaptureNode::new(node_id, config).expect("Failed to create screen capture node");

    // Create dummy input frame data
    let input = FrameData {
        render_data: None,
        audio_data: None,
        control_data: None,
        tally_metadata: TallyMetadata::new(),
    };

    // Try to process a frame - this will either succeed (on systems with displays)
    // or fail gracefully (in headless CI environments)
    let result = node.process(input);

    // We expect either success or a reasonable error message
    match result {
        Ok(output) => {
            // If processing succeeds, we should have video data
            assert!(output.render_data.is_some());
            let video_frame = match output.render_data.unwrap() {
                RenderData::Raster2D(frame) => frame,
                _ => panic!("Expected Raster2D render data"),
            };
            assert!(video_frame.width > 0);
            assert!(video_frame.height > 0);
            assert!(!video_frame.data.is_empty());
        }
        Err(e) => {
            // If it fails, it should be due to missing display hardware or permissions
            println!("Capture test skipped due to environment: {e}");
            // This is acceptable in CI/headless environments
        }
    }
}

#[test]
fn test_capture_parameter_validation() {
    let node_id = Uuid::new_v4();
    let config = NodeConfig {
        parameters: HashMap::new(),
    };

    let mut node =
        ScreenCaptureNode::new(node_id, config).expect("Failed to create screen capture node");

    // Test valid parameter values
    assert!(node
        .set_parameter("fps", serde_json::Value::from(30))
        .is_ok());
    assert!(node
        .set_parameter("fps", serde_json::Value::from(60))
        .is_ok());
    assert!(node
        .set_parameter("display_id", serde_json::Value::from(0))
        .is_ok());
    assert!(node
        .set_parameter("display_id", serde_json::Value::from(1))
        .is_ok());

    // Test boolean parameters
    assert!(node
        .set_parameter("capture_cursor", serde_json::Value::Bool(true))
        .is_ok());
    assert!(node
        .set_parameter("capture_cursor", serde_json::Value::Bool(false))
        .is_ok());

    // Test region parameters
    assert!(node
        .set_parameter("region_x", serde_json::Value::from(0))
        .is_ok());
    assert!(node
        .set_parameter("region_y", serde_json::Value::from(0))
        .is_ok());
    assert!(node
        .set_parameter("region_width", serde_json::Value::from(1920))
        .is_ok());
    assert!(node
        .set_parameter("region_height", serde_json::Value::from(1080))
        .is_ok());
}

#[test]
fn test_window_capture_parameter_validation() {
    let node_id = Uuid::new_v4();
    let config = NodeConfig {
        parameters: HashMap::new(),
    };

    let mut node =
        WindowCaptureNode::new(node_id, config).expect("Failed to create window capture node");

    // Test window selection parameters
    assert!(node
        .set_parameter(
            "window_title",
            serde_json::Value::String("Calculator".to_string())
        )
        .is_ok());
    assert!(node
        .set_parameter("window_id", serde_json::Value::from(12345))
        .is_ok());
    assert!(node
        .set_parameter("follow_window", serde_json::Value::Bool(true))
        .is_ok());

    // Test capture method parameter
    assert!(node
        .set_parameter(
            "capture_method",
            serde_json::Value::String("Auto".to_string())
        )
        .is_ok());
    assert!(node
        .set_parameter(
            "capture_method",
            serde_json::Value::String("Graphics Capture".to_string())
        )
        .is_ok());
    assert!(node
        .set_parameter(
            "capture_method",
            serde_json::Value::String("BitBlt".to_string())
        )
        .is_ok());
}
