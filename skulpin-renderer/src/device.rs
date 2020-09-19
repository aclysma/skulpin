use ash::vk;
use ash::prelude::VkResult;
use super::VkInstance;

use ash::version::DeviceV1_0;
use ash::version::InstanceV1_0;
use super::Window;

use std::ffi::CStr;

//use ash::extensions::ext as ash_ext;
use ash::extensions::khr;
use crate::PhysicalDeviceType;

/// Has the indexes for all the queue families we will need. It's possible a single family
/// is used for both graphics and presentation, in which case the index will be the same
#[derive(Default)]
pub struct VkQueueFamilyIndices {
    pub graphics_queue_family_index: u32,
    pub present_queue_family_index: u32,
}

/// An instantiated queue per queue family. We only need one queue per family.
pub struct VkQueues {
    pub graphics_queue: ash::vk::Queue,
    pub present_queue: ash::vk::Queue,
}

/// Represents the surface/physical device/logical device. Most of the code here has to do with
/// picking a good device that's compatible with the window we're given.
pub struct VkDevice {
    pub surface: ash::vk::SurfaceKHR,
    pub surface_loader: ash::extensions::khr::Surface,
    pub physical_device: ash::vk::PhysicalDevice,
    pub logical_device: ash::Device,
    pub queue_family_indices: VkQueueFamilyIndices,
    pub queues: VkQueues,
    pub memory_properties: vk::PhysicalDeviceMemoryProperties,
}

impl VkDevice {
    pub fn new(
        instance: &VkInstance,
        window: &dyn Window,
        physical_device_type_priority: &[PhysicalDeviceType],
    ) -> VkResult<Self> {
        // Get the surface, needed to select the best queue family
        let surface = unsafe {
            window
                .create_vulkan_surface(&instance.entry, &instance.instance)
                .expect("Could not create vulkan surface")
        };

        let surface_loader = khr::Surface::new(&instance.entry, &instance.instance);

        // Pick a physical device
        let (physical_device, queue_family_indices) = Self::choose_physical_device(
            &instance.instance,
            &surface_loader,
            surface,
            physical_device_type_priority,
        )?;

        // Create a logical device
        let (logical_device, queues) = Self::create_logical_device(
            &instance.instance,
            physical_device,
            &queue_family_indices,
        )?;

        let memory_properties = unsafe {
            instance
                .instance
                .get_physical_device_memory_properties(physical_device)
        };

        Ok(VkDevice {
            surface,
            surface_loader,
            physical_device,
            logical_device,
            queue_family_indices,
            queues,
            memory_properties,
        })
    }

    fn choose_physical_device(
        instance: &ash::Instance,
        surface_loader: &ash::extensions::khr::Surface,
        surface: ash::vk::SurfaceKHR,
        physical_device_type_priority: &[PhysicalDeviceType],
    ) -> VkResult<(ash::vk::PhysicalDevice, VkQueueFamilyIndices)> {
        let physical_devices = unsafe { instance.enumerate_physical_devices()? };

        if physical_devices.is_empty() {
            panic!("Could not find a physical device");
        }

        let mut best_physical_device = None;
        let mut best_physical_device_score = -1;
        let mut best_physical_device_queue_family_indices = None;
        for physical_device in physical_devices {
            let result = Self::get_score_and_queue_families_for_physical_device(
                instance,
                physical_device,
                surface_loader,
                surface,
                physical_device_type_priority,
            );

            if let Some((score, queue_family_indices)) = result? {
                if score > best_physical_device_score {
                    best_physical_device = Some(physical_device);
                    best_physical_device_score = score;
                    best_physical_device_queue_family_indices = Some(queue_family_indices);
                }
            }
        }

        //TODO: Return an error
        let physical_device = best_physical_device.expect("Could not find suitable device");
        let queue_family_indices = best_physical_device_queue_family_indices.unwrap();

        Ok((physical_device, queue_family_indices))
    }

    fn vk_version_to_string(version: u32) -> String {
        format!(
            "{}.{}.{}",
            vk::version_major(version),
            vk::version_minor(version),
            vk::version_patch(version)
        )
    }

    fn get_score_and_queue_families_for_physical_device(
        instance: &ash::Instance,
        device: ash::vk::PhysicalDevice,
        surface_loader: &ash::extensions::khr::Surface,
        surface: ash::vk::SurfaceKHR,
        physical_device_type_priority: &[PhysicalDeviceType],
    ) -> VkResult<Option<(i32, VkQueueFamilyIndices)>> {
        info!(
            "Preferred device types: {:?}",
            physical_device_type_priority
        );

        let properties: ash::vk::PhysicalDeviceProperties =
            unsafe { instance.get_physical_device_properties(device) };
        let device_name = unsafe {
            CStr::from_ptr(properties.device_name.as_ptr())
                .to_str()
                .unwrap()
                .to_string()
        };

        //TODO: Check that the extensions we want to use are supported
        let _extensions: Vec<ash::vk::ExtensionProperties> =
            unsafe { instance.enumerate_device_extension_properties(device)? };
        let _features: vk::PhysicalDeviceFeatures =
            unsafe { instance.get_physical_device_features(device) };

        let queue_family_indices =
            Self::find_queue_families(instance, device, surface_loader, surface)?;
        if let Some(queue_family_indices) = queue_family_indices {
            // Determine the index of the device_type within physical_device_type_priority
            let index = physical_device_type_priority
                .iter()
                .map(|x| x.to_vk())
                .position(|x| x == properties.device_type);

            // Convert it to a score
            let rank = if let Some(index) = index {
                // It's in the list, return a value between 1..n
                physical_device_type_priority.len() - index
            } else {
                // Not in the list, return a zero
                0
            } as i32;

            let mut score = 0;
            score += rank * 100;

            info!(
                "Found suitable device '{}' API: {} DriverVersion: {} Score = {}",
                device_name,
                Self::vk_version_to_string(properties.api_version),
                Self::vk_version_to_string(properties.driver_version),
                score
            );

            trace!("{:#?}", properties);
            Ok(Some((score, queue_family_indices)))
        } else {
            info!(
                "Found unsuitable device '{}' API: {} DriverVersion: {} could not find queue families",
                device_name,
                Self::vk_version_to_string(properties.api_version),
                Self::vk_version_to_string(properties.driver_version)
            );
            trace!("{:#?}", properties);
            Ok(None)
        }
    }

    fn find_queue_families(
        instance: &ash::Instance,
        physical_device: ash::vk::PhysicalDevice,
        surface_loader: &ash::extensions::khr::Surface,
        surface: ash::vk::SurfaceKHR,
    ) -> VkResult<Option<VkQueueFamilyIndices>> {
        let queue_families: Vec<ash::vk::QueueFamilyProperties> =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut graphics_queue_family_index = None;
        let mut present_queue_family_index = None;

        info!("Available queue families:");
        for (queue_family_index, queue_family) in queue_families.iter().enumerate() {
            info!("Queue Family {}", queue_family_index);
            info!("{:#?}", queue_family);
        }

        for (queue_family_index, queue_family) in queue_families.iter().enumerate() {
            let queue_family_index = queue_family_index as u32;

            let supports_graphics = queue_family.queue_flags & ash::vk::QueueFlags::GRAPHICS
                == ash::vk::QueueFlags::GRAPHICS;
            let supports_present = unsafe {
                surface_loader.get_physical_device_surface_support(
                    physical_device,
                    queue_family_index,
                    surface,
                )?
            };

            // A queue family that supports both is ideal. If we find it, break out early.
            if supports_graphics && supports_present {
                graphics_queue_family_index = Some(queue_family_index);
                present_queue_family_index = Some(queue_family_index);
                break;
            }

            // Otherwise, remember the first graphics queue family we saw...
            if supports_graphics && graphics_queue_family_index.is_none() {
                graphics_queue_family_index = Some(queue_family_index);
            }

            // and the first present queue family we saw
            if supports_present && present_queue_family_index.is_none() {
                present_queue_family_index = Some(queue_family_index);
            }
        }

        info!(
            "Graphics QF: {:?}  Present QF: {:?}",
            graphics_queue_family_index, present_queue_family_index
        );

        if let (Some(graphics_queue_family_index), Some(present_queue_family_index)) =
            (graphics_queue_family_index, present_queue_family_index)
        {
            Ok(Some(VkQueueFamilyIndices {
                graphics_queue_family_index,
                present_queue_family_index,
            }))
        } else {
            Ok(None)
        }
    }

    fn create_logical_device(
        instance: &ash::Instance,
        physical_device: ash::vk::PhysicalDevice,
        queue_family_indices: &VkQueueFamilyIndices,
    ) -> VkResult<(ash::Device, VkQueues)> {
        //TODO: Ideally we would set up validation layers for the logical device too.

        let device_extension_names_raw = [khr::Swapchain::name().as_ptr()];
        let features = vk::PhysicalDeviceFeatures::builder();
        let priorities = [1.0];

        let mut queue_families_to_create = std::collections::HashSet::new();
        queue_families_to_create.insert(queue_family_indices.graphics_queue_family_index);
        queue_families_to_create.insert(queue_family_indices.present_queue_family_index);

        let queue_infos: Vec<_> = queue_families_to_create
            .iter()
            .map(|queue_family_index| {
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(*queue_family_index)
                    .queue_priorities(&priorities)
                    .build()
            })
            .collect();

        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(&device_extension_names_raw)
            .enabled_features(&features);

        let device: ash::Device =
            unsafe { instance.create_device(physical_device, &device_create_info, None)? };

        let graphics_queue =
            unsafe { device.get_device_queue(queue_family_indices.graphics_queue_family_index, 0) };

        let present_queue =
            unsafe { device.get_device_queue(queue_family_indices.present_queue_family_index, 0) };

        let queues = VkQueues {
            graphics_queue,
            present_queue,
        };

        Ok((device, queues))
    }
}

impl Drop for VkDevice {
    fn drop(&mut self) {
        debug!("destroying VkDevice");
        unsafe {
            self.logical_device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
        }

        debug!("destroyed VkDevice");
    }
}
