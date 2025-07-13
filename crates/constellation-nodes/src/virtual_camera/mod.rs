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

use anyhow::Result;
use constellation_core::VideoFrame;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;

/// Virtual webcam backend trait for cross-platform implementation
pub trait VirtualWebcamBackend: Send + Sync {
    /// Create a new virtual webcam instance
    fn new(device_name: String, width: u32, height: u32, fps: u32) -> Result<Self>
    where
        Self: Sized;

    /// Start the virtual webcam service
    fn start(&mut self) -> Result<()>;

    /// Stop the virtual webcam service
    fn stop(&mut self) -> Result<()>;

    /// Send a video frame to the virtual webcam
    fn send_frame(&mut self, frame: &VideoFrame) -> Result<()>;

    /// Check if the virtual webcam is active
    fn is_active(&self) -> bool;

    /// Get the current device name
    fn get_device_name(&self) -> &str;

    /// Update resolution (if supported)
    fn set_resolution(&mut self, width: u32, height: u32) -> Result<()>;

    /// Update frame rate (if supported)
    fn set_fps(&mut self, fps: u32) -> Result<()>;
}

/// Virtual webcam configuration
#[derive(Debug, Clone)]
pub struct VirtualWebcamConfig {
    pub device_name: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub format: VideoFormat,
}

impl Default for VirtualWebcamConfig {
    fn default() -> Self {
        Self {
            device_name: "Constellation Studio".to_string(),
            width: 1920,
            height: 1080,
            fps: 30,
            format: VideoFormat::RGB24,
        }
    }
}

/// Supported video formats for virtual webcam output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoFormat {
    RGB24,
    BGRA32,
    YUV420,
    NV12,
}

impl VideoFormat {
    /// Returns bytes per pixel for packed formats.
    /// Note: For planar formats (YUV420, NV12), use frame_size() instead.
    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            VideoFormat::RGB24 => 3,
            VideoFormat::BGRA32 => 4,
            VideoFormat::YUV420 => 1, // Planar format - misleading, use frame_size()
            VideoFormat::NV12 => 1,   // Planar format - misleading, use frame_size()
        }
    }

    /// Returns the total frame size in bytes for the given dimensions
    pub fn frame_size(&self, width: u32, height: u32) -> usize {
        match self {
            VideoFormat::RGB24 => (width * height * 3) as usize,
            VideoFormat::BGRA32 => (width * height * 4) as usize,
            VideoFormat::YUV420 | VideoFormat::NV12 => (width * height * 3 / 2) as usize,
        }
    }

    pub fn stride(&self, width: u32) -> u32 {
        match self {
            VideoFormat::RGB24 => width * 3,
            VideoFormat::BGRA32 => width * 4,
            VideoFormat::YUV420 => width,
            VideoFormat::NV12 => width,
        }
    }
}

/// Platform-specific virtual webcam instance type
#[cfg(target_os = "windows")]
pub type PlatformVirtualWebcam = windows::WindowsVirtualWebcam;

#[cfg(target_os = "macos")]
pub type PlatformVirtualWebcam = macos::MacOSVirtualWebcam;

#[cfg(target_os = "linux")]
pub type PlatformVirtualWebcam = linux::LinuxVirtualWebcam;

/// Cross-platform virtual webcam manager
pub struct VirtualWebcam {
    backend: PlatformVirtualWebcam,
    config: VirtualWebcamConfig,
}

impl VirtualWebcam {
    /// Create a new virtual webcam with default configuration
    pub fn new(device_name: String) -> Result<Self> {
        let config = VirtualWebcamConfig {
            device_name: device_name.clone(),
            ..Default::default()
        };
        Self::new_with_config(config)
    }

    /// Create a new virtual webcam with custom configuration
    pub fn new_with_config(config: VirtualWebcamConfig) -> Result<Self> {
        let backend = PlatformVirtualWebcam::new(
            config.device_name.clone(),
            config.width,
            config.height,
            config.fps,
        )?;

        Ok(Self { backend, config })
    }

    /// Start the virtual webcam
    pub fn start(&mut self) -> Result<()> {
        self.backend.start()
    }

    /// Stop the virtual webcam
    pub fn stop(&mut self) -> Result<()> {
        self.backend.stop()
    }

    /// Send a frame to the virtual webcam
    pub fn send_frame(&mut self, frame: &VideoFrame) -> Result<()> {
        self.backend.send_frame(frame)
    }

    /// Check if the virtual webcam is active
    pub fn is_active(&self) -> bool {
        self.backend.is_active()
    }

    /// Get the current configuration
    pub fn config(&self) -> &VirtualWebcamConfig {
        &self.config
    }

    /// Update resolution dynamically
    pub fn set_resolution(&mut self, width: u32, height: u32) -> Result<()> {
        let result = self.backend.set_resolution(width, height);
        if result.is_ok() {
            self.config.width = width;
            self.config.height = height;
        }
        result
    }

    /// Update frame rate dynamically
    pub fn set_fps(&mut self, fps: u32) -> Result<()> {
        let result = self.backend.set_fps(fps);
        if result.is_ok() {
            self.config.fps = fps;
        }
        result
    }

    /// Get platform information
    pub fn platform_info() -> PlatformInfo {
        PlatformInfo::current()
    }
}

/// Platform information for virtual webcam capabilities
#[derive(Debug, Clone)]
pub struct PlatformInfo {
    pub platform: String,
    pub supports_dynamic_resolution: bool,
    pub supports_dynamic_fps: bool,
    pub preferred_formats: Vec<VideoFormat>,
    pub max_resolution: (u32, u32),
    pub supported_fps: Vec<u32>,
}

impl PlatformInfo {
    /// Get current platform information
    pub fn current() -> Self {
        #[cfg(target_os = "windows")]
        {
            Self {
                platform: "Windows".to_string(),
                supports_dynamic_resolution: false, // DirectShow limitation
                supports_dynamic_fps: false,
                preferred_formats: vec![VideoFormat::RGB24, VideoFormat::BGRA32],
                max_resolution: (3840, 2160), // 4K
                supported_fps: vec![15, 24, 30, 60],
            }
        }

        #[cfg(target_os = "macos")]
        {
            Self {
                platform: "macOS".to_string(),
                supports_dynamic_resolution: true, // Core Media I/O supports this
                supports_dynamic_fps: true,
                preferred_formats: vec![VideoFormat::BGRA32, VideoFormat::NV12],
                max_resolution: (3840, 2160), // 4K
                supported_fps: vec![15, 24, 30, 60, 120],
            }
        }

        #[cfg(target_os = "linux")]
        {
            Self {
                platform: "Linux".to_string(),
                supports_dynamic_resolution: true, // V4L2 supports this
                supports_dynamic_fps: true,
                preferred_formats: vec![VideoFormat::YUV420, VideoFormat::RGB24],
                max_resolution: (1920, 1080), // Limited by V4L2 loopback
                supported_fps: vec![15, 30, 60],
            }
        }
    }

    /// Check if a resolution is supported
    pub fn supports_resolution(&self, width: u32, height: u32) -> bool {
        width <= self.max_resolution.0 && height <= self.max_resolution.1
    }

    /// Check if a frame rate is supported
    pub fn supports_fps(&self, fps: u32) -> bool {
        self.supported_fps.contains(&fps)
    }

    /// Get the closest supported frame rate
    pub fn closest_fps(&self, target_fps: u32) -> u32 {
        self.supported_fps
            .iter()
            .min_by_key(|&&fps| (fps as i32 - target_fps as i32).abs())
            .copied()
            .unwrap_or(30)
    }
}

/// Frame conversion utilities
pub mod conversion {
    use super::*;

    /// Convert VideoFrame to the specified format for virtual webcam
    pub fn convert_frame(frame: &VideoFrame, target_format: VideoFormat) -> Result<Vec<u8>> {
        // Simplified conversion for now - in practice would use proper color space conversion
        match target_format {
            VideoFormat::RGB24 => convert_to_rgb24(frame),
            VideoFormat::BGRA32 => convert_to_bgra32(frame),
            VideoFormat::YUV420 => convert_to_yuv420(frame),
            VideoFormat::NV12 => convert_to_nv12(frame),
        }
    }

    fn convert_to_rgb24(frame: &VideoFrame) -> Result<Vec<u8>> {
        // Placeholder implementation - would implement proper conversion
        let size = (frame.width * frame.height * 3) as usize;
        Ok(vec![0u8; size])
    }

    fn convert_to_bgra32(frame: &VideoFrame) -> Result<Vec<u8>> {
        // Placeholder implementation - would implement proper conversion
        let size = (frame.width * frame.height * 4) as usize;
        Ok(vec![0u8; size])
    }

    fn convert_to_yuv420(frame: &VideoFrame) -> Result<Vec<u8>> {
        // Placeholder implementation - would implement proper conversion
        let size = (frame.width * frame.height * 3 / 2) as usize;
        Ok(vec![0u8; size])
    }

    fn convert_to_nv12(frame: &VideoFrame) -> Result<Vec<u8>> {
        // Placeholder implementation - would implement proper conversion
        let size = (frame.width * frame.height * 3 / 2) as usize;
        Ok(vec![0u8; size])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_format_properties() {
        assert_eq!(VideoFormat::RGB24.bytes_per_pixel(), 3);
        assert_eq!(VideoFormat::BGRA32.bytes_per_pixel(), 4);
        assert_eq!(VideoFormat::RGB24.stride(1920), 5760);
        assert_eq!(VideoFormat::BGRA32.stride(1920), 7680);
    }

    #[test]
    fn test_default_config() {
        let config = VirtualWebcamConfig::default();
        assert_eq!(config.device_name, "Constellation Studio");
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert_eq!(config.fps, 30);
        assert_eq!(config.format, VideoFormat::RGB24);
    }

    #[test]
    fn test_platform_info() {
        let info = PlatformInfo::current();
        
        // All platforms should support at least 1080p
        assert!(info.supports_resolution(1920, 1080));
        
        // All platforms should support 30fps
        assert!(info.supports_fps(30));
        
        // Platform name should not be empty
        assert!(!info.platform.is_empty());
        
        // Should have at least one preferred format
        assert!(!info.preferred_formats.is_empty());
    }

    #[test]
    fn test_platform_fps_selection() {
        let info = PlatformInfo::current();
        
        // Test closest FPS selection
        assert_eq!(info.closest_fps(29), 30);
        assert_eq!(info.closest_fps(31), 30);
        
        // Test exact match
        if info.supports_fps(60) {
            assert_eq!(info.closest_fps(60), 60);
        }
    }

    #[test]
    fn test_virtual_webcam_creation() {
        let webcam = VirtualWebcam::new("Test Camera".to_string());
        assert!(webcam.is_ok());
        
        let webcam = webcam.unwrap();
        assert_eq!(webcam.config().device_name, "Test Camera");
        assert!(!webcam.is_active());
    }

    #[test]
    fn test_virtual_webcam_with_custom_config() {
        let config = VirtualWebcamConfig {
            device_name: "Custom Camera".to_string(),
            width: 1280,
            height: 720,
            fps: 60,
            format: VideoFormat::BGRA32,
        };
        
        let webcam = VirtualWebcam::new_with_config(config);
        assert!(webcam.is_ok());
        
        let webcam = webcam.unwrap();
        assert_eq!(webcam.config().width, 1280);
        assert_eq!(webcam.config().height, 720);
        assert_eq!(webcam.config().fps, 60);
    }
}
