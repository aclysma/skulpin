#[cfg(feature = "winit-21")]
pub use winit_21 as winit;
#[cfg(feature = "winit-22")]
pub use winit_22 as winit;
#[cfg(feature = "winit-23")]
pub use winit_23 as winit;
#[cfg(feature = "winit-latest")]
pub use winit_latest as winit;

use skulpin_renderer::ash;

pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk;

use skulpin_renderer::PhysicalSize;
use skulpin_renderer::LogicalSize;
use skulpin_renderer::Window;
use std::ffi::CStr;
use ash::prelude::VkResult;

#[derive(Clone)]
pub struct WinitWindow<'a> {
    window: &'a winit::window::Window,
}

impl<'a> WinitWindow<'a> {
    pub fn new(window: &'a winit::window::Window) -> Self {
        WinitWindow { window }
    }
}

impl<'a> Window for WinitWindow<'a> {
    fn physical_size(&self) -> PhysicalSize {
        let physical_size: winit::dpi::PhysicalSize<u32> = self.window.inner_size();
        PhysicalSize::new(physical_size.width, physical_size.height)
    }

    fn logical_size(&self) -> LogicalSize {
        let logical_size: winit::dpi::LogicalSize<u32> = self
            .window
            .inner_size()
            .to_logical(self.window.scale_factor());
        LogicalSize::new(logical_size.width, logical_size.height)
    }

    fn scale_factor(&self) -> f64 {
        self.window.scale_factor()
    }

    unsafe fn create_vulkan_surface(
        &self,
        entry: &ash::Entry,
        instance: &ash::Instance,
    ) -> VkResult<vk::SurfaceKHR> {
        ash_window::create_surface(entry, instance, self.window, None)
    }

    fn extension_names(&self) -> VkResult<Vec<&'static CStr>> {
        ash_window::enumerate_required_extensions(self.window)
    }
}
