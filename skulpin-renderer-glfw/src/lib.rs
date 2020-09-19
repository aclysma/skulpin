use skulpin_renderer::ash;

pub use glfw;
use ash::vk;

use skulpin_renderer::PhysicalSize;
use skulpin_renderer::LogicalSize;
use skulpin_renderer::Window;
use std::ffi::CStr;
use ash::prelude::VkResult;

pub struct GlfwWindow<'a> {
    window: &'a glfw::Window,
}

impl<'a> GlfwWindow<'a> {
    pub fn new(window: &'a glfw::Window) -> Self {
        GlfwWindow { window }
    }
}

impl<'a> Window for GlfwWindow<'a> {
    fn physical_size(&self) -> PhysicalSize {
        let (x, y) = self.window.get_framebuffer_size();
        PhysicalSize::new(x as u32, y as u32)
    }

    #[cfg(not(target_os = "windows"))]
    fn logical_size(&self) -> LogicalSize {
        let (x, y) = self.window.get_size();
        LogicalSize::new(x as u32, y as u32)
    }

    #[cfg(not(target_os = "windows"))]
    fn scale_factor(&self) -> f64 {
        self.window.get_content_scale().0 as f64
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
