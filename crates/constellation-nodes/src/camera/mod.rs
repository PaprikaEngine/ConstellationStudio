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
use constellation_core::{VideoFormat, VideoFrame};
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{ApiBackend, CameraIndex, RequestedFormat, RequestedFormatType, Resolution};
use nokhwa::Camera;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

pub mod platform;

pub struct CameraCapture {
    camera: Option<Camera>,
    is_running: bool,
    device_index: CameraIndex,
    width: u32,
    height: u32,
    fps: u32,
    format: VideoFormat,
    frame_sender: Option<mpsc::UnboundedSender<VideoFrame>>,
    frame_receiver: Option<mpsc::UnboundedReceiver<VideoFrame>>,
}

impl CameraCapture {
    pub fn new(device_index: u32, width: u32, height: u32, fps: u32) -> Result<Self> {
        let device_index = if device_index == 0 {
            CameraIndex::Index(0)
        } else {
            CameraIndex::Index(device_index)
        };

        let (frame_sender, frame_receiver) = mpsc::unbounded_channel();

        Ok(Self {
            camera: None,
            is_running: false,
            device_index,
            width,
            height,
            fps,
            format: VideoFormat::Rgba8,
            frame_sender: Some(frame_sender),
            frame_receiver: Some(frame_receiver),
        })
    }

    pub fn list_devices() -> Result<Vec<CameraDevice>> {
        let devices = nokhwa::query(ApiBackend::Auto)?;

        let camera_devices = devices
            .into_iter()
            .map(|info| CameraDevice {
                index: match info.index() {
                    CameraIndex::Index(i) => *i,
                    CameraIndex::String(s) => {
                        match s.parse() {
                            Ok(index) => index,
                            Err(_) => {
                                warn!(
                                    "Non-numeric camera index '{}', assigning hash-based index",
                                    s
                                );
                                // Use hash of string to avoid conflicts
                                use std::collections::hash_map::DefaultHasher;
                                use std::hash::{Hash, Hasher};
                                let mut hasher = DefaultHasher::new();
                                s.hash(&mut hasher);
                                (hasher.finish() % 1000) as u32 // Keep it reasonable
                            }
                        }
                    }
                },
                name: info.human_name(),
                description: info.description().to_string(),
            })
            .collect();

        Ok(camera_devices)
    }

    pub fn start_capture(&mut self) -> Result<()> {
        if self.is_running {
            return Ok(());
        }

        info!(
            "Starting camera capture: device={:?}, resolution={}x{}, fps={}",
            self.device_index, self.width, self.height, self.fps
        );

        let requested_format =
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);

        let mut camera = Camera::new(self.device_index.clone(), requested_format)?;

        // Set camera resolution and frame rate
        let resolution = Resolution::new(self.width, self.height);
        camera.set_resolution(resolution)?;
        camera.set_frame_rate(self.fps)?;

        // Open camera stream
        camera.open_stream()?;

        self.camera = Some(camera);
        self.is_running = true;

        info!("Camera capture started successfully");
        Ok(())
    }

    pub fn stop_capture(&mut self) -> Result<()> {
        if !self.is_running {
            return Ok(());
        }

        info!("Stopping camera capture");

        if let Some(mut camera) = self.camera.take() {
            camera.stop_stream()?;
        }

        self.is_running = false;
        info!("Camera capture stopped");
        Ok(())
    }

    pub fn capture_frame(&mut self) -> Result<VideoFrame> {
        if !self.is_running {
            return Err(anyhow::anyhow!("Camera capture not started"));
        }

        let camera = self
            .camera
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Camera not initialized"))?;

        let frame = camera.frame()?;
        let buffer = frame.buffer_bytes();

        // Convert to our VideoFrame format
        let video_frame = VideoFrame {
            width: frame.resolution().width_x,
            height: frame.resolution().height_y,
            format: self.format.clone(),
            data: buffer.to_vec(),
        };

        debug!(
            "Captured frame: {}x{}, {} bytes",
            video_frame.width,
            video_frame.height,
            video_frame.data.len()
        );

        Ok(video_frame)
    }

    pub fn get_capabilities(&self) -> Result<CameraCapabilities> {
        if let Some(camera) = &self.camera {
            let _camera_format = camera.camera_format();
            Ok(CameraCapabilities {
                resolutions: vec![(self.width, self.height)], // Simplified for now
                frame_rates: vec![self.fps],
                formats: vec![self.format.clone()],
            })
        } else {
            Err(anyhow::anyhow!("Camera not initialized"))
        }
    }

    pub fn set_parameters(
        &mut self,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        if let Some(width) = parameters.get("width").and_then(|v| v.as_u64()) {
            self.width = width as u32;
        }

        if let Some(height) = parameters.get("height").and_then(|v| v.as_u64()) {
            self.height = height as u32;
        }

        if let Some(fps) = parameters.get("fps").and_then(|v| v.as_u64()) {
            self.fps = fps as u32;
        }

        if let Some(device_index) = parameters.get("device_index").and_then(|v| v.as_u64()) {
            self.device_index = CameraIndex::Index(device_index as u32);
        }

        // Restart capture with new parameters if running
        if self.is_running {
            self.stop_capture()?;
            self.start_capture()?;
        }

        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn fps(&self) -> f32 {
        self.fps as f32
    }
}

impl Drop for CameraCapture {
    fn drop(&mut self) {
        if let Err(e) = self.stop_capture() {
            error!("Failed to stop camera capture during drop: {}", e);
        }
    }
}

#[derive(Debug, Clone)]
pub struct CameraDevice {
    pub index: u32,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct CameraCapabilities {
    pub resolutions: Vec<(u32, u32)>,
    pub frame_rates: Vec<u32>,
    pub formats: Vec<VideoFormat>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_capture_creation() {
        let capture = CameraCapture::new(0, 640, 480, 30);
        assert!(capture.is_ok());

        let capture = capture.unwrap();
        assert_eq!(capture.width, 640);
        assert_eq!(capture.height, 480);
        assert_eq!(capture.fps, 30);
        assert!(!capture.is_running());
    }

    #[test]
    fn test_list_camera_devices() {
        let result = CameraCapture::list_devices();

        // This may fail in CI environments without cameras
        match result {
            Ok(devices) => {
                println!("Found {} camera devices", devices.len());
                for device in devices {
                    println!(
                        "  Device {}: {} - {}",
                        device.index, device.name, device.description
                    );
                }
            }
            Err(e) => {
                println!("Failed to list camera devices (expected in CI): {e}");
            }
        }
    }

    #[test]
    fn test_camera_parameters() {
        let mut capture = CameraCapture::new(0, 640, 480, 30).unwrap();

        let mut params = HashMap::new();
        params.insert("width".to_string(), serde_json::Value::from(1920));
        params.insert("height".to_string(), serde_json::Value::from(1080));
        params.insert("fps".to_string(), serde_json::Value::from(60));

        let result = capture.set_parameters(&params);
        assert!(result.is_ok());

        assert_eq!(capture.width, 1920);
        assert_eq!(capture.height, 1080);
        assert_eq!(capture.fps, 60);
    }
}
