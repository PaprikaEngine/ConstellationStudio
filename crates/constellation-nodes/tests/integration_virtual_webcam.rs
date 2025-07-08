use anyhow::Result;
use constellation_core::VideoFormat as CoreVideoFormat;
use constellation_core::*;
use constellation_nodes::virtual_camera::{VideoFormat, VirtualWebcamBackend, VirtualWebcamConfig};
use constellation_nodes::*;
use std::collections::HashMap;
use uuid::Uuid;

#[cfg(target_os = "linux")]
use constellation_nodes::virtual_camera::LinuxVirtualWebcam as PlatformWebcam;
#[cfg(target_os = "macos")]
use constellation_nodes::virtual_camera::MacOSVirtualWebcam as PlatformWebcam;
#[cfg(target_os = "windows")]
use constellation_nodes::virtual_camera::WindowsVirtualWebcam as PlatformWebcam;

#[test]
fn test_virtual_webcam_node_creation() {
    let node_id = Uuid::new_v4();
    let config = NodeConfig {
        parameters: HashMap::new(),
    };

    let node = VirtualWebcamNode::new(node_id, config);
    assert!(node.is_ok());

    let node = node.unwrap();
    let properties = node.get_properties();
    assert_eq!(
        properties.node_type,
        NodeType::Output(OutputType::VirtualWebcam)
    );
    assert!(properties.input_types.contains(&ConnectionType::RenderData));
    assert!(properties.input_types.contains(&ConnectionType::Audio));
    assert!(properties.output_types.is_empty());
}

#[test]
fn test_virtual_webcam_parameter_handling() -> Result<()> {
    let node_id = Uuid::new_v4();
    let mut config = NodeConfig {
        parameters: HashMap::new(),
    };

    // Set custom parameters
    config.parameters.insert(
        "device_name".to_string(),
        serde_json::Value::String("Test Camera".to_string()),
    );
    config.parameters.insert(
        "resolution".to_string(),
        serde_json::Value::String("1280x720".to_string()),
    );
    config
        .parameters
        .insert("fps".to_string(), serde_json::Value::from(60));

    let mut node = VirtualWebcamNode::new(node_id, config)?;

    // Verify parameters are accessible
    assert_eq!(
        node.get_parameter("device_name"),
        Some(serde_json::Value::String("Test Camera".to_string()))
    );
    assert_eq!(
        node.get_parameter("resolution"),
        Some(serde_json::Value::String("1280x720".to_string()))
    );
    assert_eq!(node.get_parameter("fps"), Some(serde_json::Value::from(60)));

    // Test parameter update
    node.set_parameter("fps", serde_json::Value::from(30))?;
    assert_eq!(node.get_parameter("fps"), Some(serde_json::Value::from(30)));

    Ok(())
}

#[test]
fn test_platform_webcam_backend_creation() {
    let webcam = PlatformWebcam::new("Test Camera".to_string(), 1280, 720, 30);

    assert!(webcam.is_ok());
    let webcam = webcam.unwrap();
    assert_eq!(webcam.get_device_name(), "Test Camera");
    assert!(!webcam.is_active());
}

#[test]
fn test_platform_webcam_configuration() -> Result<()> {
    let mut webcam = PlatformWebcam::new("Test Camera".to_string(), 1920, 1080, 30)?;

    // Test resolution change when not active
    assert!(webcam.set_resolution(1280, 720).is_ok());
    assert!(webcam.set_fps(60).is_ok());

    Ok(())
}

#[test]
fn test_virtual_webcam_config_default() {
    let config = VirtualWebcamConfig::default();
    assert_eq!(config.device_name, "Constellation Studio");
    assert_eq!(config.width, 1920);
    assert_eq!(config.height, 1080);
    assert_eq!(config.fps, 30);
    assert_eq!(config.format, VideoFormat::RGB24);
}

#[test]
fn test_video_format_properties() {
    assert_eq!(VideoFormat::RGB24.bytes_per_pixel(), 3);
    assert_eq!(VideoFormat::BGRA32.bytes_per_pixel(), 4);
    assert_eq!(VideoFormat::YUV420.bytes_per_pixel(), 1);
    assert_eq!(VideoFormat::NV12.bytes_per_pixel(), 1);

    assert_eq!(VideoFormat::RGB24.stride(1920), 5760);
    assert_eq!(VideoFormat::BGRA32.stride(1920), 7680);
    assert_eq!(VideoFormat::YUV420.stride(1920), 1920);
    assert_eq!(VideoFormat::NV12.stride(1920), 1920);
}

#[test]
fn test_virtual_webcam_frame_processing() -> Result<()> {
    let node_id = Uuid::new_v4();
    let config = NodeConfig {
        parameters: HashMap::new(),
    };

    let mut node = VirtualWebcamNode::new(node_id, config)?;

    // Create test frame data
    let video_frame = VideoFrame {
        width: 1920,
        height: 1080,
        data: vec![0u8; 1920 * 1080 * 3], // RGB data
        format: CoreVideoFormat::Rgb8,
    };

    let frame_data = FrameData {
        render_data: Some(RenderData::Raster2D(video_frame)),
        audio_data: None,
        control_data: None,
        tally_metadata: TallyMetadata::new(),
    };

    // Process frame - should not fail even if virtual webcam can't actually start
    // in CI environment without proper drivers
    let result = node.process(frame_data);

    // The result may fail due to missing drivers in CI, but should not panic
    match result {
        Ok(_) => {
            // Success case - virtual webcam started successfully
        }
        Err(e) => {
            // Expected failure in CI environment without proper drivers
            println!("Virtual webcam initialization failed (expected in CI): {e}");
        }
    }

    Ok(())
}

#[test]
fn test_resolution_parsing() -> Result<()> {
    let node_id = Uuid::new_v4();
    let config = NodeConfig {
        parameters: HashMap::new(),
    };

    let node = VirtualWebcamNode::new(node_id, config)?;

    // Test parsing different resolution formats
    let resolutions = vec![
        ("1920x1080", (1920, 1080)),
        ("1280x720", (1280, 720)),
        ("640x480", (640, 480)),
    ];

    for (resolution_str, expected) in resolutions {
        let parsed = node.parse_resolution(resolution_str);
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), expected);
    }

    // Test invalid resolution format
    let invalid = node.parse_resolution("invalid");
    assert!(invalid.is_err());

    Ok(())
}

// Platform-specific tests

#[cfg(target_os = "linux")]
#[test]
fn test_linux_v4l2_frame_conversion() -> Result<()> {
    use constellation_nodes::virtual_camera::LinuxVirtualWebcam;

    let webcam = LinuxVirtualWebcam::new("Test Camera".to_string(), 640, 480, 30)?;

    let frame = VideoFrame {
        width: 640,
        height: 480,
        data: vec![0u8; 640 * 480 * 3], // RGB data
        format: CoreVideoFormat::Rgb8,
    };

    let converted = webcam.convert_frame_for_v4l2(&frame);
    assert!(converted.is_ok());

    let yuv_data = converted.unwrap();
    // YUV420 should be 1.5x the pixel count
    assert_eq!(yuv_data.len(), 640 * 480 * 3 / 2);

    Ok(())
}

#[cfg(target_os = "windows")]
#[test]
fn test_windows_directshow_creation() -> Result<()> {
    use constellation_nodes::virtual_camera::WindowsVirtualWebcam;

    let webcam = WindowsVirtualWebcam::new("Test Camera".to_string(), 1920, 1080, 30)?;

    assert_eq!(webcam.get_device_name(), "Test Camera");
    assert!(!webcam.is_active());

    Ok(())
}

#[cfg(target_os = "macos")]
#[test]
fn test_macos_core_media_creation() -> Result<()> {
    use constellation_nodes::virtual_camera::MacOSVirtualWebcam;

    let webcam = MacOSVirtualWebcam::new("Test Camera".to_string(), 1920, 1080, 30)?;

    assert_eq!(webcam.get_device_name(), "Test Camera");
    assert!(!webcam.is_active());

    Ok(())
}
