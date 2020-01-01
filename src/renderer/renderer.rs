use std::ffi::CString;

use ash::version::DeviceV1_0;
use ash::prelude::VkResult;

use std::mem::ManuallyDrop;
use ash::vk;

use super::VkInstance;
use super::VkCreateInstanceError;
use super::VkDevice;
use super::VkSkiaContext;
use super::VkSwapchain;
use super::VkSkiaRenderPass;

#[cfg(feature = "with_imgui")]
use super::VkImGuiRenderPass;
use super::MAX_FRAMES_IN_FLIGHT;
use super::PresentMode;
use super::PhysicalDeviceType;
use super::CoordinateSystemHelper;
use winit::dpi::PhysicalSize;
use crate::CoordinateSystem;

#[cfg(feature = "with_imgui")]
use crate::renderer::imgui_support::ImguiManager;

/// A builder to create the renderer. It's easier to use AppBuilder and implement an AppHandler, but
/// initializing the renderer and maintaining the window yourself allows for more customization
#[derive(Default)]
pub struct RendererBuilder {
    app_name: CString,
    validation_layer_debug_report_flags: vk::DebugReportFlagsEXT,
    present_mode_priority: Vec<PresentMode>,
    physical_device_type_priority: Vec<PhysicalDeviceType>,
    coordinate_system: CoordinateSystem,
}

impl RendererBuilder {
    /// Construct the renderer builder with default options
    pub fn new() -> Self {
        RendererBuilder {
            app_name: CString::new("Skulpin").unwrap(),
            validation_layer_debug_report_flags: vk::DebugReportFlagsEXT::all(),
            present_mode_priority: vec![PresentMode::Fifo],
            physical_device_type_priority: vec![
                PhysicalDeviceType::DiscreteGpu,
                PhysicalDeviceType::IntegratedGpu,
            ],
            coordinate_system: Default::default(),
        }
    }

    /// Name of the app. This is passed into the vulkan layer. I believe it can hint things to the
    /// vulkan driver, but it's unlikely this makes a real difference. Still a good idea to set this
    /// to something meaningful though.
    pub fn app_name(
        mut self,
        app_name: CString,
    ) -> Self {
        self.app_name = app_name;
        self
    }

    /// If true, initialize the vulkan debug layers. This will require the vulkan SDK to be
    /// installed and the app will fail to launch if it isn't. This turns on ALL logging. For
    /// more control, see `validation_layer_debug_report_flags()`
    pub fn use_vulkan_debug_layer(
        self,
        use_vulkan_debug_layer: bool,
    ) -> Self {
        self.validation_layer_debug_report_flags(if use_vulkan_debug_layer {
            vk::DebugReportFlagsEXT::empty()
        } else {
            vk::DebugReportFlagsEXT::all()
        })
    }

    /// Sets the desired debug layer flags. If any flag is set, the vulkan debug layers will be
    /// loaded, which requires the Vulkan SDK to be installed. The app will fail to launch if it
    /// isn't.
    pub fn validation_layer_debug_report_flags(
        mut self,
        validation_layer_debug_report_flags: vk::DebugReportFlagsEXT,
    ) -> Self {
        self.validation_layer_debug_report_flags = validation_layer_debug_report_flags;
        self
    }

    /// Determine the coordinate system to use for the canvas. This can be overridden by using the
    /// canvas sizer passed into the draw callback
    pub fn coordinate_system(
        mut self,
        coordinate_system: CoordinateSystem,
    ) -> Self {
        self.coordinate_system = coordinate_system;
        self
    }

    /// Specify which PresentMode is preferred. Some of this is hardware/platform dependent and
    /// it's a good idea to read the Vulkan spec. You
    ///
    /// `present_mode_priority` should be a list of desired present modes, in descending order of
    /// preference. In other words, passing `[Mailbox, Fifo]` will direct Skulpin to use mailbox
    /// where available, but otherwise use `Fifo`.
    ///
    /// Since `Fifo` is always available, this is the mode that will be chosen if no desired mode is
    /// available.
    pub fn present_mode_priority(
        mut self,
        present_mode_priority: Vec<PresentMode>,
    ) -> Self {
        self.present_mode_priority = present_mode_priority;
        self
    }

    /// Specify which type of physical device is preferred. It's recommended to read the Vulkan spec
    /// to understand precisely what these types mean
    ///
    /// `physical_device_type_priority` should be a list of desired present modes, in descending
    /// order of preference. In other words, passing `[Discrete, Integrated]` will direct Skulpin to
    /// use the discrete GPU where available, otherwise integrated.
    ///
    /// If the desired device type can't be found, Skulpin will try to use whatever device is
    /// available. By default `Discrete` is favored, then `Integrated`, then anything that's
    /// available. It could make sense to favor `Integrated` over `Discrete` when minimizing
    /// power consumption is important. (Although I haven't tested this myself)
    pub fn physical_device_type_priority(
        mut self,
        physical_device_type_priority: Vec<PhysicalDeviceType>,
    ) -> Self {
        self.physical_device_type_priority = physical_device_type_priority;
        self
    }

    /// Easy shortcut to set device type priority to `Integrated`, then `Discrete`, then any.
    pub fn prefer_integrated_gpu(self) -> Self {
        self.physical_device_type_priority(vec![
            PhysicalDeviceType::IntegratedGpu,
            PhysicalDeviceType::DiscreteGpu,
        ])
    }

    /// Easy shortcut to set device type priority to `Discrete`, then `Integrated`, than any.
    /// (This is the default behavior)
    pub fn prefer_discrete_gpu(self) -> Self {
        self.physical_device_type_priority(vec![
            PhysicalDeviceType::DiscreteGpu,
            PhysicalDeviceType::IntegratedGpu,
        ])
    }

    /// Prefer using `Fifo` presentation mode. This presentation mode is always available on a
    /// device that complies with the vulkan spec.
    pub fn prefer_fifo_present_mode(self) -> Self {
        self.present_mode_priority(vec![PresentMode::Fifo])
    }

    /// Prefer using `Mailbox` presentation mode, and fall back to `Fifo` when not available.
    pub fn prefer_mailbox_present_mode(self) -> Self {
        self.present_mode_priority(vec![PresentMode::Mailbox, PresentMode::Fifo])
    }

    /// Builds the renderer. The window that's passed in will be used for creating the swapchain
    pub fn build(
        &self,
        window: &winit::window::Window,
        #[cfg(feature = "with_imgui")] imgui_manager: &mut ImguiManager
    ) -> Result<Renderer, CreateRendererError> {
        Renderer::new(
            &self.app_name,
            window,
            #[cfg(feature = "with_imgui")] imgui_manager,
            self.validation_layer_debug_report_flags,
            self.physical_device_type_priority.clone(),
            self.present_mode_priority.clone(),
            self.coordinate_system,
        )
    }
}

/// Vulkan renderer that creates and manages the vulkan instance, device, swapchain, and
/// render passes.
pub struct Renderer {
    instance: ManuallyDrop<VkInstance>,
    device: ManuallyDrop<VkDevice>,

    skia_context: ManuallyDrop<VkSkiaContext>,

    swapchain: ManuallyDrop<VkSwapchain>,
    skia_renderpass: ManuallyDrop<VkSkiaRenderPass>,
    #[cfg(feature = "with_imgui")] imgui_renderpass: ManuallyDrop<VkImGuiRenderPass>,

    // Increase until > MAX_FRAMES_IN_FLIGHT, then set to 0, or -1 if no frame drawn yet
    sync_frame_index: usize,

    present_mode_priority: Vec<PresentMode>,

    previous_inner_size: PhysicalSize<u32>,

    coordinate_system: CoordinateSystem,
}

/// Represents an error from creating the renderer
#[derive(Debug)]
pub enum CreateRendererError {
    CreateInstanceError(VkCreateInstanceError),
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

impl From<VkCreateInstanceError> for CreateRendererError {
    fn from(result: VkCreateInstanceError) -> Self {
        CreateRendererError::CreateInstanceError(result)
    }
}

impl From<vk::Result> for CreateRendererError {
    fn from(result: vk::Result) -> Self {
        CreateRendererError::VkError(result)
    }
}

impl Renderer {
    /// Create the renderer
    pub fn new(
        app_name: &CString,
        window: &winit::window::Window,
        #[cfg(feature = "with_imgui")] imgui_manager: &mut ImguiManager,
        validation_layer_debug_report_flags: vk::DebugReportFlagsEXT,
        physical_device_type_priority: Vec<PhysicalDeviceType>,
        present_mode_priority: Vec<PresentMode>,
        coordinate_system: CoordinateSystem,
    ) -> Result<Renderer, CreateRendererError> {
        let instance = ManuallyDrop::new(VkInstance::new(
            window,
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
        let skia_renderpass = ManuallyDrop::new(VkSkiaRenderPass::new(
            &device,
            &swapchain,
            &mut skia_context,
        )?);

        #[cfg(feature = "with_imgui")]
        let imgui_renderpass = ManuallyDrop::new(VkImGuiRenderPass::new(&device, &swapchain, imgui_manager)?);

        let sync_frame_index = 0;

        let previous_inner_size = window.inner_size();

        Ok(Renderer {
            instance,
            device,
            skia_context,
            swapchain,
            skia_renderpass,
            #[cfg(feature = "with_imgui")] imgui_renderpass,
            sync_frame_index,
            present_mode_priority,
            previous_inner_size,
            coordinate_system,
        })
    }

    /// Call to render a frame. This can block for certain presentation modes. This will rebuild
    /// the swapchain if necessary.
    pub fn draw<
        #[cfg(feature = "with_imgui")] F: FnOnce(&mut skia_safe::Canvas, &CoordinateSystemHelper, &mut ImguiManager),
        #[cfg(not(feature = "with_imgui"))] F: FnOnce(&mut skia_safe::Canvas, &CoordinateSystemHelper)
    >(
        &mut self,
        window: &winit::window::Window,
        #[cfg(feature = "with_imgui")] imgui_manager: &mut ImguiManager,
        f: F,
    ) -> VkResult<()> {
        if window.inner_size() != self.previous_inner_size {
            debug!("Detected window inner size change, rebuilding swapchain");
            self.rebuild_swapchain(window, #[cfg(feature = "with_imgui")] imgui_manager)?;
        }

        let result = self.do_draw(window, #[cfg(feature = "with_imgui")] imgui_manager, f);
        if let Err(e) = result {
            match e {
                ash::vk::Result::ERROR_OUT_OF_DATE_KHR => self.rebuild_swapchain(window, #[cfg(feature = "with_imgui")] imgui_manager),
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

    fn rebuild_swapchain(
        &mut self,
        window: &winit::window::Window,
        #[cfg(feature = "with_imgui")] imgui_manager: &mut ImguiManager,
    ) -> VkResult<()> {
        unsafe {
            self.device.logical_device.device_wait_idle()?;
            ManuallyDrop::drop(&mut self.skia_renderpass);

            #[cfg(feature = "with_imgui")]
            ManuallyDrop::drop(&mut self.imgui_renderpass);
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
        self.skia_renderpass = ManuallyDrop::new(VkSkiaRenderPass::new(
            &self.device,
            &self.swapchain,
            &mut self.skia_context,
        )?);

        #[cfg(feature = "with_imgui")]
        {
            self.imgui_renderpass = ManuallyDrop::new(VkImGuiRenderPass::new(&self.device, &self.swapchain, imgui_manager)?);
        }

        self.previous_inner_size = window.inner_size();

        Ok(())
    }

    /// Do the render
    fn do_draw<
        #[cfg(feature = "with_imgui")] F: FnOnce(&mut skia_safe::Canvas, &CoordinateSystemHelper, &mut ImguiManager),
        #[cfg(not(feature = "with_imgui"))] F: FnOnce(&mut skia_safe::Canvas, &CoordinateSystemHelper)
    >(
        &mut self,
        window: &winit::window::Window,
        #[cfg(feature = "with_imgui")] imgui_manager: &mut ImguiManager,
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
            let surface = self.skia_renderpass.skia_surface(present_index as usize);
            let mut canvas = surface.surface.canvas();

            let surface_extents = self.swapchain.swapchain_info.extents;
            let window_physical_size = window.inner_size();
            let scale_factor = window.scale_factor();
            let window_logical_size = window_physical_size.to_logical(scale_factor);

            let coordinate_system_helper = CoordinateSystemHelper::new(
                surface_extents,
                window_logical_size,
                window_physical_size,
                scale_factor,
            );

            match self.coordinate_system {
                CoordinateSystem::None => {}
                CoordinateSystem::Physical => {
                    coordinate_system_helper.use_physical_coordinates(&mut canvas)
                }
                CoordinateSystem::Logical => {
                    coordinate_system_helper.use_logical_coordinates(&mut canvas)
                }
                CoordinateSystem::VisibleRange(range, scale_to_fit) => coordinate_system_helper
                    .use_visible_range(&mut canvas, range, scale_to_fit)
                    .unwrap(),
                CoordinateSystem::FixedWidth(center, x_half_extents) => coordinate_system_helper
                    .use_fixed_width(&mut canvas, center, x_half_extents)
                    .unwrap(),
            }

            f(&mut canvas, &coordinate_system_helper, #[cfg(feature = "with_imgui")] imgui_manager);

            canvas.flush();
        }

        #[cfg(feature = "with_imgui")]
        {
            imgui_manager.render(window);
            let imgui_draw_data : Option<&imgui::DrawData> = imgui_manager.draw_data();

            self.imgui_renderpass.update(
                &self.device.memory_properties,
                imgui_draw_data,
                present_index as usize,
                window.hidpi_factor()
            )?;

            imgui_manager.begin_frame(window);
        }

        let wait_semaphores = [self.swapchain.image_available_semaphores[self.sync_frame_index]];
        let signal_semaphores = [self.swapchain.render_finished_semaphores[self.sync_frame_index]];

        let wait_dst_stage_mask = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        #[cfg(not(feature = "with_imgui"))]
        let command_buffers = vec![self.skia_renderpass.command_buffers[present_index as usize]];

        #[cfg(feature = "with_imgui")]
        let command_buffers = vec![
            self.skia_renderpass.command_buffers[present_index as usize],
            self.imgui_renderpass.command_buffers[present_index as usize]
        ];

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
            ManuallyDrop::drop(&mut self.skia_renderpass);

            #[cfg(feature = "with_imgui")]
            ManuallyDrop::drop(&mut self.imgui_renderpass);

            ManuallyDrop::drop(&mut self.swapchain);
            ManuallyDrop::drop(&mut self.skia_context);
            ManuallyDrop::drop(&mut self.device);
            ManuallyDrop::drop(&mut self.instance);
        }

        debug!("destroyed Renderer");
    }
}
