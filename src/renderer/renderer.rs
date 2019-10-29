
use std::ffi::CString;

use ash::version::DeviceV1_0;
use ash::prelude::VkResult;

use std::mem::ManuallyDrop;
use ash::vk;

use super::VkInstance;
use super::VkDevice;
use super::VkSkiaContext;
use super::VkSwapchain;
use super::VkPipeline;
use super::MAX_FRAMES_IN_FLIGHT;


pub struct RendererBuilder {
    app_name: CString,
    use_vulkan_debug_layer: bool
}

impl RendererBuilder {
    pub fn new() -> Self {
        RendererBuilder {
            app_name: CString::new("Skulpin").unwrap(),
            use_vulkan_debug_layer: false
        }
    }

    pub fn app_name(mut self, app_name: CString) -> Self {
        self.app_name = app_name;
        self
    }

    pub fn use_vulkan_debug_layer(mut self, use_vulkan_debug_layer: bool) -> RendererBuilder {
        self.use_vulkan_debug_layer = use_vulkan_debug_layer;
        self
    }

    pub fn build(&self, window: &winit::window::Window) -> VkResult<Renderer> {
        Renderer::new(&self.app_name, window, self.use_vulkan_debug_layer)
    }
}

pub struct Renderer {
    instance: ManuallyDrop<VkInstance>,
    device: ManuallyDrop<VkDevice>,

    skia_context: ManuallyDrop<VkSkiaContext>,

    swapchain: ManuallyDrop<VkSwapchain>,
    pipeline: ManuallyDrop<VkPipeline>,

    // Increase until > MAX_FRAMES_IN_FLIGHT, then set to 0, or -1 if no frame drawn yet
    sync_frame_index: usize,
}

impl Renderer {
    pub fn new(
        app_name: &CString,
        window: &winit::window::Window,
        use_vulkan_debug_layer: bool
    ) -> VkResult<Renderer> {
        let instance = ManuallyDrop::new(VkInstance::new(app_name, use_vulkan_debug_layer)?);
        let device = ManuallyDrop::new(VkDevice::new(&instance, window)?);
        let mut skia_context = ManuallyDrop::new(VkSkiaContext::new(&instance, &device));
        let swapchain = ManuallyDrop::new(VkSwapchain::new(&instance, &device, window)?);
        let pipeline = ManuallyDrop::new(VkPipeline::new(&device, &swapchain, &mut skia_context)?);
        let sync_frame_index = 0;

        Ok(Renderer {
            instance,
            device,
            skia_context,
            swapchain,
            pipeline,
            sync_frame_index
        })
    }

    pub fn draw<F : FnOnce(&mut skia_safe::Canvas)>(
        &mut self,
        window: &winit::window::Window,
        f: F
    ) -> VkResult<()> {
        let result = self.do_draw(window, f);
        if let Err(e) = result {
            match e {
                ash::vk::Result::ERROR_OUT_OF_DATE_KHR => {

                    //TODO: Clean the do_draw stuff up
                    //TODO: How does it work to render from another thread?
                    unsafe {
                        self.device.logical_device.device_wait_idle()?;
                        ManuallyDrop::drop(&mut self.pipeline);
                        ManuallyDrop::drop(&mut self.swapchain);
                    }

                    self.swapchain = ManuallyDrop::new(VkSwapchain::new(&self.instance, &self.device, window)?);
                    self.pipeline = ManuallyDrop::new(VkPipeline::new(&self.device, &self.swapchain, &mut self.skia_context)?);
                    Ok(())
                },
                ash::vk::Result::SUCCESS => {
                    Ok(())
                },
                ash::vk::Result::SUBOPTIMAL_KHR => {
                    Ok(())
                },
                _ => {
                    warn!("Unexpected rendering error");
                    Err(e)
                }
            }
        } else {
            Ok(())
        }
    }

    fn do_draw<F : FnOnce(&mut skia_safe::Canvas)>(
        &mut self,
        _window: &winit::window::Window,
        f: F
    )
        -> VkResult<()>
    {
        let frame_fence = self.swapchain.in_flight_fences[self.sync_frame_index];

        //TODO: Dont lock up forever (don't use std::u64::MAX)
        //TODO: Can part of this run in a separate thread from the window pump?

        // Wait if two frame are already in flight
        unsafe {
            self.device.logical_device.wait_for_fences(&[frame_fence], true, std::u64::MAX)?;
            self.device.logical_device.reset_fences(&[frame_fence])?;
        }

        let (present_index, _is_suboptimal) = unsafe {
            self.swapchain
                .swapchain_loader
                .acquire_next_image(
                    self.swapchain.swapchain,
                    std::u64::MAX,
                    self.swapchain.image_available_semaphores[self.sync_frame_index],
                    vk::Fence::null(),
                )?
        };

        {
            let surface = self.pipeline.skia_surface(present_index as usize);
            let mut canvas = surface.surface.canvas();

            f(&mut canvas);

            canvas.flush();
        }

        self.pipeline.update_uniform_buffer(present_index, self.swapchain.swapchain_info.extents)?;

        let wait_semaphores = [self.swapchain.image_available_semaphores[self.sync_frame_index]];
        let signal_semaphores = [self.swapchain.render_finished_semaphores[self.sync_frame_index]];

        let wait_dst_stage_mask = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = [self.pipeline.command_buffers[present_index as usize]];

        //add fence to queue submit
        let submit_info = [
            vk::SubmitInfo::builder()
                .wait_semaphores(&wait_semaphores)
                .signal_semaphores(&signal_semaphores)
                .wait_dst_stage_mask(&wait_dst_stage_mask)
                .command_buffers(&command_buffers)
                .build()
        ];

        unsafe {
            self.device.logical_device
                .queue_submit(self.device.queues.graphics_queue, &submit_info, frame_fence)?;
        }

        let wait_semaphors = [self.swapchain.render_finished_semaphores[self.sync_frame_index]];
        let swapchains = [self.swapchain.swapchain];
        let image_indices = [present_index];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&wait_semaphors) // &base.rendering_complete_semaphore)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        unsafe {
            self.swapchain.swapchain_loader
                .queue_present(self.device.queues.present_queue, &present_info)?;
        }

        self.sync_frame_index = (self.sync_frame_index + 1) % MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        info!("destroying Renderer");

        unsafe {
            self.device.logical_device.device_wait_idle().unwrap();
            ManuallyDrop::drop(&mut self.pipeline);
            ManuallyDrop::drop(&mut self.swapchain);
            ManuallyDrop::drop(&mut self.skia_context);
            ManuallyDrop::drop(&mut self.device);
            ManuallyDrop::drop(&mut self.instance);
        }

        info!("destroyed Renderer");
    }
}
