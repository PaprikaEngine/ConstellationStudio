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
}
