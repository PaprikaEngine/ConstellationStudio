use crate::{NodeProcessor, NodeProperties, ParameterDefinition, ParameterType};
use crate::camera::CameraCapture;
use crate::video_file::VideoFileReader;
use anyhow::Result;
use constellation_core::*;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;
use tracing::{debug, error, info};

pub struct CameraInputNode {
    id: Uuid,
    config: NodeConfig,
    properties: NodeProperties,
    camera_capture: Option<CameraCapture>,
}

impl CameraInputNode {
    pub fn new(id: Uuid, config: NodeConfig) -> Result<Self> {
        let mut parameters = HashMap::new();
        parameters.insert(
            "device_id".to_string(),
            ParameterDefinition {
                name: "Device ID".to_string(),
                parameter_type: ParameterType::String,
                default_value: Value::String("default".to_string()),
                min_value: None,
                max_value: None,
                description: "Camera device identifier".to_string(),
            },
        );
        parameters.insert(
            "resolution".to_string(),
            ParameterDefinition {
                name: "Resolution".to_string(),
                parameter_type: ParameterType::Enum(vec![
                    "1920x1080".to_string(),
                    "1280x720".to_string(),
                    "640x480".to_string(),
                ]),
                default_value: Value::String("1920x1080".to_string()),
                min_value: None,
                max_value: None,
                description: "Camera resolution".to_string(),
            },
        );
        parameters.insert(
            "fps".to_string(),
            ParameterDefinition {
                name: "Frame Rate".to_string(),
                parameter_type: ParameterType::Integer,
                default_value: Value::from(30),
                min_value: Some(Value::from(1)),
                max_value: Some(Value::from(60)),
                description: "Frames per second".to_string(),
            },
        );

        let properties = NodeProperties {
            id,
            name: "Camera Input".to_string(),
            node_type: NodeType::Input(InputType::Camera),
            input_types: vec![],
            output_types: vec![ConnectionType::RenderData, ConnectionType::Audio],
            parameters,
        };

        Ok(Self {
            id,
            config,
            properties,
            camera_capture: None,
        })
    }
}

impl NodeProcessor for CameraInputNode {
    fn process(&mut self, _input: FrameData) -> Result<FrameData> {
        // Initialize camera capture if not already done
        if self.camera_capture.is_none() {
            if let Err(e) = self.initialize_camera() {
                error!("Failed to initialize camera: {}", e);
                // Continue with fallback frame instead of failing
            }
        }

        // Capture frame from camera
        let video_frame = if let Some(ref mut camera) = self.camera_capture {
            if !camera.is_running() {
                match camera.start_capture() {
                    Ok(_) => {
                        info!("Camera capture started successfully");
                    },
                    Err(e) => {
                        error!("Failed to start camera capture: {}", e);
                        return Ok(FrameData {
                            render_data: Some(RenderData::Raster2D(self.create_fallback_frame())),
                            audio_data: Some(UnifiedAudioData::Stereo {
                                sample_rate: 48000,
                                channels: 2,
                                samples: vec![0.0; 1024],
                            }),
                            control_data: None,
                            tally_metadata: TallyMetadata::new(),
                        });
                    }
                }
            }
            
            match camera.capture_frame() {
                Ok(frame) => {
                    debug!("Successfully captured frame: {}x{}", frame.width, frame.height);
                    Some(frame)
                },
                Err(e) => {
                    error!("Failed to capture frame: {}", e);
                    // Return a fallback frame instead of failing
                    Some(self.create_fallback_frame())
                }
            }
        } else {
            error!("Camera capture not initialized, using fallback frame");
            Some(self.create_fallback_frame())
        };

        Ok(FrameData {
            render_data: video_frame.map(RenderData::Raster2D),
            audio_data: Some(UnifiedAudioData::Stereo {
                sample_rate: 48000,
                channels: 2,
                samples: vec![0.0; 1024],
            }),
            control_data: None,
            tally_metadata: TallyMetadata::new(),
        })
    }

    fn get_properties(&self) -> NodeProperties {
        self.properties.clone()
    }

    fn set_parameter(&mut self, key: &str, value: Value) -> Result<()> {
        self.config.parameters.insert(key.to_string(), value);
        // Reset camera capture to apply new parameters
        self.camera_capture = None;
        Ok(())
    }

    fn get_parameter(&self, key: &str) -> Option<Value> {
        self.config.parameters.get(key).cloned()
    }
}

impl CameraInputNode {
    fn initialize_camera(&mut self) -> Result<()> {
        info!("Initializing camera capture");

        // Get parameters from config
        let device_index = self.config.parameters
            .get("device_id")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        let (width, height) = self.parse_resolution()?;
        
        let fps = self.config.parameters
            .get("fps")
            .and_then(|v| v.as_u64())
            .unwrap_or(30) as u32;

        // Create camera capture instance
        let camera = CameraCapture::new(device_index, width, height, fps)?;
        
        info!("Camera capture initialized: device={}, {}x{}@{}", 
              device_index, width, height, fps);

        self.camera_capture = Some(camera);
        Ok(())
    }

    fn parse_resolution(&self) -> Result<(u32, u32)> {
        let resolution = self.config.parameters
            .get("resolution")
            .and_then(|v| v.as_str())
            .unwrap_or("1920x1080");

        let parts: Vec<&str> = resolution.split('x').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid resolution format: {}", resolution));
        }

        let width = parts[0].parse::<u32>()
            .map_err(|_| anyhow::anyhow!("Invalid width: {}", parts[0]))?;
        let height = parts[1].parse::<u32>()
            .map_err(|_| anyhow::anyhow!("Invalid height: {}", parts[1]))?;

        Ok((width, height))
    }

    fn create_fallback_frame(&self) -> VideoFrame {
        let (width, height) = self.parse_resolution().unwrap_or((1920, 1080));
        
        // Create a simple error pattern (red frame with diagonal lines)
        let frame_size = (width * height * 4) as usize;
        let mut data = vec![0u8; frame_size];
        
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                
                // Create diagonal stripes pattern for error indication
                if (x + y) % 32 < 16 {
                    data[idx] = 255;     // R - red error pattern
                    data[idx + 1] = 0;   // G
                    data[idx + 2] = 0;   // B
                    data[idx + 3] = 255; // A
                } else {
                    data[idx] = 128;     // R - darker red
                    data[idx + 1] = 0;   // G
                    data[idx + 2] = 0;   // B
                    data[idx + 3] = 255; // A
                }
            }
        }

        VideoFrame {
            width,
            height,
            format: VideoFormat::Rgba8,
            data,
        }
    }
}

pub struct VideoFileInputNode {
    id: Uuid,
    config: NodeConfig,
    properties: NodeProperties,
    video_reader: Option<VideoFileReader>,
}

impl VideoFileInputNode {
    pub fn new(id: Uuid, config: NodeConfig) -> Result<Self> {
        let mut parameters = HashMap::new();
        parameters.insert(
            "file_path".to_string(),
            ParameterDefinition {
                name: "File Path".to_string(),
                parameter_type: ParameterType::String,
                default_value: Value::String("".to_string()),
                min_value: None,
                max_value: None,
                description: "Path to video file".to_string(),
            },
        );
        parameters.insert(
            "loop".to_string(),
            ParameterDefinition {
                name: "Loop".to_string(),
                parameter_type: ParameterType::Boolean,
                default_value: Value::Bool(false),
                min_value: None,
                max_value: None,
                description: "Loop playback".to_string(),
            },
        );

        let properties = NodeProperties {
            id,
            name: "Video File Input".to_string(),
            node_type: NodeType::Input(InputType::VideoFile),
            input_types: vec![],
            output_types: vec![ConnectionType::RenderData, ConnectionType::Audio],
            parameters,
        };

        Ok(Self {
            id,
            config,
            properties,
            video_reader: None,
        })
    }
}

impl NodeProcessor for VideoFileInputNode {
    fn process(&mut self, _input: FrameData) -> Result<FrameData> {
        // Initialize video reader if not already done
        if self.video_reader.is_none() {
            if let Err(e) = self.initialize_video_reader() {
                error!("Failed to initialize video reader: {}", e);
                // Continue with fallback instead of failing
            }
        }

        // Read frame from video file
        let (video_frame, audio_frame) = if let Some(ref mut reader) = self.video_reader {
            match reader.read_frame() {
                Ok((video, audio)) => {
                    debug!("Successfully read frame from video file: {}x{}", 
                           video.width, video.height);
                    (Some(video), audio)
                },
                Err(e) => {
                    error!("Failed to read frame from video file: {}", e);
                    // Return a fallback frame instead of failing
                    (Some(self.create_fallback_video_frame()), Some(self.create_fallback_audio_frame()))
                }
            }
        } else {
            error!("Video reader not initialized, using fallback");
            (Some(self.create_fallback_video_frame()), Some(self.create_fallback_audio_frame()))
        };

        Ok(FrameData {
            render_data: video_frame.map(RenderData::Raster2D),
            audio_data: audio_frame.map(|af| UnifiedAudioData::Stereo {
                sample_rate: af.sample_rate,
                channels: af.channels,
                samples: af.samples,
            }),
            control_data: None,
            tally_metadata: TallyMetadata::new(),
        })
    }

    fn get_properties(&self) -> NodeProperties {
        self.properties.clone()
    }

    fn set_parameter(&mut self, key: &str, value: Value) -> Result<()> {
        self.config.parameters.insert(key.to_string(), value);
        // Reset video reader to apply new parameters
        self.video_reader = None;
        Ok(())
    }

    fn get_parameter(&self, key: &str) -> Option<Value> {
        self.config.parameters.get(key).cloned()
    }
}

impl VideoFileInputNode {
    fn initialize_video_reader(&mut self) -> Result<()> {
        info!("Initializing video file reader");

        // Get file path from parameters
        let file_path = self.config.parameters
            .get("file_path")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if file_path.is_empty() {
            return Err(anyhow::anyhow!("No video file path specified"));
        }

        info!("Opening video file: {}", file_path);

        // Create video reader
        let mut reader = VideoFileReader::new(file_path)?;

        // Set loop playback if enabled
        let loop_playback = self.config.parameters
            .get("loop")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        reader.set_loop_playback(loop_playback);

        // Open the video file immediately during initialization
        reader.open()?;

        self.video_reader = Some(reader);
        info!("Video file reader initialized and opened successfully");
        Ok(())
    }

    fn create_fallback_video_frame(&self) -> VideoFrame {
        let width = 1920;
        let height = 1080;
        
        // Create a "No Video" pattern (blue frame with text pattern)
        let frame_size = (width * height * 4) as usize;
        let mut data = vec![0u8; frame_size];
        
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                
                // Create a blue background with white diagonal stripes
                if (x + y) % 64 < 32 {
                    data[idx] = 64;      // R - dark blue
                    data[idx + 1] = 64;  // G
                    data[idx + 2] = 255; // B - blue
                    data[idx + 3] = 255; // A
                } else {
                    data[idx] = 128;     // R - lighter blue
                    data[idx + 1] = 128; // G
                    data[idx + 2] = 255; // B
                    data[idx + 3] = 255; // A
                }
            }
        }

        VideoFrame {
            width,
            height,
            format: VideoFormat::Rgba8,
            data,
        }
    }

    fn create_fallback_audio_frame(&self) -> AudioFrame {
        AudioFrame {
            sample_rate: 48000,
            channels: 2,
            samples: vec![0.0; 2048], // Silent audio
        }
    }
}

pub struct TestPatternNode {
    id: Uuid,
    config: NodeConfig,
    properties: NodeProperties,
}

impl TestPatternNode {
    pub fn new(id: Uuid, config: NodeConfig) -> Result<Self> {
        let mut parameters = HashMap::new();
        parameters.insert(
            "pattern_type".to_string(),
            ParameterDefinition {
                name: "Pattern Type".to_string(),
                parameter_type: ParameterType::Enum(vec![
                    "Color Bars".to_string(),
                    "Gradient".to_string(),
                    "Solid Color".to_string(),
                    "Noise".to_string(),
                ]),
                default_value: Value::String("Color Bars".to_string()),
                min_value: None,
                max_value: None,
                description: "Test pattern type".to_string(),
            },
        );
        parameters.insert(
            "color".to_string(),
            ParameterDefinition {
                name: "Color".to_string(),
                parameter_type: ParameterType::Color,
                default_value: Value::Array(vec![
                    Value::from(1.0),
                    Value::from(1.0),
                    Value::from(1.0),
                    Value::from(1.0),
                ]),
                min_value: None,
                max_value: None,
                description: "Pattern color (RGBA)".to_string(),
            },
        );

        let properties = NodeProperties {
            id,
            name: "Test Pattern".to_string(),
            node_type: NodeType::Input(InputType::TestPattern),
            input_types: vec![],
            output_types: vec![ConnectionType::RenderData],
            parameters,
        };

        Ok(Self {
            id,
            config,
            properties,
        })
    }
}

impl NodeProcessor for TestPatternNode {
    fn process(&mut self, _input: FrameData) -> Result<FrameData> {
        let pattern_type = self
            .get_parameter("pattern_type")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "Color Bars".to_string());

        let frame_data = match pattern_type.as_str() {
            "Color Bars" => self.generate_color_bars(),
            "Gradient" => self.generate_gradient(),
            "Solid Color" => self.generate_solid_color(),
            "Noise" => self.generate_noise(),
            _ => self.generate_color_bars(),
        };

        Ok(FrameData {
            render_data: Some(RenderData::Raster2D(frame_data)),
            audio_data: None,
            control_data: None,
            tally_metadata: TallyMetadata::new(),
        })
    }

    fn get_properties(&self) -> NodeProperties {
        self.properties.clone()
    }

    fn set_parameter(&mut self, key: &str, value: Value) -> Result<()> {
        self.config.parameters.insert(key.to_string(), value);
        Ok(())
    }

    fn get_parameter(&self, key: &str) -> Option<Value> {
        self.config.parameters.get(key).cloned()
    }
}

impl TestPatternNode {
    fn generate_color_bars(&self) -> VideoFrame {
        const WIDTH: u32 = 1920;
        const HEIGHT: u32 = 1080;
        let mut data = vec![0u8; (WIDTH * HEIGHT * 4) as usize];

        let colors = [
            [255, 255, 255, 255], // White
            [255, 255, 0, 255],   // Yellow
            [0, 255, 255, 255],   // Cyan
            [0, 255, 0, 255],     // Green
            [255, 0, 255, 255],   // Magenta
            [255, 0, 0, 255],     // Red
            [0, 0, 255, 255],     // Blue
            [0, 0, 0, 255],       // Black
        ];

        let bar_width = WIDTH / colors.len() as u32;

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let bar_index = (x / bar_width).min(colors.len() as u32 - 1) as usize;
                let pixel_index = ((y * WIDTH + x) * 4) as usize;

                data[pixel_index] = colors[bar_index][0]; // R
                data[pixel_index + 1] = colors[bar_index][1]; // G
                data[pixel_index + 2] = colors[bar_index][2]; // B
                data[pixel_index + 3] = colors[bar_index][3]; // A
            }
        }

        VideoFrame {
            width: WIDTH,
            height: HEIGHT,
            format: VideoFormat::Rgba8,
            data,
        }
    }

    fn generate_gradient(&self) -> VideoFrame {
        const WIDTH: u32 = 1920;
        const HEIGHT: u32 = 1080;
        let mut data = vec![0u8; (WIDTH * HEIGHT * 4) as usize];

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let pixel_index = ((y * WIDTH + x) * 4) as usize;
                let intensity = (x as f32 / WIDTH as f32 * 255.0) as u8;

                data[pixel_index] = intensity; // R
                data[pixel_index + 1] = intensity; // G
                data[pixel_index + 2] = intensity; // B
                data[pixel_index + 3] = 255; // A
            }
        }

        VideoFrame {
            width: WIDTH,
            height: HEIGHT,
            format: VideoFormat::Rgba8,
            data,
        }
    }

    fn generate_solid_color(&self) -> VideoFrame {
        const WIDTH: u32 = 1920;
        const HEIGHT: u32 = 1080;
        let mut data = vec![0u8; (WIDTH * HEIGHT * 4) as usize];

        let color = self
            .get_parameter("color")
            .and_then(|v| v.as_array().map(|arr| arr.clone()))
            .unwrap_or_else(|| {
                vec![
                    Value::from(1.0),
                    Value::from(1.0),
                    Value::from(1.0),
                    Value::from(1.0),
                ]
            });

        let r = (color.get(0).and_then(|v| v.as_f64()).unwrap_or(1.0) * 255.0) as u8;
        let g = (color.get(1).and_then(|v| v.as_f64()).unwrap_or(1.0) * 255.0) as u8;
        let b = (color.get(2).and_then(|v| v.as_f64()).unwrap_or(1.0) * 255.0) as u8;
        let a = (color.get(3).and_then(|v| v.as_f64()).unwrap_or(1.0) * 255.0) as u8;

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let pixel_index = ((y * WIDTH + x) * 4) as usize;
                data[pixel_index] = r;
                data[pixel_index + 1] = g;
                data[pixel_index + 2] = b;
                data[pixel_index + 3] = a;
            }
        }

        VideoFrame {
            width: WIDTH,
            height: HEIGHT,
            format: VideoFormat::Rgba8,
            data,
        }
    }

    fn generate_noise(&self) -> VideoFrame {
        const WIDTH: u32 = 1920;
        const HEIGHT: u32 = 1080;
        let mut data = vec![0u8; (WIDTH * HEIGHT * 4) as usize];

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let pixel_index = ((y * WIDTH + x) * 4) as usize;
                let noise = ((x + y) * 123456789) % 256;

                data[pixel_index] = noise as u8;
                data[pixel_index + 1] = noise as u8;
                data[pixel_index + 2] = noise as u8;
                data[pixel_index + 3] = 255;
            }
        }

        VideoFrame {
            width: WIDTH,
            height: HEIGHT,
            format: VideoFormat::Rgba8,
            data,
        }
    }
}
