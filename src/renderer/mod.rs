mod alignment;
use alignment::Align;

use ash::version::DeviceV1_0;

pub mod util;

mod window_support;

mod instance;
pub use instance::VkInstance;

mod device;
pub use device::VkDevice;
pub use device::QueueFamilyIndices; // TODO: Should this be re-exported like this? Name is very general.
pub use device::Queues; // TODO: Should this be re-exported like this? Name is very general.

mod swapchain;
pub use swapchain::VkSwapchain;
pub use swapchain::SwapchainInfo;
pub use swapchain::MAX_FRAMES_IN_FLIGHT;

mod skia_pipeline;
pub use skia_pipeline::VkPipeline;

mod skia_support;
pub use skia_support::VkSkiaContext;

mod buffer;
pub use buffer::VkBuffer;

/*
mod image;
pub use self::image::VkImage;
*/

use std::mem::ManuallyDrop;
use ash::vk;

mod debug_reporter;
pub use debug_reporter::VkDebugReporter;

pub struct RendererBuilder {
    use_vulkan_debug_layer: bool
}

impl RendererBuilder {
    pub fn new() -> Self {
        RendererBuilder {
            use_vulkan_debug_layer: false
        }
    }

    pub fn use_vulkan_debug_layer(mut self, use_vulkan_debug_layer: bool) -> RendererBuilder {
        self.use_vulkan_debug_layer = use_vulkan_debug_layer;
        self
    }

    //TODO: Make this return a result and properly return errors
    pub fn build(&self, window: &winit::window::Window) -> Renderer {
        Renderer::new(window, self.use_vulkan_debug_layer)
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
    pub fn new(window: &winit::window::Window, use_vulkan_debug_layer: bool) -> Renderer {
        let instance = ManuallyDrop::new(VkInstance::new(use_vulkan_debug_layer));
        let device = ManuallyDrop::new(VkDevice::new(&instance, window));
        let mut skia_context = ManuallyDrop::new(VkSkiaContext::new(&instance, &device));
        let swapchain = ManuallyDrop::new(VkSwapchain::new(&instance, &device, window));
        let pipeline = ManuallyDrop::new(VkPipeline::new(&device, &swapchain, &mut skia_context));
        let sync_frame_index = 0;

        Renderer {
            instance,
            device,
            skia_context,
            swapchain,
            pipeline,
            sync_frame_index
        }
    }

    pub fn draw<F : FnOnce(&mut skia_safe::Canvas)>(
        &mut self,
        window: &winit::window::Window,
        f: F
    ) {
        let result = self.do_draw(window, f);
        if let Err(e) = result {
            match e {
                ash::vk::Result::ERROR_OUT_OF_DATE_KHR => {

                    //TODO: Clean the do_draw stuff up
                    //TODO: Replace unwrap() with something better where possible
                    //TODO: How does it work to render from another thread?
                    unsafe {
                        self.device.logical_device.device_wait_idle().unwrap();
                        ManuallyDrop::drop(&mut self.pipeline);
                        ManuallyDrop::drop(&mut self.swapchain);
                    }

                    self.swapchain = ManuallyDrop::new(VkSwapchain::new(&self.instance, &self.device, window));
                    self.pipeline = ManuallyDrop::new(VkPipeline::new(&self.device, &self.swapchain, &mut self.skia_context));

                },
                ash::vk::Result::SUCCESS => {},
                ash::vk::Result::SUBOPTIMAL_KHR => {},
                _ => {
                    panic!("Unexpected rendering error");
                }
            }
        }
    }

    fn do_draw<F : FnOnce(&mut skia_safe::Canvas)>(
        &mut self,
        _window: &winit::window::Window,
        f: F
    )
        -> Result<(), ash::vk::Result>
    {
        let frame_fence = self.swapchain.in_flight_fences[self.sync_frame_index];

        //TODO: Dont lock up forever (don't use std::u64::MAX)
        //TODO: Can part of this run in a separate thread from the window pump?

        // Wait if two frame are already in flight
        unsafe {
            self.device.logical_device.wait_for_fences(&[frame_fence], true, std::u64::MAX).unwrap();
            self.device.logical_device.reset_fences(&[frame_fence]).unwrap();
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

        self.pipeline.update_uniform_buffer(present_index, self.swapchain.swapchain_info.extents);

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
                .queue_submit(self.device.queues.graphics_queue, &submit_info, frame_fence)
                .expect("queue submit failed.");
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
                .queue_present(self.device.queues.present_queue, &present_info)
                .unwrap();
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
