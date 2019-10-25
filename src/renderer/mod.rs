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

use super::TimeState;
use crate::InputState;

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
    pub fn new(window: &winit::window::Window) -> Renderer {
        let instance = ManuallyDrop::new(VkInstance::new());
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

    pub fn draw(
        &mut self,
        window: &winit::window::Window,
        time_state: &TimeState,
        input_state: &InputState
    ) {
        let result = self.do_draw(window, time_state, input_state);
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

    pub fn do_draw(
        &mut self,
        _window: &winit::window::Window,
        time_state: &TimeState,
        input_state: &InputState
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
            Self::draw_canvas(&mut canvas, time_state, input_state);
            canvas.flush();
        }

        self.pipeline.update_uniform_buffer(time_state, present_index, self.swapchain.swapchain_info.extents);

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

    fn draw_canvas(
        canvas: &mut skia_safe::Canvas,
        time_state: &TimeState,
        input_state: &InputState
    ) {
        canvas.clear(skia_safe::Color::from_argb(0, 0, 0, 255));

        let f = (time_state.system().frame_count % 60) as f32 / 60.0;

        let mut paint = skia_safe::Paint::new(skia_safe::Color4f::new(1.0 - f, 0.0, f, 1.0), None);
        paint.set_anti_alias(true);
        paint.set_style(skia_safe::paint::Style::Stroke);
        paint.set_stroke_width(3.0);
        //canvas.draw_line(skia_safe::Point::new(0.0, 0.0), skia_safe::Point::new(100.0, 50.0), &paint);

        canvas.draw_circle(
            skia_safe::Point::new(
                100.0 + (f * 300.0),
                50.0 + (f * 300.0)
            ),
            50.0,
            &paint);

        let rect = skia_safe::Rect {
            left: 10.0,
            top: 10.0,
            right: 500.0,
            bottom: 500.0
        };
        canvas.draw_rect(rect, &paint);

        let mut paint_green = skia_safe::Paint::new(skia_safe::Color4f::new(0.0, 1.0, 0.0, 1.0), None);
        paint_green.set_anti_alias(true);
        paint_green.set_style(skia_safe::paint::Style::Stroke);
        paint_green.set_stroke_width(3.0);

        let mouse_position_px = input_state.mouse_position();

        let rect = skia_safe::Rect {
            left: mouse_position_px.x() - 10.0,
            top: mouse_position_px.y() - 10.0,
            right: mouse_position_px.x() + 10.0,
            bottom: mouse_position_px.y() + 10.0
        };
        canvas.draw_rect(rect, &paint_green);


        //canvas.draw_text();
        //TODO: draw_str
        //TODO: draw_bitmap

        let mut font = skia_safe::Font::default();
        font.set_size(500.0);

        canvas.draw_str("Here is a string", (130, 500), &font, &paint);
        canvas.draw_str("Here is a string", (150, 500), &font, &paint);
        canvas.draw_str("Here is a string", (160, 500), &font, &paint);
        canvas.draw_str("Here is a string", (170, 500), &font, &paint);
        canvas.draw_str("Here is a string", (180, 500), &font, &paint);
        canvas.draw_str("Here is a string", (190, 500), &font, &paint);
        canvas.draw_str("Here is a string", (200, 500), &font, &paint);
        canvas.draw_str("Here is a string", (210, 500), &font, &paint);
        canvas.draw_str("Here is a string", (220, 500), &font, &paint);
        canvas.draw_str("Here is a string", (230, 500), &font, &paint);
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
