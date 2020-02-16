//! OS-specific code required to get a surface for our swapchain

pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk;
use ash::vk::Handle;

use sdl2::video::Window;

pub unsafe fn create_surface<I: InstanceV1_0>(window: &Window, instance: &I) -> Result<vk::SurfaceKHR, String> {
    let surface_pointer = window.vulkan_create_surface(instance.handle().as_raw() as usize)?;
    Ok(vk::SurfaceKHR::from_raw(surface_pointer as u64))
}

pub fn extension_names(window: &Window) -> Vec<*const i8> {
    window.vulkan_instance_extensions()
        .expect("Could not get vulkan instance extensions")
        .into_iter()
        .map(|extension| extension.as_ptr() as *const i8)
        .collect()
}
