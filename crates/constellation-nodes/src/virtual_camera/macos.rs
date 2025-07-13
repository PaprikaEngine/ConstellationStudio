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

use super::{VideoFormat, VirtualWebcamBackend};
use anyhow::{anyhow, Result};
use constellation_core::VideoFrame;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// Phase 1: Simplified Core Media bindings
// Full Core Media I/O Extensions integration will be implemented in Phase 2

/// macOS virtual webcam implementation using Core Media I/O Extensions
/// Phase 1: Basic implementation with frame buffering
/// Phase 2: Full Core Media I/O Extensions integration
pub struct MacOSVirtualWebcam {
    device_name: String,
    width: u32,
    height: u32,
    fps: u32,
    format: VideoFormat,
    is_active: Arc<AtomicBool>,
    device_id: Option<String>,
    // Phase 1: Simple frame counter for timing
    frame_count: u64,
}

impl VirtualWebcamBackend for MacOSVirtualWebcam {
    fn new(device_name: String, width: u32, height: u32, fps: u32) -> Result<Self> {
        Ok(Self {
            device_name,
            width,
            height,
            fps,
            format: VideoFormat::BGRA32, // macOS prefers BGRA
            is_active: Arc::new(AtomicBool::new(false)),
            device_id: None,
            frame_count: 0,
        })
    }

    fn start(&mut self) -> Result<()> {
        if self.is_active.load(Ordering::Relaxed) {
            return Ok(());
        }

        // Create virtual camera device using Core Media I/O Extensions
        self.create_virtual_device()?;
        self.is_active.store(true, Ordering::Relaxed);

        tracing::info!(
            "Started macOS virtual webcam: {} ({}x{}@{}fps)",
            self.device_name,
            self.width,
            self.height,
            self.fps
        );

        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        if !self.is_active.load(Ordering::Relaxed) {
            return Ok(());
        }

        self.destroy_virtual_device()?;
        self.is_active.store(false, Ordering::Relaxed);

        tracing::info!("Stopped macOS virtual webcam: {}", self.device_name);
        Ok(())
    }

    fn send_frame(&mut self, frame: &VideoFrame) -> Result<()> {
        if !self.is_active.load(Ordering::Relaxed) {
            return Err(anyhow!("Virtual webcam is not active"));
        }

        // Process frame and simulate sending to virtual device
        let processed_data = self.process_frame(frame)?;
        self.send_processed_frame(processed_data)?;

        Ok(())
    }

    fn is_active(&self) -> bool {
        self.is_active.load(Ordering::Relaxed)
    }

    fn get_device_name(&self) -> &str {
        &self.device_name
    }

    fn set_resolution(&mut self, width: u32, height: u32) -> Result<()> {
        if self.is_active.load(Ordering::Relaxed) {
            return Err(anyhow!("Cannot change resolution while active"));
        }

        self.width = width;
        self.height = height;
        Ok(())
    }

    fn set_fps(&mut self, fps: u32) -> Result<()> {
        if self.is_active.load(Ordering::Relaxed) {
            return Err(anyhow!("Cannot change FPS while active"));
        }

        self.fps = fps;
        Ok(())
    }
}

impl MacOSVirtualWebcam {
    /// Create Core Media I/O virtual camera device
    /// Phase 1: Basic device simulation
    fn create_virtual_device(&mut self) -> Result<()> {
        tracing::debug!("Creating Core Media I/O virtual device (Phase 1)");

        // Generate a unique device identifier
        let uuid = uuid::Uuid::new_v4();
        let device_id = format!("constellation-{uuid}");
        self.device_id = Some(device_id);

        // Reset frame counter
        self.frame_count = 0;

        tracing::info!(
            "Created virtual camera device: {} ({}x{}@{}fps)",
            self.device_name,
            self.width,
            self.height,
            self.fps
        );

        // Phase 1: Log device creation for development
        // Phase 2 will implement:
        // - CMIOExtensionDevice creation
        // - System registration
        // - Stream configuration
        // - App compatibility layer

        Ok(())
    }

    /// Destroy virtual camera device
    /// Phase 1: Basic cleanup
    fn destroy_virtual_device(&mut self) -> Result<()> {
        if let Some(device_id) = &self.device_id {
            tracing::debug!("Destroying Core Media I/O virtual device: {}", device_id);

            // Phase 1: Simple cleanup
            self.device_id = None;
            self.frame_count = 0;

            // Phase 2 will implement:
            // - Stop active streams
            // - Unregister device from system
            // - Release Core Media resources

            tracing::info!("Virtual camera device destroyed");
        }

        Ok(())
    }

    /// Process VideoFrame for virtual webcam output
    /// Phase 1: Basic frame processing and logging
    fn process_frame(&mut self, frame: &VideoFrame) -> Result<Vec<u8>> {
        // Convert frame data if necessary
        let processed_data = if frame.format == constellation_core::VideoFormat::Rgba8 {
            self.convert_rgba_to_bgra(&frame.data)
        } else {
            frame.data.clone()
        };

        // Increment frame counter for timing reference
        self.frame_count += 1;

        tracing::debug!(
            "Processed frame {}: {}x{} format={:?} size={}",
            self.frame_count,
            frame.width,
            frame.height,
            frame.format,
            processed_data.len()
        );

        // Phase 1: Return processed frame data
        // Phase 2 will implement actual CMSampleBuffer creation
        Ok(processed_data)
    }

    /// Convert RGBA to BGRA format for macOS compatibility
    fn convert_rgba_to_bgra(&self, rgba_data: &[u8]) -> Vec<u8> {
        let mut bgra_data = Vec::with_capacity(rgba_data.len());

        for chunk in rgba_data.chunks_exact(4) {
            if chunk.len() == 4 {
                // RGBA -> BGRA: swap R and B channels
                bgra_data.push(chunk[2]); // B
                bgra_data.push(chunk[1]); // G
                bgra_data.push(chunk[0]); // R
                bgra_data.push(chunk[3]); // A
            }
        }

        bgra_data
    }

    /// Send processed frame data to virtual device
    /// Phase 1: Simulate frame delivery to virtual webcam
    fn send_processed_frame(&self, frame_data: Vec<u8>) -> Result<()> {
        if let Some(device_id) = &self.device_id {
            tracing::debug!(
                "Sending {} bytes to virtual device: {}",
                frame_data.len(),
                device_id
            );

            // Phase 1: Simulate successful frame delivery
            // Log frame delivery for development and testing
            tracing::trace!(
                "Frame {} sent successfully to virtual webcam ({}x{}@{}fps)",
                self.frame_count,
                self.width,
                self.height,
                self.fps
            );

            // Phase 2 will implement:
            // - CMSampleBuffer creation
            // - CMIOExtensionDevice frame delivery
            // - Timestamp synchronization
            // - Format conversion optimization
        } else {
            return Err(anyhow!("No virtual device available"));
        }

        Ok(())
    }
}

impl Drop for MacOSVirtualWebcam {
    fn drop(&mut self) {
        if self.is_active.load(Ordering::Relaxed) {
            if let Err(e) = self.stop() {
                tracing::error!("Failed to stop macOS virtual webcam on drop: {}", e);
            }
        }
    }
}

// Phase 2 implementation will include:
// - Full Core Media I/O Extensions integration
// - CMIOExtensionDevice creation and management
// - CMSampleBuffer creation with proper timing
// - System-level virtual camera registration
// - App compatibility layer for Zoom/Teams/OBS

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macos_virtual_webcam_creation() {
        let webcam = MacOSVirtualWebcam::new("Test Camera".to_string(), 1280, 720, 30);

        assert!(webcam.is_ok());
        let webcam = webcam.unwrap();
        assert_eq!(webcam.get_device_name(), "Test Camera");
        assert!(!webcam.is_active());
    }

    #[test]
    fn test_resolution_change() {
        let mut webcam =
            MacOSVirtualWebcam::new("Test Camera".to_string(), 1920, 1080, 30).unwrap();

        // Should succeed when not active
        assert!(webcam.set_resolution(1280, 720).is_ok());
        assert_eq!(webcam.width, 1280);
        assert_eq!(webcam.height, 720);
    }

    #[test]
    fn test_fps_change() {
        let mut webcam =
            MacOSVirtualWebcam::new("Test Camera".to_string(), 1920, 1080, 30).unwrap();

        // Should succeed when not active
        assert!(webcam.set_fps(60).is_ok());
        assert_eq!(webcam.fps, 60);
    }
}
