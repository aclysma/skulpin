use skulpin_renderer::ash;
use ash::vk;
use ash::prelude::VkResult;

mod imgui_renderpass;
pub use imgui_renderpass::VkImGuiRenderPass;

mod image;
pub use self::image::VkImage;

pub use imgui;

use skulpin_renderer::VkSwapchain;
use skulpin_renderer::VkDevice;
use skulpin_renderer::Window;
use crate::imgui_renderpass::VkImGuiRenderPassFontAtlas;

pub struct ImguiRendererPlugin {
    // The renderpass, none if it's not created
    imgui_renderpass: Option<VkImGuiRenderPass>,

    // A copy of the font atlas, used so we can recreate the renderpass as needed
    font_atlas: VkImGuiRenderPassFontAtlas,
}

impl ImguiRendererPlugin {
    pub fn new(imgui_context: &mut imgui::Context) -> Self {
        // Copy off the font atlas during initialization so that we can re-create the renderpass
        // as needed
        let mut fonts = imgui_context.fonts();
        let font_atlas_texture = fonts.build_rgba32_texture();
        let font_atlas = VkImGuiRenderPassFontAtlas::new(&font_atlas_texture);

        ImguiRendererPlugin {
            imgui_renderpass: None,
            font_atlas,
        }
    }
}

impl skulpin_renderer::RendererPlugin for ImguiRendererPlugin {
    fn swapchain_created(
        &mut self,
        device: &VkDevice,
        swapchain: &VkSwapchain,
    ) -> VkResult<()> {
        self.imgui_renderpass = Some(VkImGuiRenderPass::new(device, swapchain, &self.font_atlas)?);
        Ok(())
    }

    fn swapchain_destroyed(&mut self) {
        self.imgui_renderpass = None;
    }

    fn render(
        &mut self,
        window: &dyn Window,
        device: &VkDevice,
        present_index: usize,
    ) -> VkResult<Vec<vk::CommandBuffer>> {
        let draw_data = unsafe { imgui::sys::igGetDrawData() };
        if draw_data.is_null() {
            log::warn!("no draw data available");
            return Err(vk::Result::ERROR_INITIALIZATION_FAILED);
        }

        let draw_data = unsafe { &*(draw_data as *mut imgui::DrawData) };

        let renderpass = self.imgui_renderpass.as_mut().unwrap();

        renderpass.update(
            &device.memory_properties,
            Some(&draw_data),
            present_index as usize,
            window.scale_factor(),
        )?;

        Ok(vec![renderpass.command_buffers[present_index].clone()])
    }
}
