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


use super::{CameraDeviceInfo, PlatformCamera};
use anyhow::Result;
use constellation_core::{VideoFormat, VideoFrame};
use std::collections::HashMap;

pub struct WindowsCamera {
    device_index: u32,
    width: u32,
    height: u32,
    fps: u32,
    is_active: bool,
}

impl PlatformCamera for WindowsCamera {
    fn new(device_index: u32, width: u32, height: u32, fps: u32) -> Result<Self> {
        Ok(Self {
            device_index,
            width,
            height,
            fps,
            is_active: false,
        })
    }

    fn start(&mut self) -> Result<()> {
        // TODO: Implement DirectShow/Media Foundation camera capture
        // For now, create a placeholder implementation
        self.is_active = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.is_active = false;
        Ok(())
    }

    fn capture_frame(&mut self) -> Result<VideoFrame> {
        if !self.is_active {
            return Err(anyhow::anyhow!("Camera not active"));
        }

        // TODO: Implement actual frame capture from DirectShow/Media Foundation
        // For now, generate a test pattern
        let frame_size = (self.width * self.height * 4) as usize;
        let mut data = vec![0u8; frame_size];

        // Create a checkerboard test pattern
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = ((y * self.width + x) * 4) as usize;
                let checker = ((x / 32) + (y / 32)) % 2;
                let color = if checker == 0 { 255 } else { 128 };

                data[idx] = color; // R
                data[idx + 1] = color; // G
                data[idx + 2] = color; // B
                data[idx + 3] = 255; // A
            }
        }

        Ok(VideoFrame {
            width: self.width,
            height: self.height,
            format: VideoFormat::Rgba8,
            data,
        })
    }

    fn list_devices() -> Result<Vec<CameraDeviceInfo>> {
        // TODO: Implement actual device enumeration using DirectShow/Media Foundation
        // For now, return placeholder devices
        Ok(vec![
            CameraDeviceInfo {
                index: 0,
                name: "Integrated Camera".to_string(),
                description: "Windows DirectShow Camera".to_string(),
                vendor_id: Some("Microsoft".to_string()),
                product_id: Some("IntegratedCamera".to_string()),
            },
            CameraDeviceInfo {
                index: 1,
                name: "USB Camera".to_string(),
                description: "Generic USB Camera".to_string(),
                vendor_id: Some("Generic".to_string()),
                product_id: Some("USBCamera".to_string()),
            },
        ])
    }

    fn set_parameters(&mut self, params: &HashMap<String, serde_json::Value>) -> Result<()> {
        if let Some(width) = params.get("width").and_then(|v| v.as_u64()) {
            self.width = width as u32;
        }

        if let Some(height) = params.get("height").and_then(|v| v.as_u64()) {
            self.height = height as u32;
        }

        if let Some(fps) = params.get("fps").and_then(|v| v.as_u64()) {
            self.fps = fps as u32;
        }

        Ok(())
    }

    fn is_active(&self) -> bool {
        self.is_active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_camera_creation() {
        let camera = WindowsCamera::new(0, 1920, 1080, 30);
        assert!(camera.is_ok());

        let camera = camera.unwrap();
        assert_eq!(camera.width, 1920);
        assert_eq!(camera.height, 1080);
        assert_eq!(camera.fps, 30);
        assert!(!camera.is_active());
    }

    #[test]
    fn test_windows_camera_lifecycle() {
        let mut camera = WindowsCamera::new(0, 1280, 720, 60).unwrap();

        assert!(!camera.is_active());

        let start_result = camera.start();
        assert!(start_result.is_ok());
        assert!(camera.is_active());

        let frame_result = camera.capture_frame();
        assert!(frame_result.is_ok());

        let frame = frame_result.unwrap();
        assert_eq!(frame.width, 1280);
        assert_eq!(frame.height, 720);
        assert_eq!(frame.data.len(), 1280 * 720 * 4);

        let stop_result = camera.stop();
        assert!(stop_result.is_ok());
        assert!(!camera.is_active());
    }

    #[test]
    fn test_windows_list_devices() {
        let devices = WindowsCamera::list_devices().unwrap();
        assert_eq!(devices.len(), 2);

        assert_eq!(devices[0].index, 0);
        assert_eq!(devices[0].name, "Integrated Camera");

        assert_eq!(devices[1].index, 1);
        assert_eq!(devices[1].name, "USB Camera");
    }
}
