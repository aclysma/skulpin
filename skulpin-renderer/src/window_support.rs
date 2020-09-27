//! OS-specific code required to get a surface for our swapchain

pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk;

use super::PhysicalSize;
use super::LogicalSize;
use std::ffi::CStr;
use ash::prelude::VkResult;

pub trait Window {
    /// Must return the size in pixels
    fn physical_size(&self) -> PhysicalSize;

    /// Must return the size in logical units (i.e. on a 2x hidpi screen, a 4k window might be
    /// 1080p in logical units
    fn logical_size(&self) -> LogicalSize;

    /// A scale factor to convert between physical/logical size
    fn scale_factor(&self) -> f64;

    /// Create a surface
    ///
    /// # Safety
    ///
    /// Interacting with the vulkan API is intrinsically unsafe and relies upon the entry and
    /// instance being properly configured.
    unsafe fn create_vulkan_surface(
        &self,
        entry: &ash::Entry,
        instance: &ash::Instance,
    ) -> VkResult<vk::SurfaceKHR>;

    /// Return the vulkan extensions required to create and use the vulkan surface
    fn extension_names(&self) -> VkResult<Vec<&'static CStr>>;
}
