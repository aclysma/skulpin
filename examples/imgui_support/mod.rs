use skulpin_renderer::ash;
use ash::vk;
use ash::prelude::VkResult;

mod imgui_support;
pub use imgui_support::init_imgui_manager;
pub use imgui_support::ImguiManager;

mod imgui_renderpass;
pub use imgui_renderpass::VkImGuiRenderPass;

mod image;
pub use self::image::VkImage;

pub use imgui;
pub use imgui_winit_support;

use skulpin_renderer::VkSwapchain;
use skulpin_renderer::VkDevice;
use skulpin_renderer::Window;

pub struct ImguiRendererPlugin {
    imgui_manager: ImguiManager,
    imgui_renderpass: Option<VkImGuiRenderPass>,
}

impl ImguiRendererPlugin {
    pub fn new(imgui_manager: ImguiManager) -> Self {
        ImguiRendererPlugin {
            imgui_manager,
            imgui_renderpass: None,
        }
    }
}

impl skulpin_renderer::RendererPlugin for ImguiRendererPlugin {
    fn swapchain_created(
        &mut self,
        device: &VkDevice,
        swapchain: &VkSwapchain,
    ) -> VkResult<()> {
        self.imgui_renderpass = Some(VkImGuiRenderPass::new(
            device,
            swapchain,
            self.imgui_manager.clone(),
        )?);
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
        let imgui_draw_data: Option<&imgui::DrawData> = self.imgui_manager.draw_data();

        let renderpass = self.imgui_renderpass.as_mut().unwrap();

        renderpass.update(
            &device.memory_properties,
            imgui_draw_data,
            present_index as usize,
            window.scale_factor(),
        )?;

        Ok(vec![renderpass.command_buffers[present_index].clone()])
    }
}
