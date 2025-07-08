use super::{VideoFormat, VirtualWebcamBackend};
use anyhow::{anyhow, Result};
use constellation_core::VideoFrame;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Linux virtual webcam implementation using V4L2 loopback
pub struct LinuxVirtualWebcam {
    device_name: String,
    width: u32,
    height: u32,
    fps: u32,
    format: VideoFormat,
    is_active: Arc<AtomicBool>,
    device_path: Option<String>,
    device_file: Option<File>,
}

impl VirtualWebcamBackend for LinuxVirtualWebcam {
    fn new(device_name: String, width: u32, height: u32, fps: u32) -> Result<Self> {
        Ok(Self {
            device_name,
            width,
            height,
            fps,
            format: VideoFormat::YUV420, // V4L2 commonly uses YUV formats
            is_active: Arc::new(AtomicBool::new(false)),
            device_path: None,
            device_file: None,
        })
    }

    fn start(&mut self) -> Result<()> {
        if self.is_active.load(Ordering::Relaxed) {
            return Ok(());
        }

        // Find or create V4L2 loopback device
        let device_path = self.find_or_create_loopback_device()?;

        // Configure V4L2 device
        self.configure_v4l2_device(&device_path)?;

        // Open device for writing
        let device_file = OpenOptions::new()
            .write(true)
            .mode(0o666)
            .open(&device_path)?;

        self.device_path = Some(device_path);
        self.device_file = Some(device_file);
        self.is_active.store(true, Ordering::Relaxed);

        tracing::info!(
            "Started Linux virtual webcam: {} ({}x{}@{}fps) at {:?}",
            self.device_name,
            self.width,
            self.height,
            self.fps,
            self.device_path
        );

        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        if !self.is_active.load(Ordering::Relaxed) {
            return Ok(());
        }

        // Close device file
        self.device_file = None;

        // Clean up device configuration if needed
        self.cleanup_v4l2_device()?;

        self.is_active.store(false, Ordering::Relaxed);

        tracing::info!("Stopped Linux virtual webcam: {}", self.device_name);
        Ok(())
    }

    fn send_frame(&mut self, frame: &VideoFrame) -> Result<()> {
        if !self.is_active.load(Ordering::Relaxed) {
            return Err(anyhow!("Virtual webcam is not active"));
        }

        let device_file = self
            .device_file
            .as_mut()
            .ok_or_else(|| anyhow!("Device file not opened"))?;

        // Convert frame to V4L2 format and write to device
        let converted_frame = self.convert_frame_for_v4l2(frame)?;
        device_file.write_all(&converted_frame)?;
        device_file.flush()?;

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

impl LinuxVirtualWebcam {
    /// Find existing or create new V4L2 loopback device
    fn find_or_create_loopback_device(&self) -> Result<String> {
        // Check if v4l2loopback is loaded
        if !self.is_v4l2loopback_available()? {
            return Err(anyhow!(
                "v4l2loopback kernel module not found. Please install and load v4l2loopback:\n\
                sudo modprobe v4l2loopback devices=1 video_nr=10 card_label=\"{}\"\n\
                Or install: sudo apt-get install v4l2loopback-dkms",
                self.device_name
            ));
        }

        // Look for available V4L2 loopback devices
        for i in 0..32 {
            let device_path = format!("/dev/video{i}");
            if self.is_loopback_device(&device_path)? {
                tracing::debug!("Found V4L2 loopback device: {device_path}");
                return Ok(device_path);
            }
        }

        Err(anyhow!(
            "No V4L2 loopback devices found. Create one with:\n\
            sudo modprobe v4l2loopback devices=1 video_nr=10 card_label=\"{}\"\n\
            Then device will be available at /dev/video10",
            self.device_name
        ))
    }

    /// Check if v4l2loopback kernel module is available
    fn is_v4l2loopback_available(&self) -> Result<bool> {
        // Check /proc/modules for v4l2loopback
        if let Ok(modules) = std::fs::read_to_string("/proc/modules") {
            Ok(modules.contains("v4l2loopback"))
        } else {
            // Fallback: check if any loopback devices exist
            Ok(Path::new("/sys/devices/virtual/video4linux").exists())
        }
    }

    /// Check if a device is a V4L2 loopback device
    fn is_loopback_device(&self, device_path: &str) -> Result<bool> {
        // Extract device number from path
        let device_num = device_path
            .trim_start_matches("/dev/video")
            .parse::<u32>()
            .map_err(|_| anyhow!("Invalid device path format: {}", device_path))?;

        // Check if device exists and is a loopback device
        let sys_path = format!("/sys/class/video4linux/video{device_num}/name");
        if let Ok(name) = std::fs::read_to_string(&sys_path) {
            // V4L2 loopback devices typically have "Dummy video device" or custom names
            Ok(name.contains("Dummy")
                || name.contains("loopback")
                || name.trim() == self.device_name)
        } else {
            Ok(false)
        }
    }

    /// Configure V4L2 device format and parameters
    fn configure_v4l2_device(&self, device_path: &str) -> Result<()> {
        // This would use V4L2 ioctls to configure:
        // - Video format (VIDIOC_S_FMT)
        // - Frame rate (VIDIOC_S_PARM)
        // - Buffer settings

        tracing::debug!(
            "Configuring V4L2 device {} for {}x{}@{}fps",
            device_path,
            self.width,
            self.height,
            self.fps
        );

        // For now, we'll use v4l2-ctl command line tool as fallback
        self.configure_with_v4l2_ctl(device_path)
    }

    /// Configure device using v4l2-ctl command line tool
    fn configure_with_v4l2_ctl(&self, device_path: &str) -> Result<()> {
        use std::process::Command;

        // Set video format
        let format_cmd = Command::new("v4l2-ctl")
            .args(&[
                "--device",
                device_path,
                "--set-fmt-video",
                &format!(
                    "width={},height={},pixelformat=YU12",
                    self.width, self.height
                ),
            ])
            .output();

        match format_cmd {
            Ok(output) => {
                if !output.status.success() {
                    tracing::warn!(
                        "v4l2-ctl format configuration failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            }
            Err(_) => {
                tracing::warn!("v4l2-ctl not found, using default device configuration");
            }
        }

        // Set frame rate
        let fps_cmd = Command::new("v4l2-ctl")
            .args(&[
                "--device",
                device_path,
                "--set-parm",
                &format!("{}", self.fps),
            ])
            .output();

        match fps_cmd {
            Ok(output) => {
                if !output.status.success() {
                    tracing::warn!(
                        "v4l2-ctl framerate configuration failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            }
            Err(_) => {
                // v4l2-ctl not available, continue without it
            }
        }

        Ok(())
    }

    /// Clean up V4L2 device configuration
    fn cleanup_v4l2_device(&self) -> Result<()> {
        // Any cleanup needed when stopping
        tracing::debug!("Cleaning up V4L2 device configuration");
        Ok(())
    }

    /// Convert VideoFrame to V4L2-compatible format
    fn convert_frame_for_v4l2(&self, frame: &VideoFrame) -> Result<Vec<u8>> {
        // Convert frame data to YUV420 format for V4L2
        // This is a simplified implementation

        let expected_size = (self.width * self.height * 3 / 2) as usize;
        let mut yuv_data = vec![0u8; expected_size];

        // Placeholder conversion - in practice would implement proper RGB->YUV conversion
        // For now, create a test pattern
        self.create_test_pattern(&mut yuv_data);

        Ok(yuv_data)
    }

    /// Create test pattern for debugging
    fn create_test_pattern(&self, buffer: &mut [u8]) {
        let y_size = (self.width * self.height) as usize;
        let uv_size = y_size / 4;

        // Y plane (luminance) - create gradient
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y * self.width + x) as usize;
                if idx < y_size {
                    buffer[idx] = ((x + y) % 256) as u8;
                }
            }
        }

        // U plane (chroma)
        for i in 0..uv_size {
            if y_size + i < buffer.len() {
                buffer[y_size + i] = 128; // Neutral chroma
            }
        }

        // V plane (chroma)
        for i in 0..uv_size {
            if y_size + uv_size + i < buffer.len() {
                buffer[y_size + uv_size + i] = 128; // Neutral chroma
            }
        }
    }
}

impl Drop for LinuxVirtualWebcam {
    fn drop(&mut self) {
        if self.is_active.load(Ordering::Relaxed) {
            if let Err(e) = self.stop() {
                tracing::error!("Failed to stop Linux virtual webcam on drop: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_virtual_webcam_creation() {
        let webcam = LinuxVirtualWebcam::new("Test Camera".to_string(), 1280, 720, 30);

        assert!(webcam.is_ok());
        let webcam = webcam.unwrap();
        assert_eq!(webcam.get_device_name(), "Test Camera");
        assert!(!webcam.is_active());
    }

    #[test]
    fn test_resolution_change() {
        let mut webcam =
            LinuxVirtualWebcam::new("Test Camera".to_string(), 1920, 1080, 30).unwrap();

        // Should succeed when not active
        assert!(webcam.set_resolution(1280, 720).is_ok());
        assert_eq!(webcam.width, 1280);
        assert_eq!(webcam.height, 720);
    }

    #[test]
    fn test_fps_change() {
        let mut webcam =
            LinuxVirtualWebcam::new("Test Camera".to_string(), 1920, 1080, 30).unwrap();

        // Should succeed when not active
        assert!(webcam.set_fps(60).is_ok());
        assert_eq!(webcam.fps, 60);
    }

    #[test]
    fn test_frame_conversion() {
        let webcam = LinuxVirtualWebcam::new("Test Camera".to_string(), 640, 480, 30).unwrap();

        let frame = VideoFrame {
            width: 640,
            height: 480,
            data: vec![0u8; 640 * 480 * 3], // RGB data
            format: constellation_core::VideoFormat::Rgb8,
        };

        let converted = webcam.convert_frame_for_v4l2(&frame);
        assert!(converted.is_ok());

        let yuv_data = converted.unwrap();
        // YUV420 should be 1.5x the pixel count
        assert_eq!(yuv_data.len(), 640 * 480 * 3 / 2);
    }
}
