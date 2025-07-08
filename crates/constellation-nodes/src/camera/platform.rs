use anyhow::Result;
use constellation_core::VideoFrame;
use std::collections::HashMap;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

/// Platform-specific camera interface
pub trait PlatformCamera: Send + Sync {
    fn new(device_index: u32, width: u32, height: u32, fps: u32) -> Result<Self>
    where
        Self: Sized;

    fn start(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn capture_frame(&mut self) -> Result<VideoFrame>;
    fn list_devices() -> Result<Vec<CameraDeviceInfo>>;
    fn set_parameters(&mut self, params: &HashMap<String, serde_json::Value>) -> Result<()>;
    fn is_active(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct CameraDeviceInfo {
    pub index: u32,
    pub name: String,
    pub description: String,
    pub vendor_id: Option<String>,
    pub product_id: Option<String>,
}

// Platform-specific camera implementation
#[cfg(target_os = "windows")]
pub type PlatformCameraImpl = windows::WindowsCamera;

#[cfg(target_os = "macos")]
pub type PlatformCameraImpl = macos::MacOSCamera;

#[cfg(target_os = "linux")]
pub type PlatformCameraImpl = linux::LinuxCamera;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_camera_creation() {
        let result = PlatformCameraImpl::new(0, 640, 480, 30);

        // This test may fail in CI environments without cameras
        match result {
            Ok(_) => println!("Platform camera created successfully"),
            Err(e) => println!("Failed to create platform camera (expected in CI): {e}"),
        }
    }

    #[test]
    fn test_list_platform_devices() {
        let result = PlatformCameraImpl::list_devices();

        match result {
            Ok(devices) => {
                println!("Found {} platform camera devices", devices.len());
                for device in devices {
                    println!("  Device {}: {}", device.index, device.name);
                }
            }
            Err(e) => {
                println!(
                    "Failed to list platform camera devices (expected in CI): {}",
                    e
                );
            }
        }
    }
}
