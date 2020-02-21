//! OS-specific code required to get a surface for our swapchain

pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk;
use ash::vk::Handle;

use super::PhysicalSize;
use super::LogicalSize;

mod sdl2_support;
pub use sdl2_support::Sdl2Window;

mod winit_support;
pub use winit_support::WinitWindow;

pub trait Window {
    fn physical_size(&self) -> PhysicalSize;
    fn logical_size(&self) -> LogicalSize;
    fn scale_factor(&self) -> f64;

    //TODO: Break these out into a separate WindowSystem trait?
    fn create_vulkan_surface(&self, entry: &ash::Entry, instance: &ash::Instance) -> Result<vk::SurfaceKHR, vk::Result>;
    fn extension_names(&self) -> Vec<*const i8>;
}
