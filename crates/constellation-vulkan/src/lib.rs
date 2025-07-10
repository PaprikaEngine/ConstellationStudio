use ash::vk;
use ash::{Device, Entry, Instance};
use std::collections::VecDeque;
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

        physical_devices
            .into_iter()
            .find(|&device| Self::is_device_suitable(instance, device))
            .ok_or_else(|| VulkanError::HardwareNotSupported {
                hardware: "No suitable GPU found with required features".to_string(),
            })
    }

    fn is_device_suitable(instance: &Instance, device: vk::PhysicalDevice) -> bool {
        let properties = unsafe { instance.get_physical_device_properties(device) };
        let features = unsafe { instance.get_physical_device_features(device) };

        properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
            && features.geometry_shader == vk::TRUE
            && features.tessellation_shader == vk::TRUE
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

        let mut graphics_family = None;
        let mut compute_family = None;
        let mut transfer_family = None;

        for (index, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics_family = Some(index as u32);
            }

            if queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
                compute_family = Some(index as u32);
            }

            if queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER) {
                transfer_family = Some(index as u32);
            }
        }

        Ok(QueueFamilyIndices {
            graphics: graphics_family.ok_or_else(|| VulkanError::HardwareNotSupported {
                hardware: "Graphics queue family not found".to_string(),
            })?,
            compute: compute_family.ok_or_else(|| VulkanError::HardwareNotSupported {
                hardware: "Compute queue family not found".to_string(),
            })?,
            transfer: transfer_family.ok_or_else(|| VulkanError::HardwareNotSupported {
                hardware: "Transfer queue family not found".to_string(),
            })?,
        })
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

pub struct MemoryManager {
    device: Device,
    #[allow(dead_code)]
    physical_device: vk::PhysicalDevice,
    memory_pools: Vec<vk::DeviceMemory>,
    free_blocks: VecDeque<MemoryBlock>,
    total_allocated: u64,
    #[allow(dead_code)]
    max_allocation_size: u64,
}

impl MemoryManager {
    pub fn new(context: &VulkanContext) -> VulkanResult<Self> {
        let memory_properties = unsafe {
            context
                .instance
                .get_physical_device_memory_properties(context.physical_device)
        };

        let max_allocation_size = memory_properties.memory_heaps[0].size / 4;

        Ok(Self {
            device: context.device.clone(),
            physical_device: context.physical_device,
            memory_pools: Vec::new(),
            free_blocks: VecDeque::new(),
            total_allocated: 0,
            max_allocation_size,
        })
    }

    pub fn allocate_frame_buffer(
        &mut self,
        size: u64,
        memory_type_index: u32,
    ) -> VulkanResult<FrameBuffer> {
        if let Some(block) = self.free_blocks.pop_front() {
            if block.size >= size {
                return Ok(FrameBuffer::from_block(block));
            }
            self.free_blocks.push_back(block);
        }

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

        self.memory_pools.push(device_memory);
        self.total_allocated += size;

        Ok(FrameBuffer::new(device_memory, size))
    }

    pub fn free_frame_buffer(&mut self, frame_buffer: FrameBuffer) {
        let block = frame_buffer.into_block();
        self.free_blocks.push_back(block);
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
            for &memory in &self.memory_pools {
                self.device.free_memory(memory, None);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryBlock {
    pub memory: vk::DeviceMemory,
    pub size: u64,
    pub offset: u64,
}

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
