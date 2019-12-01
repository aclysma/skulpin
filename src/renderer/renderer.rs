use std::ffi::CString;

use ash::version::DeviceV1_0;
use ash::prelude::VkResult;

use std::mem::ManuallyDrop;
use ash::vk;

use super::VkInstance;
use super::CreateInstanceError;
use super::VkDevice;
use super::VkSkiaContext;
use super::VkSwapchain;
use super::VkPipeline;
use super::MAX_FRAMES_IN_FLIGHT;
use super::PresentMode;
use super::PhysicalDeviceType;

#[derive(Default)]
pub struct RendererBuilder {
    app_name: CString,
    validation_layer_debug_report_flags: vk::DebugReportFlagsEXT,
    present_mode_priority: Vec<PresentMode>,
    physical_device_type_priority: Vec<PhysicalDeviceType>,
}

impl RendererBuilder {
    pub fn new() -> Self {
        RendererBuilder {
            app_name: CString::new("Skulpin").unwrap(),
            validation_layer_debug_report_flags: vk::DebugReportFlagsEXT::all(),
            present_mode_priority: vec![PresentMode::Fifo],
            physical_device_type_priority: vec![
                PhysicalDeviceType::DiscreteGpu,
                PhysicalDeviceType::IntegratedGpu,
            ],
        }
    }

    pub fn app_name(
        mut self,
        app_name: CString,
    ) -> Self {
        self.app_name = app_name;
        self
    }

    pub fn use_vulkan_debug_layer(
        self,
        use_vulkan_debug_layer: bool,
    ) -> RendererBuilder {
        self.validation_layer_debug_report_flags(if use_vulkan_debug_layer {
            vk::DebugReportFlagsEXT::empty()
        } else {
            vk::DebugReportFlagsEXT::all()
        })
    }

    pub fn validation_layer_debug_report_flags(
        mut self,
        validation_layer_debug_report_flags: vk::DebugReportFlagsEXT,
    ) -> RendererBuilder {
        self.validation_layer_debug_report_flags = validation_layer_debug_report_flags;
        self
    }

    pub fn present_mode_priority(
        mut self,
        present_mode_priority: Vec<PresentMode>,
    ) -> RendererBuilder {
        self.present_mode_priority = present_mode_priority;
        self
    }

    pub fn physical_device_type_priority(
        mut self,
        physical_device_type_priority: Vec<PhysicalDeviceType>,
    ) -> RendererBuilder {
        self.physical_device_type_priority = physical_device_type_priority;
        self
    }

    pub fn prefer_integrated_gpu(self) -> RendererBuilder {
        self.physical_device_type_priority(vec![
            PhysicalDeviceType::IntegratedGpu,
            PhysicalDeviceType::DiscreteGpu,
        ])
    }

    pub fn prefer_discrete_gpu(self) -> RendererBuilder {
        self.physical_device_type_priority(vec![
            PhysicalDeviceType::DiscreteGpu,
            PhysicalDeviceType::IntegratedGpu,
        ])
    }

    pub fn prefer_fifo_present_mode(self) -> RendererBuilder {
        self.present_mode_priority(vec![PresentMode::Fifo])
    }

    pub fn prefer_mailbox_present_mode(self) -> RendererBuilder {
        self.present_mode_priority(vec![PresentMode::Mailbox, PresentMode::Fifo])
    }

    pub fn build(
        &self,
        window: &winit::window::Window,
    ) -> Result<Renderer, CreateRendererError> {
        Renderer::new(
            &self.app_name,
            window,
            self.validation_layer_debug_report_flags,
            self.physical_device_type_priority.clone(),
            self.present_mode_priority.clone(),
        )
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

    present_mode_priority: Vec<PresentMode>,
}

#[derive(Debug)]
pub enum CreateRendererError {
    CreateInstanceError(CreateInstanceError),
    VkError(vk::Result),
}

impl std::error::Error for CreateRendererError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            CreateRendererError::CreateInstanceError(ref e) => Some(e),
            CreateRendererError::VkError(ref e) => Some(e),
        }
    }
}

impl core::fmt::Display for CreateRendererError {
    fn fmt(
        &self,
        fmt: &mut core::fmt::Formatter,
    ) -> core::fmt::Result {
        match *self {
            CreateRendererError::CreateInstanceError(ref e) => e.fmt(fmt),
            CreateRendererError::VkError(ref e) => e.fmt(fmt),
        }
    }
}

impl From<CreateInstanceError> for CreateRendererError {
    fn from(result: CreateInstanceError) -> Self {
        CreateRendererError::CreateInstanceError(result)
    }
}

impl From<vk::Result> for CreateRendererError {
    fn from(result: vk::Result) -> Self {
        CreateRendererError::VkError(result)
    }
}

impl Renderer {
    pub fn new(
        app_name: &CString,
        window: &winit::window::Window,
        validation_layer_debug_report_flags: vk::DebugReportFlagsEXT,
        physical_device_type_priority: Vec<PhysicalDeviceType>,
        present_mode_priority: Vec<PresentMode>,
    ) -> Result<Renderer, CreateRendererError> {
        let instance = ManuallyDrop::new(VkInstance::new(
            app_name,
            validation_layer_debug_report_flags,
        )?);
        let device = ManuallyDrop::new(VkDevice::new(
            &instance,
            window,
            &physical_device_type_priority,
        )?);
        let mut skia_context = ManuallyDrop::new(VkSkiaContext::new(&instance, &device));
        let swapchain = ManuallyDrop::new(VkSwapchain::new(
            &instance,
            &device,
            window,
            None,
            &present_mode_priority,
        )?);
        let pipeline = ManuallyDrop::new(VkPipeline::new(&device, &swapchain, &mut skia_context)?);
        let sync_frame_index = 0;

        Ok(Renderer {
            instance,
            device,
            skia_context,
            swapchain,
            pipeline,
            sync_frame_index,
            present_mode_priority,
        })
    }

    pub fn draw<F: FnOnce(&mut skia_safe::Canvas)>(
        &mut self,
        window: &winit::window::Window,
        f: F,
    ) -> VkResult<()> {
        let result = self.do_draw(window, f);
        if let Err(e) = result {
            match e {
                ash::vk::Result::ERROR_OUT_OF_DATE_KHR => {
                    unsafe {
                        self.device.logical_device.device_wait_idle()?;
                        ManuallyDrop::drop(&mut self.pipeline);
                    }

                    let new_swapchain = ManuallyDrop::new(VkSwapchain::new(
                        &self.instance,
                        &self.device,
                        window,
                        Some(self.swapchain.swapchain),
                        &self.present_mode_priority,
                    )?);

                    unsafe {
                        ManuallyDrop::drop(&mut self.swapchain);
                    }

                    self.swapchain = new_swapchain;
                    self.pipeline = ManuallyDrop::new(VkPipeline::new(
                        &self.device,
                        &self.swapchain,
                        &mut self.skia_context,
                    )?);
                    Ok(())
                }
                ash::vk::Result::SUCCESS => Ok(()),
                ash::vk::Result::SUBOPTIMAL_KHR => Ok(()),
                _ => {
                    warn!("Unexpected rendering error");
                    Err(e)
                }
            }
        } else {
            Ok(())
        }
    }

    fn do_draw<F: FnOnce(&mut skia_safe::Canvas)>(
        &mut self,
        window: &winit::window::Window,
        f: F,
    ) -> VkResult<()> {
        let frame_fence = self.swapchain.in_flight_fences[self.sync_frame_index];

        //TODO: Dont lock up forever (don't use std::u64::MAX)
        //TODO: Can part of this run in a separate thread from the window pump?
        //TODO: Explore an option that ensures we receive the same skia canvas back every draw call.
        // This may require a copy from a surface that is not use in the swapchain into one that is

        // Wait if two frame are already in flight
        unsafe {
            self.device
                .logical_device
                .wait_for_fences(&[frame_fence], true, std::u64::MAX)?;
            self.device.logical_device.reset_fences(&[frame_fence])?;
        }

        let (present_index, _is_suboptimal) = unsafe {
            self.swapchain.swapchain_loader.acquire_next_image(
                self.swapchain.swapchain,
                std::u64::MAX,
                self.swapchain.image_available_semaphores[self.sync_frame_index],
                vk::Fence::null(),
            )?
        };

        {
            let surface = self.pipeline.skia_surface(present_index as usize);
            let mut canvas = surface.surface.canvas();

            // To handle hi-dpi displays, we need to compare the logical size of the window with the
            // actual canvas size. Critically, the canvas size won't necessarily be the size of the
            // window in physical pixels.
            let window_size = window.inner_size();
            let scale = (
                (f64::from(self.swapchain.swapchain_info.extents.width) / window_size.width) as f32,
                (f64::from(self.swapchain.swapchain_info.extents.height) / window_size.height)
                    as f32,
            );

            canvas.reset_matrix();
            canvas.scale(scale);

            f(&mut canvas);

            canvas.flush();
        }

        let wait_semaphores = [self.swapchain.image_available_semaphores[self.sync_frame_index]];
        let signal_semaphores = [self.swapchain.render_finished_semaphores[self.sync_frame_index]];

        let wait_dst_stage_mask = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = [self.pipeline.command_buffers[present_index as usize]];

        //add fence to queue submit
        let submit_info = [vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .signal_semaphores(&signal_semaphores)
            .wait_dst_stage_mask(&wait_dst_stage_mask)
            .command_buffers(&command_buffers)
            .build()];

        unsafe {
            self.device.logical_device.queue_submit(
                self.device.queues.graphics_queue,
                &submit_info,
                frame_fence,
            )?;
        }

        let wait_semaphors = [self.swapchain.render_finished_semaphores[self.sync_frame_index]];
        let swapchains = [self.swapchain.swapchain];
        let image_indices = [present_index];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&wait_semaphors) // &base.rendering_complete_semaphore)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        unsafe {
            self.swapchain
                .swapchain_loader
                .queue_present(self.device.queues.present_queue, &present_info)?;
        }

        self.sync_frame_index = (self.sync_frame_index + 1) % MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        debug!("destroying Renderer");

        unsafe {
            self.device.logical_device.device_wait_idle().unwrap();
            ManuallyDrop::drop(&mut self.pipeline);
            ManuallyDrop::drop(&mut self.swapchain);
            ManuallyDrop::drop(&mut self.skia_context);
            ManuallyDrop::drop(&mut self.device);
            ManuallyDrop::drop(&mut self.instance);
        }

        debug!("destroyed Renderer");
    }
}
