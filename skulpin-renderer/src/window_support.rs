//! OS-specific code required to get a surface for our swapchain

pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk;

use super::PhysicalSize;
use super::LogicalSize;
use std::ffi::CStr;
use ash::prelude::VkResult;

pub trait Window {
    fn physical_size(&self) -> PhysicalSize;
    fn logical_size(&self) -> LogicalSize;
    fn scale_factor(&self) -> f64;

    unsafe fn create_vulkan_surface(
        &self,
        entry: &ash::Entry,
        instance: &ash::Instance,
    ) -> VkResult<vk::SurfaceKHR>;
    fn extension_names(&self) -> VkResult<Vec<&'static CStr>>;
}
