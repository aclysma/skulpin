use skulpin_renderer::ash;
pub use sdl2;

pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk;

use skulpin_renderer::PhysicalSize;
use skulpin_renderer::LogicalSize;
use skulpin_renderer::Window;
use std::ffi::CStr;
use ash::prelude::VkResult;

#[cfg(target_os = "windows")]
const DEFAULT_DPI: f32 = 96.0;

pub struct Sdl2Window<'a> {
    window: &'a sdl2::video::Window,
}

impl<'a> Sdl2Window<'a> {
    pub fn new(window: &'a sdl2::video::Window) -> Self {
        Sdl2Window { window }
    }

    #[cfg(target_os = "windows")]
    fn compute_scale_factor(&self) -> Option<f64> {
        let display_index = self.window.display_index().ok()?;
        let system = self.window.subsystem();
        let (_, dpi, _) = system.display_dpi(display_index).ok()?;
        Some((DEFAULT_DPI / dpi).into())
    }
}

impl<'a> Window for Sdl2Window<'a> {
    fn physical_size(&self) -> PhysicalSize {
        let physical_size = self.window.vulkan_drawable_size();
        PhysicalSize::new(physical_size.0, physical_size.1)
    }

    #[cfg(target_os = "windows")]
    fn logical_size(&self) -> LogicalSize {
        let physical_size = self.physical_size();
        physical_size.to_logical(self.scale_factor())
    }

    #[cfg(not(target_os = "windows"))]
    fn logical_size(&self) -> LogicalSize {
        let logical_size = self.window.size();
        LogicalSize::new(logical_size.0, logical_size.1)
    }

    #[cfg(target_os = "windows")]
    fn scale_factor(&self) -> f64 {
        self.compute_scale_factor().unwrap_or(1.0)
    }

    #[cfg(not(target_os = "windows"))]
    fn scale_factor(&self) -> f64 {
        let physical_size = self.window.vulkan_drawable_size();
        let logical_size = self.window.size();
        logical_size.0 as f64 / physical_size.0 as f64
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
