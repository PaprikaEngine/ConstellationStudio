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

use ash::vk;
use ash::{Device, Entry, Instance};
use std::collections::{HashMap, VecDeque};
use thiserror::Error;

/// Vulkan固有のエラー型
#[derive(Error, Debug)]
pub enum VulkanError {
    #[error("Vulkan initialization failed: {reason}")]
    InitializationFailed { reason: String },

    #[error("Device creation failed: {reason}")]
    DeviceCreationFailed { reason: String },

    #[error("Hardware not supported: {hardware}")]
    HardwareNotSupported { hardware: String },

    #[error("Insufficient memory: required {required_bytes} bytes")]
    InsufficientMemory { required_bytes: u64 },

    #[error("GPU processing failed: {reason}")]
    GpuProcessingFailed { reason: String },
}

pub type VulkanResult<T> = std::result::Result<T, VulkanError>;

pub struct VulkanContext {
    pub entry: Entry,
    pub instance: Instance,
    pub device: Device,
    pub physical_device: vk::PhysicalDevice,
    pub graphics_queue: vk::Queue,
    pub compute_queue: vk::Queue,
    pub transfer_queue: vk::Queue,
    pub graphics_queue_family_index: u32,
    pub compute_queue_family_index: u32,
    pub transfer_queue_family_index: u32,
    pub command_pools: Vec<vk::CommandPool>,
}

impl VulkanContext {
    pub fn new() -> VulkanResult<Self> {
        let entry = unsafe {
            Entry::load().map_err(|e| VulkanError::InitializationFailed {
                reason: format!("Failed to load Vulkan library: {e:?}"),
            })?
        };
        let instance = Self::create_instance(&entry)?;
        let physical_device = Self::select_physical_device(&instance)?;
        let (device, queue_family_indices) =
            Self::create_logical_device(&instance, physical_device)?;

        let graphics_queue = unsafe { device.get_device_queue(queue_family_indices.graphics, 0) };
        let compute_queue = unsafe { device.get_device_queue(queue_family_indices.compute, 0) };
        let transfer_queue = unsafe { device.get_device_queue(queue_family_indices.transfer, 0) };

        let command_pools = Self::create_command_pools(&device, &queue_family_indices)?;

        Ok(Self {
            entry,
            instance,
            device,
            physical_device,
            graphics_queue,
            compute_queue,
            transfer_queue,
            graphics_queue_family_index: queue_family_indices.graphics,
            compute_queue_family_index: queue_family_indices.compute,
            transfer_queue_family_index: queue_family_indices.transfer,
            command_pools,
        })
    }

    fn create_instance(entry: &Entry) -> VulkanResult<Instance> {
        let app_info = vk::ApplicationInfo {
            p_application_name: c"Constellation Studio".as_ptr(),
            application_version: vk::make_api_version(0, 1, 0, 0),
            p_engine_name: c"Constellation Engine".as_ptr(),
            engine_version: vk::make_api_version(0, 1, 0, 0),
            api_version: vk::API_VERSION_1_2,
            ..Default::default()
        };

        let layer_names = [
            #[cfg(debug_assertions)]
            c"VK_LAYER_KHRONOS_validation".as_ptr(),
        ];

        let extension_names = [];

        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            enabled_layer_count: layer_names.len() as u32,
            pp_enabled_layer_names: layer_names.as_ptr(),
            enabled_extension_count: extension_names.len() as u32,
            pp_enabled_extension_names: extension_names.as_ptr(),
            ..Default::default()
        };

        unsafe {
            entry.create_instance(&create_info, None).map_err(|e| {
                VulkanError::InitializationFailed {
                    reason: format!("Failed to create Vulkan instance: {e:?}"),
                }
            })
        }
    }

    fn select_physical_device(instance: &Instance) -> VulkanResult<vk::PhysicalDevice> {
        let physical_devices = unsafe {
            instance.enumerate_physical_devices().map_err(|e| {
                VulkanError::HardwareNotSupported {
                    hardware: format!("Failed to enumerate physical devices: {e:?}"),
                }
            })?
        };

        // Score and rank devices for optimal video processing performance
        let mut scored_devices: Vec<(vk::PhysicalDevice, u32)> = physical_devices
            .into_iter()
            .map(|device| (device, Self::score_device(instance, device)))
            .filter(|(_, score)| *score > 0) // Only include suitable devices
            .collect();

        // Sort by score (highest first)
        scored_devices.sort_by(|a, b| b.1.cmp(&a.1));

        scored_devices
            .first()
            .map(|(device, score)| {
                tracing::info!("Selected GPU with score {}: {:?}", score, unsafe {
                    instance.get_physical_device_properties(*device).device_name
                });
                *device
            })
            .ok_or_else(|| VulkanError::HardwareNotSupported {
                hardware: "No suitable GPU found with required features for video processing"
                    .to_string(),
            })
    }

    fn score_device(instance: &Instance, device: vk::PhysicalDevice) -> u32 {
        let properties = unsafe { instance.get_physical_device_properties(device) };
        let features = unsafe { instance.get_physical_device_features(device) };
        let memory_properties = unsafe { instance.get_physical_device_memory_properties(device) };

        let mut score = 0;

        // Essential features for video processing
        // Note: All Vulkan devices support compute shaders as part of core functionality
        // No explicit feature check needed for compute pipeline support

        // Device type scoring (discrete GPU strongly preferred for video processing)
        match properties.device_type {
            vk::PhysicalDeviceType::DISCRETE_GPU => score += 1000,
            vk::PhysicalDeviceType::INTEGRATED_GPU => score += 500,
            vk::PhysicalDeviceType::VIRTUAL_GPU => score += 300,
            _ => return 0, // Not suitable for video processing
        }

        // Memory size scoring (critical for 4K+ video processing)
        let total_memory: u64 = memory_properties
            .memory_heaps
            .iter()
            .take(memory_properties.memory_heap_count as usize)
            .filter(|heap| heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL))
            .map(|heap| heap.size)
            .sum();

        if total_memory >= 8 * 1024 * 1024 * 1024 {
            // 8GB+
            score += 500;
        } else if total_memory >= 4 * 1024 * 1024 * 1024 {
            // 4GB+
            score += 300;
        } else if total_memory >= 2 * 1024 * 1024 * 1024 {
            // 2GB+
            score += 100;
        } else {
            score += 10; // Minimal memory, may struggle with 4K
        }

        // Compute queue count (parallel video processing)
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(device) };
        let compute_queues: u32 = queue_families
            .iter()
            .filter(|family| family.queue_flags.contains(vk::QueueFlags::COMPUTE))
            .map(|family| family.queue_count)
            .sum();
        score += compute_queues * 50;

        // Preferred features for video processing
        if features.geometry_shader == vk::TRUE {
            score += 100;
        }
        if features.tessellation_shader == vk::TRUE {
            score += 50;
        }
        if features.shader_storage_image_write_without_format == vk::TRUE {
            score += 200; // Important for image processing
        }

        // API version bonus (newer APIs often have better performance)
        let major = vk::api_version_major(properties.api_version);
        let minor = vk::api_version_minor(properties.api_version);
        if major >= 1 && minor >= 3 {
            score += 100;
        } else if major >= 1 && minor >= 2 {
            score += 50;
        }

        score
    }

    fn create_logical_device(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
    ) -> VulkanResult<(Device, QueueFamilyIndices)> {
        let queue_family_indices = Self::find_queue_families(instance, physical_device)?;

        let queue_priorities = [1.0f32];
        let queue_create_infos = [
            vk::DeviceQueueCreateInfo {
                queue_family_index: queue_family_indices.graphics,
                queue_count: 1,
                p_queue_priorities: queue_priorities.as_ptr(),
                ..Default::default()
            },
            vk::DeviceQueueCreateInfo {
                queue_family_index: queue_family_indices.compute,
                queue_count: 1,
                p_queue_priorities: queue_priorities.as_ptr(),
                ..Default::default()
            },
            vk::DeviceQueueCreateInfo {
                queue_family_index: queue_family_indices.transfer,
                queue_count: 1,
                p_queue_priorities: queue_priorities.as_ptr(),
                ..Default::default()
            },
        ];

        let device_features = vk::PhysicalDeviceFeatures {
            geometry_shader: vk::TRUE,
            tessellation_shader: vk::TRUE,
            ..Default::default()
        };

        let device_extensions = [];

        let device_create_info = vk::DeviceCreateInfo {
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            p_enabled_features: &device_features,
            enabled_extension_count: device_extensions.len() as u32,
            pp_enabled_extension_names: device_extensions.as_ptr(),
            ..Default::default()
        };

        let device = unsafe {
            instance
                .create_device(physical_device, &device_create_info, None)
                .map_err(|e| VulkanError::DeviceCreationFailed {
                    reason: format!("Failed to create Vulkan device: {e:?}"),
                })?
        };

        Ok((device, queue_family_indices))
    }

    fn find_queue_families(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
    ) -> VulkanResult<QueueFamilyIndices> {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        // Optimal queue family selection for video processing workloads
        let mut graphics_family = None;
        let mut compute_family = None;
        let mut transfer_family = None;

        // First pass: Find dedicated queues for optimal performance
        for (index, queue_family) in queue_families.iter().enumerate() {
            let index = index as u32;

            // Prefer dedicated compute queue (without graphics) for parallel processing
            if queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE)
                && !queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                && compute_family.is_none()
            {
                compute_family = Some(index);
                tracing::debug!("Found dedicated compute queue family: {}", index);
            }

            // Prefer dedicated transfer queue for async memory operations
            if queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER)
                && !queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                && !queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE)
                && transfer_family.is_none()
            {
                transfer_family = Some(index);
                tracing::debug!("Found dedicated transfer queue family: {}", index);
            }

            // Graphics queue (required for some operations)
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                && graphics_family.is_none()
            {
                graphics_family = Some(index);
                tracing::debug!("Found graphics queue family: {}", index);
            }
        }

        // Second pass: Fallback to shared queues if needed
        for (index, queue_family) in queue_families.iter().enumerate() {
            let index = index as u32;

            // Fallback compute queue (can be shared with graphics)
            if queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE)
                && compute_family.is_none()
            {
                compute_family = Some(index);
                tracing::debug!("Using shared compute queue family: {}", index);
            }

            // Fallback transfer queue (can be shared)
            if queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER)
                && transfer_family.is_none()
            {
                transfer_family = Some(index);
                tracing::debug!("Using shared transfer queue family: {}", index);
            }
        }

        let indices = QueueFamilyIndices {
            graphics: graphics_family.ok_or_else(|| VulkanError::HardwareNotSupported {
                hardware: "Graphics queue family not found".to_string(),
            })?,
            compute: compute_family.ok_or_else(|| VulkanError::HardwareNotSupported {
                hardware: "Compute queue family not found".to_string(),
            })?,
            transfer: transfer_family.ok_or_else(|| VulkanError::HardwareNotSupported {
                hardware: "Transfer queue family not found".to_string(),
            })?,
        };

        tracing::info!(
            "Selected queue families - Graphics: {}, Compute: {}, Transfer: {}",
            indices.graphics,
            indices.compute,
            indices.transfer
        );

        Ok(indices)
    }

    fn create_command_pools(
        device: &Device,
        queue_family_indices: &QueueFamilyIndices,
    ) -> VulkanResult<Vec<vk::CommandPool>> {
        let indices = [
            queue_family_indices.graphics,
            queue_family_indices.compute,
            queue_family_indices.transfer,
        ];

        let mut command_pools = Vec::new();

        for &index in &indices {
            let pool_create_info = vk::CommandPoolCreateInfo {
                flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
                queue_family_index: index,
                ..Default::default()
            };

            let command_pool = unsafe {
                device
                    .create_command_pool(&pool_create_info, None)
                    .map_err(|e| VulkanError::InitializationFailed {
                        reason: format!("Failed to create command pool: {e:?}"),
                    })?
            };

            command_pools.push(command_pool);
        }

        Ok(command_pools)
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe {
            for &command_pool in &self.command_pools {
                self.device.destroy_command_pool(command_pool, None);
            }
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

#[derive(Debug, Clone)]
struct QueueFamilyIndices {
    graphics: u32,
    compute: u32,
    transfer: u32,
}

/// High-performance memory pool manager optimized for video processing
/// Implements pre-allocated memory pools with zero-allocation frame buffer management
pub struct MemoryManager {
    device: Device,
    #[allow(dead_code)] // Phase 2: Will be used for advanced memory type selection
    physical_device: vk::PhysicalDevice,
    #[allow(dead_code)] // Phase 2: Will be used for memory heap analysis
    memory_properties: vk::PhysicalDeviceMemoryProperties,

    // Pre-allocated pools for different frame sizes
    frame_pools: std::collections::HashMap<FrameSize, MemoryPool>,

    // Legacy memory pools for backward compatibility
    memory_pools: Vec<vk::DeviceMemory>,
    free_blocks: VecDeque<MemoryBlock>,

    // Fast allocation tracking
    total_allocated: u64,
    peak_allocation: u64,
    allocation_count: u64,

    // Memory type indices for optimal performance
    device_local_memory_type: u32,
    host_visible_memory_type: u32,
    #[allow(dead_code)] // Phase 2: Will be used for cached memory optimization
    host_coherent_memory_type: u32,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FrameSize {
    pub width: u32,
    pub height: u32,
    pub format: FrameFormat,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum FrameFormat {
    Rgba8, // 4 bytes per pixel
    Bgra8, // 4 bytes per pixel
    Rgb8,  // 3 bytes per pixel
    R8,    // 1 byte per pixel
    R16,   // 2 bytes per pixel
    R32F,  // 4 bytes per pixel (float)
}

impl FrameFormat {
    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            FrameFormat::Rgba8 | FrameFormat::Bgra8 | FrameFormat::R32F => 4,
            FrameFormat::Rgb8 => 3,
            FrameFormat::R16 => 2,
            FrameFormat::R8 => 1,
        }
    }
}

impl FrameSize {
    pub fn buffer_size(&self) -> u64 {
        (self.width * self.height * self.format.bytes_per_pixel()) as u64
    }
}

/// Memory pool for specific frame sizes with pre-allocated buffers
struct MemoryPool {
    memory: vk::DeviceMemory,
    buffer_size: u64,
    #[allow(dead_code)] // Phase 2: Will be used for pool size validation
    buffer_count: u32,
    free_buffers: VecDeque<u32>, // Buffer indices
    #[allow(dead_code)] // Phase 2: Will be used for sub-allocation within pools
    allocation_offset: u64,
    memory_type_index: u32,
}

impl MemoryManager {
    pub fn new(context: &VulkanContext) -> VulkanResult<Self> {
        let memory_properties = unsafe {
            context
                .instance
                .get_physical_device_memory_properties(context.physical_device)
        };

        // Find optimal memory types for video processing
        let device_local_memory_type =
            Self::find_memory_type(&memory_properties, vk::MemoryPropertyFlags::DEVICE_LOCAL)?;

        let host_visible_memory_type = Self::find_memory_type(
            &memory_properties,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        let host_coherent_memory_type = Self::find_memory_type(
            &memory_properties,
            vk::MemoryPropertyFlags::HOST_VISIBLE
                | vk::MemoryPropertyFlags::HOST_COHERENT
                | vk::MemoryPropertyFlags::HOST_CACHED,
        )
        .unwrap_or(host_visible_memory_type); // Fallback to host visible

        tracing::info!(
            "Memory types - Device Local: {}, Host Visible: {}, Host Coherent: {}",
            device_local_memory_type,
            host_visible_memory_type,
            host_coherent_memory_type
        );

        Ok(Self {
            device: context.device.clone(),
            physical_device: context.physical_device,
            memory_properties,
            frame_pools: std::collections::HashMap::new(),
            memory_pools: Vec::new(),
            free_blocks: VecDeque::new(),
            total_allocated: 0,
            peak_allocation: 0,
            allocation_count: 0,
            device_local_memory_type,
            host_visible_memory_type,
            host_coherent_memory_type,
        })
    }

    fn find_memory_type(
        memory_properties: &vk::PhysicalDeviceMemoryProperties,
        required_properties: vk::MemoryPropertyFlags,
    ) -> VulkanResult<u32> {
        for (index, memory_type) in memory_properties
            .memory_types
            .iter()
            .take(memory_properties.memory_type_count as usize)
            .enumerate()
        {
            if memory_type.property_flags.contains(required_properties) {
                return Ok(index as u32);
            }
        }

        Err(VulkanError::HardwareNotSupported {
            hardware: format!("Required memory type not found: {:?}", required_properties),
        })
    }

    /// Pre-allocate frame buffer pool for specific size and format
    /// This enables zero-allocation frame buffer acquisition at runtime
    pub fn create_frame_pool(
        &mut self,
        frame_size: FrameSize,
        buffer_count: u32,
        use_device_local: bool,
    ) -> VulkanResult<()> {
        if self.frame_pools.contains_key(&frame_size) {
            return Ok(());
        }

        let buffer_size = frame_size.buffer_size();
        let total_size = buffer_size * buffer_count as u64;
        let memory_type_index = if use_device_local {
            self.device_local_memory_type
        } else {
            self.host_visible_memory_type
        };

        tracing::info!(
            "Creating frame pool: {}x{} {:?}, {} buffers, {} MB total",
            frame_size.width,
            frame_size.height,
            frame_size.format,
            buffer_count,
            total_size / 1024 / 1024
        );

        let memory_allocate_info = vk::MemoryAllocateInfo {
            allocation_size: total_size,
            memory_type_index,
            ..Default::default()
        };

        let memory = unsafe {
            self.device
                .allocate_memory(&memory_allocate_info, None)
                .map_err(|_e| VulkanError::InsufficientMemory {
                    required_bytes: total_size,
                })?
        };

        let mut free_buffers = VecDeque::new();
        for i in 0..buffer_count {
            free_buffers.push_back(i);
        }

        let pool = MemoryPool {
            memory,
            buffer_size,
            buffer_count,
            free_buffers,
            allocation_offset: 0,
            memory_type_index,
        };

        self.frame_pools.insert(frame_size, pool);
        self.total_allocated += total_size;
        self.peak_allocation = self.peak_allocation.max(self.total_allocated);

        Ok(())
    }

    /// Acquire frame buffer from pre-allocated pool (zero allocation)
    /// This is the primary method for high-performance frame acquisition
    pub fn acquire_frame_buffer(
        &mut self,
        frame_size: &FrameSize,
    ) -> VulkanResult<PooledFrameBuffer> {
        let pool = self.frame_pools.get_mut(frame_size).ok_or_else(|| {
            VulkanError::InsufficientMemory {
                required_bytes: frame_size.buffer_size(),
            }
        })?;

        let buffer_index =
            pool.free_buffers
                .pop_front()
                .ok_or_else(|| VulkanError::InsufficientMemory {
                    required_bytes: frame_size.buffer_size(),
                })?;

        self.allocation_count += 1;

        tracing::trace!(
            "Acquired frame buffer {} from pool {}x{} {:?}",
            buffer_index,
            frame_size.width,
            frame_size.height,
            frame_size.format
        );

        Ok(PooledFrameBuffer {
            memory: pool.memory,
            offset: buffer_index as u64 * pool.buffer_size,
            size: pool.buffer_size,
            pool_frame_size: frame_size.clone(),
            buffer_index,
            memory_type_index: pool.memory_type_index,
        })
    }

    /// Return frame buffer to pool for reuse
    pub fn release_frame_buffer(&mut self, frame_buffer: PooledFrameBuffer) {
        if let Some(pool) = self.frame_pools.get_mut(&frame_buffer.pool_frame_size) {
            pool.free_buffers.push_back(frame_buffer.buffer_index);

            tracing::trace!(
                "Released frame buffer {} to pool {}x{} {:?}",
                frame_buffer.buffer_index,
                frame_buffer.pool_frame_size.width,
                frame_buffer.pool_frame_size.height,
                frame_buffer.pool_frame_size.format
            );
        }
    }

    /// Fallback allocation for non-pooled memory (discouraged for performance)
    pub fn allocate_frame_buffer(
        &mut self,
        size: u64,
        memory_type_index: u32,
    ) -> VulkanResult<FrameBuffer> {
        tracing::warn!(
            "Using fallback allocation for {} bytes - consider using frame pools",
            size
        );

        let memory_allocate_info = vk::MemoryAllocateInfo {
            allocation_size: size,
            memory_type_index,
            ..Default::default()
        };

        let device_memory = unsafe {
            self.device
                .allocate_memory(&memory_allocate_info, None)
                .map_err(|_e| VulkanError::InsufficientMemory {
                    required_bytes: size,
                })?
        };

        self.total_allocated += size;
        self.peak_allocation = self.peak_allocation.max(self.total_allocated);
        self.allocation_count += 1;

        Ok(FrameBuffer::new(device_memory, size))
    }

    pub fn get_memory_usage(&self) -> MemoryUsage {
        MemoryUsage {
            total_allocated: self.total_allocated,
            free_blocks: self.free_blocks.len(),
            total_pools: self.memory_pools.len(),
        }
    }
}

impl Drop for MemoryManager {
    fn drop(&mut self) {
        unsafe {
            // Clean up frame pools
            for pool in self.frame_pools.values() {
                self.device.free_memory(pool.memory, None);
            }

            // Clean up legacy memory pools
            for &memory in &self.memory_pools {
                self.device.free_memory(memory, None);
            }
        }
    }
}

/// High-performance compute pipeline manager for video processing
/// Manages pre-compiled compute shaders for common video operations
pub struct ComputePipelineManager {
    device: Device,
    pipelines: HashMap<String, ComputePipeline>,
    descriptor_set_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
}

/// Individual compute pipeline for specific video processing operations
pub struct ComputePipeline {
    pipeline: vk::Pipeline,
    workgroup_size: [u32; 3],
    #[allow(dead_code)] // Phase 2: Will be used for pipeline management and validation
    operation_type: VideoOperation,
}

/// Supported video processing operations for compute shaders
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum VideoOperation {
    ColorSpaceConversion, // RGB to YUV, etc.
    Resize,               // Bilinear/bicubic scaling
    Blur,                 // Gaussian blur effect
    Sharpen,              // Sharpening filter
    ColorCorrection,      // Brightness/contrast/saturation
    Flip,                 // Horizontal/vertical flip
}

impl ComputePipelineManager {
    pub fn new(context: &VulkanContext) -> VulkanResult<Self> {
        let descriptor_set_layout = Self::create_descriptor_set_layout(&context.device)?;
        let pipeline_layout = Self::create_pipeline_layout(&context.device, descriptor_set_layout)?;

        tracing::info!("Created compute pipeline manager with base layout");

        Ok(Self {
            device: context.device.clone(),
            pipelines: HashMap::new(),
            descriptor_set_layout,
            pipeline_layout,
        })
    }

    fn create_descriptor_set_layout(device: &Device) -> VulkanResult<vk::DescriptorSetLayout> {
        let bindings = [
            // Input image binding
            vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: vk::DescriptorType::STORAGE_IMAGE,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::COMPUTE,
                p_immutable_samplers: std::ptr::null(),
                ..Default::default()
            },
            // Output image binding
            vk::DescriptorSetLayoutBinding {
                binding: 1,
                descriptor_type: vk::DescriptorType::STORAGE_IMAGE,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::COMPUTE,
                p_immutable_samplers: std::ptr::null(),
                ..Default::default()
            },
            // Parameters buffer binding
            vk::DescriptorSetLayoutBinding {
                binding: 2,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::COMPUTE,
                p_immutable_samplers: std::ptr::null(),
                ..Default::default()
            },
        ];

        let layout_info = vk::DescriptorSetLayoutCreateInfo {
            binding_count: bindings.len() as u32,
            p_bindings: bindings.as_ptr(),
            ..Default::default()
        };

        unsafe {
            device
                .create_descriptor_set_layout(&layout_info, None)
                .map_err(|e| VulkanError::InitializationFailed {
                    reason: format!("Failed to create descriptor set layout: {e:?}"),
                })
        }
    }

    fn create_pipeline_layout(
        device: &Device,
        descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> VulkanResult<vk::PipelineLayout> {
        let set_layouts = [descriptor_set_layout];

        let layout_info = vk::PipelineLayoutCreateInfo {
            set_layout_count: set_layouts.len() as u32,
            p_set_layouts: set_layouts.as_ptr(),
            push_constant_range_count: 0,
            p_push_constant_ranges: std::ptr::null(),
            ..Default::default()
        };

        unsafe {
            device
                .create_pipeline_layout(&layout_info, None)
                .map_err(|e| VulkanError::InitializationFailed {
                    reason: format!("Failed to create pipeline layout: {e:?}"),
                })
        }
    }

    /// Create a compute pipeline for specific video operation
    /// Phase 1: Basic pipeline creation framework
    /// Phase 2: Will load actual SPIR-V shaders for each operation
    pub fn create_pipeline(&mut self, operation: VideoOperation) -> VulkanResult<()> {
        if self.pipelines.contains_key(&operation.shader_name()) {
            return Ok(()); // Pipeline already exists
        }

        // Phase 1: Create placeholder pipeline
        // Phase 2: Will load actual SPIR-V shader bytecode
        let workgroup_size = operation.optimal_workgroup_size();

        tracing::info!(
            "Creating compute pipeline for {:?} with workgroup size {:?}",
            operation,
            workgroup_size
        );

        // For Phase 1, create a basic compute pipeline structure
        // Phase 2 will implement actual shader loading and compilation
        let placeholder_pipeline = ComputePipeline {
            pipeline: vk::Pipeline::null(), // Phase 1: Placeholder
            workgroup_size,
            operation_type: operation.clone(),
        };

        self.pipelines
            .insert(operation.shader_name(), placeholder_pipeline);

        tracing::info!("Created placeholder compute pipeline for {:?}", operation);
        Ok(())
    }

    /// Get pipeline for specific video operation
    pub fn get_pipeline(&self, operation: &VideoOperation) -> Option<&ComputePipeline> {
        self.pipelines.get(&operation.shader_name())
    }

    /// Execute compute operation on video frame
    /// Phase 1: Framework for compute dispatch
    /// Phase 2: Will implement actual GPU execution
    pub fn execute_operation(
        &self,
        operation: &VideoOperation,
        input_frame: &PooledFrameBuffer,
        output_frame: &PooledFrameBuffer,
        _command_buffer: vk::CommandBuffer,
    ) -> VulkanResult<()> {
        let pipeline =
            self.get_pipeline(operation)
                .ok_or_else(|| VulkanError::GpuProcessingFailed {
                    reason: format!("Pipeline for {:?} not found", operation),
                })?;

        // Phase 1: Log the operation for development
        tracing::debug!(
            "Executing {:?} operation: {} -> {} (workgroup: {:?})",
            operation,
            input_frame.size(),
            output_frame.size(),
            pipeline.workgroup_size
        );

        // Phase 2 will implement:
        // - Bind descriptor sets
        // - Dispatch compute shader
        // - Memory barriers
        // - Synchronization

        Ok(())
    }

    pub fn descriptor_set_layout(&self) -> vk::DescriptorSetLayout {
        self.descriptor_set_layout
    }

    pub fn pipeline_layout(&self) -> vk::PipelineLayout {
        self.pipeline_layout
    }
}

impl VideoOperation {
    fn shader_name(&self) -> String {
        match self {
            VideoOperation::ColorSpaceConversion => "color_space_conversion".to_string(),
            VideoOperation::Resize => "resize".to_string(),
            VideoOperation::Blur => "blur".to_string(),
            VideoOperation::Sharpen => "sharpen".to_string(),
            VideoOperation::ColorCorrection => "color_correction".to_string(),
            VideoOperation::Flip => "flip".to_string(),
        }
    }

    fn optimal_workgroup_size(&self) -> [u32; 3] {
        match self {
            VideoOperation::ColorSpaceConversion => [16, 16, 1], // 2D processing
            VideoOperation::Resize => [32, 8, 1],                // Optimized for memory access
            VideoOperation::Blur => [16, 16, 1],                 // 2D kernel processing
            VideoOperation::Sharpen => [16, 16, 1],              // 2D kernel processing
            VideoOperation::ColorCorrection => [64, 1, 1],       // 1D processing
            VideoOperation::Flip => [32, 8, 1],                  // Memory bandwidth bound
        }
    }
}

impl Drop for ComputePipelineManager {
    fn drop(&mut self) {
        unsafe {
            // Clean up pipelines
            for pipeline in self.pipelines.values() {
                if pipeline.pipeline != vk::Pipeline::null() {
                    self.device.destroy_pipeline(pipeline.pipeline, None);
                }
            }

            // Clean up layouts
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryBlock {
    pub memory: vk::DeviceMemory,
    pub size: u64,
    pub offset: u64,
}

/// High-performance pooled frame buffer with automatic cleanup
/// Returned by MemoryManager::acquire_frame_buffer for zero-allocation frame access
pub struct PooledFrameBuffer {
    memory: vk::DeviceMemory,
    offset: u64,
    size: u64,
    pool_frame_size: FrameSize,
    buffer_index: u32,
    memory_type_index: u32,
}

impl PooledFrameBuffer {
    pub fn memory(&self) -> vk::DeviceMemory {
        self.memory
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn memory_type_index(&self) -> u32 {
        self.memory_type_index
    }

    pub fn frame_size(&self) -> &FrameSize {
        &self.pool_frame_size
    }
}

/// Traditional frame buffer for fallback allocation
pub struct FrameBuffer {
    memory: vk::DeviceMemory,
    size: u64,
}

impl FrameBuffer {
    pub fn new(memory: vk::DeviceMemory, size: u64) -> Self {
        Self { memory, size }
    }

    pub fn from_block(block: MemoryBlock) -> Self {
        Self {
            memory: block.memory,
            size: block.size,
        }
    }

    pub fn into_block(self) -> MemoryBlock {
        MemoryBlock {
            memory: self.memory,
            size: self.size,
            offset: 0,
        }
    }

    pub fn memory(&self) -> vk::DeviceMemory {
        self.memory
    }

    pub fn size(&self) -> u64 {
        self.size
    }
}

#[derive(Debug)]
pub struct MemoryUsage {
    pub total_allocated: u64,
    pub free_blocks: usize,
    pub total_pools: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vulkan_context_creation() {
        let result = VulkanContext::new();
        match result {
            Ok(_) => println!("Vulkan context created successfully"),
            Err(e) => println!("Failed to create Vulkan context: {e}"),
        }
    }

    #[test]
    fn test_memory_manager_creation() {
        if let Ok(context) = VulkanContext::new() {
            let result = MemoryManager::new(&context);
            assert!(result.is_ok());
        }
    }
}
