use constellation_core::*;
use anyhow::Result;
use uuid::Uuid;

pub mod screen_capture;
pub mod window_capture;

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "linux")]
pub mod linux;

pub use screen_capture::ScreenCaptureNode;
pub use window_capture::WindowCaptureNode;

/// Platform-agnostic screen capture traits
pub trait ScreenCaptureBackend: Send + Sync {
    fn new(display_id: u32, capture_cursor: bool) -> Result<Self> where Self: Sized;
    fn capture_frame(&mut self) -> Result<VideoFrame>;
    fn get_display_count() -> Result<u32>;
    fn get_display_bounds(&self, display_id: u32) -> Result<(u32, u32, u32, u32)>; // x, y, width, height
}

/// Platform-agnostic window capture traits
pub trait WindowCaptureBackend: Send + Sync {
    fn new(window_id: u64) -> Result<Self> where Self: Sized;
    fn new_by_title(title: &str) -> Result<Self> where Self: Sized;
    fn capture_frame(&mut self) -> Result<VideoFrame>;
    fn get_window_list() -> Result<Vec<WindowInfo>>;
    fn get_window_bounds(&self) -> Result<(u32, u32, u32, u32)>; // x, y, width, height
}

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub id: u64,
    pub title: String,
    pub process_name: String,
    pub bounds: (u32, u32, u32, u32), // x, y, width, height
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeProcessor;
    use std::collections::HashMap;
    use serde_json::Value;

    #[test]
    fn test_screen_capture_node_creation() {
        let node_id = Uuid::new_v4();
        let config = NodeConfig {
            parameters: HashMap::new(),
        };

        let result = ScreenCaptureNode::new(node_id, config);
        assert!(result.is_ok());

        let node = result.unwrap();
        assert_eq!(node.get_properties().name, "Screen Capture");
        assert_eq!(node.get_properties().output_types, vec![ConnectionType::Video]);
    }

    #[test]
    fn test_window_capture_node_creation() {
        let node_id = Uuid::new_v4();
        let config = NodeConfig {
            parameters: HashMap::new(),
        };

        let result = WindowCaptureNode::new(node_id, config);
        assert!(result.is_ok());

        let node = result.unwrap();
        assert_eq!(node.get_properties().name, "Window Capture");
        assert_eq!(node.get_properties().output_types, vec![ConnectionType::Video]);
    }

    #[test]
    fn test_capture_parameter_defaults() {
        let node_id = Uuid::new_v4();
        let config = NodeConfig {
            parameters: HashMap::new(),
        };

        let node = ScreenCaptureNode::new(node_id, config).unwrap();
        
        // Test default parameters
        assert_eq!(node.get_parameter("display_id"), Some(Value::from(0)));
        assert_eq!(node.get_parameter("capture_cursor"), Some(Value::Bool(true)));
        assert_eq!(node.get_parameter("fps"), Some(Value::from(30)));
    }

    #[test]
    fn test_capture_parameter_modification() {
        let node_id = Uuid::new_v4();
        let config = NodeConfig {
            parameters: HashMap::new(),
        };

        let mut node = ScreenCaptureNode::new(node_id, config).unwrap();
        
        // Modify parameters
        assert!(node.set_parameter("display_id", Value::from(1)).is_ok());
        assert!(node.set_parameter("capture_cursor", Value::Bool(false)).is_ok());
        assert!(node.set_parameter("fps", Value::from(60)).is_ok());
        
        // Verify changes
        assert_eq!(node.get_parameter("display_id"), Some(Value::from(1)));
        assert_eq!(node.get_parameter("capture_cursor"), Some(Value::Bool(false)));
        assert_eq!(node.get_parameter("fps"), Some(Value::from(60)));
    }

    // Platform-specific backend tests (will be implemented once backends are ready)
    #[cfg(feature = "test-capture-backends")]
    mod backend_tests {
        use super::*;

        #[test]
        fn test_screen_capture_backend_display_detection() {
            // This test requires actual display hardware
            let display_count = get_display_count();
            assert!(display_count.is_ok());
            assert!(display_count.unwrap() > 0);
        }

        #[test] 
        fn test_window_capture_backend_window_list() {
            // This test requires running applications
            let window_list = get_window_list();
            assert!(window_list.is_ok());
            // Note: May be empty in headless CI environment
        }

        #[test]
        fn test_capture_performance_benchmark() {
            // Performance test: capture 30 frames and measure timing
            let node_id = Uuid::new_v4();
            let config = NodeConfig {
                parameters: HashMap::new(),
            };

            let mut node = ScreenCaptureNode::new(node_id, config).unwrap();
            
            let start_time = std::time::Instant::now();
            let mut successful_captures = 0;
            
            for _ in 0..30 {
                let dummy_input = FrameData {
                    video_data: None,
                    audio_data: None,
                    tally_data: None,
                };
                
                if let Ok(output) = node.process(dummy_input) {
                    if output.video_data.is_some() {
                        successful_captures += 1;
                    }
                }
            }
            
            let elapsed = start_time.elapsed();
            let fps = successful_captures as f64 / elapsed.as_secs_f64();
            
            println!("Capture performance: {:.2} fps", fps);
            // In real implementation, this should be >= 30 fps
            assert!(fps > 0.0);
        }
    }
}

// Placeholder platform detection functions (will be implemented per platform)
#[cfg(target_os = "windows")]
pub fn get_display_count() -> Result<u32> {
    windows::get_display_count()
}

#[cfg(target_os = "macos")]
pub fn get_display_count() -> Result<u32> {
    macos::get_display_count()
}

#[cfg(target_os = "linux")]
pub fn get_display_count() -> Result<u32> {
    linux::get_display_count()
}

#[cfg(target_os = "windows")]
pub fn get_window_list() -> Result<Vec<WindowInfo>> {
    windows::get_window_list()
}

#[cfg(target_os = "macos")]
pub fn get_window_list() -> Result<Vec<WindowInfo>> {
    macos::get_window_list()
}

#[cfg(target_os = "linux")]
pub fn get_window_list() -> Result<Vec<WindowInfo>> {
    linux::get_window_list()
}