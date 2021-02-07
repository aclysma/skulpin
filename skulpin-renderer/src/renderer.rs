use rafx::api::*;
use rafx::nodes::*;
use rafx::framework::*;

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

use super::CoordinateSystemHelper;
use super::PhysicalSize;
use super::CoordinateSystem;
use rafx::api::raw_window_handle::HasRawWindowHandle;
use std::path::Path;
use std::sync::Arc;

/// May be implemented to get callbacks related to the renderer and framebuffer usage
pub trait RendererPlugin {
    // /// Called whenever the swapchain needs to be created (the first time, and in cases where the
    // /// swapchain needs to be recreated)
    // fn swapchain_created(
    //     &mut self,
    //     device: &RafxDeviceContext,
    //     //swapchain: &Swapchain,
    // ) -> RafxResult<()>;
    //
    // /// Called whenever the swapchain will be destroyed (when renderer is dropped, and also in cases
    // /// where the swapchain needs to be recreated)
    // fn swapchain_destroyed(&mut self);

    /// Called when we are presenting a new frame. The returned command buffer will be submitted
    /// with command buffers for the skia canvas
    fn render(
        &mut self,
        window: &dyn HasRawWindowHandle,
        device: &RafxDeviceContext,
        command_buffer: &RafxCommandBuffer,
        present_index: usize,
    ) -> RafxResult<()>;
}

/// A builder to create the renderer. It's easier to use AppBuilder and implement an AppHandler, but
/// initializing the renderer and maintaining the window yourself allows for more customization
#[derive(Default)]
pub struct RendererBuilder {
    coordinate_system: CoordinateSystem,
    plugins: Vec<Box<dyn RendererPlugin>>,
}

impl RendererBuilder {
    /// Construct the renderer builder with default options
    pub fn new() -> Self {
        RendererBuilder {
            coordinate_system: Default::default(),
            plugins: vec![],
        }
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

    pub fn add_plugin(
        mut self,
        plugin: Box<dyn RendererPlugin>,
    ) -> Self {
        self.plugins.push(plugin);
        self
    }

    /// Builds the renderer. The window that's passed in will be used for creating the swapchain
    pub fn build(
        self,
        window: &dyn HasRawWindowHandle,
        window_size: RafxExtents2D,
    ) -> RafxResult<Renderer> {
        Renderer::new(window, window_size, self.coordinate_system, self.plugins)
    }
}

/// Vulkan renderer that creates and manages the vulkan instance, device, swapchain, and
/// render passes.
pub struct Renderer {
    // instance: ManuallyDrop<VkInstance>,
    // device: ManuallyDrop<VkDevice>,
    //
    // skia_context: ManuallyDrop<VkSkiaContext>,
    //
    // swapchain: ManuallyDrop<VkSwapchain>,
    // skia_renderpass: ManuallyDrop<VkSkiaRenderPass>,
    //
    // // Increase until > MAX_FRAMES_IN_FLIGHT, then set to 0, or -1 if no frame drawn yet
    // sync_frame_index: usize,
    //
    // present_mode_priority: Vec<PresentMode>,
    //
    // previous_inner_size: PhysicalSize,

    // Ordered in drop order
    skia_material_pass: MaterialPass,
    graphics_queue: RafxQueue,
    swapchain_helper: RafxSwapchainHelper,
    resource_manager: ResourceManager,
    api: RafxApi,

    coordinate_system: CoordinateSystem,

    plugins: Vec<Box<dyn RendererPlugin>>,
}

impl Renderer {
    /// Create the renderer
    pub fn new(
        window: &dyn HasRawWindowHandle,
        window_size: RafxExtents2D,
        coordinate_system: CoordinateSystem,
        mut plugins: Vec<Box<dyn RendererPlugin>>,
    ) -> RafxResult<Renderer> {
        let mut api = RafxApi::new(window, &Default::default())?;
        let device_context = api.device_context();

        let render_registry = RenderRegistryBuilder::default()
            .register_render_phase::<OpaqueRenderPhase>("opaque")
            .build();
        let resource_manager =
            rafx::framework::ResourceManager::new(&device_context, &render_registry);

        let swapchain = device_context.create_swapchain(
            window,
            &RafxSwapchainDef {
                width: window_size.width,
                height: window_size.height,
                enable_vsync: true,
            },
        )?;
        let mut swapchain_helper = RafxSwapchainHelper::new(&device_context, swapchain, None)?;
        let graphics_queue = device_context.create_queue(RafxQueueType::Graphics)?;

        let resource_context = resource_manager.resource_context();

        let skia_material_pass = Self::load_material_pass(
            &resource_context,
            include_bytes!("../shaders/out/skia.vert.cookedshaderpackage"),
            include_bytes!("../shaders/out/skia.frag.cookedshaderpackage"),
            FixedFunctionState {
                rasterizer_state: Default::default(),
                depth_state: Default::default(),
                blend_state: Default::default(),
            },
        )?;


        //let mut skia_context = ManuallyDrop::new(VkSkiaContext::new(&instance, &device));
        // let swapchain = ManuallyDrop::new(VkSwapchain::new(
        //     &instance,
        //     &device,
        //     window,
        //     None,
        //     &present_mode_priority,
        // )?);
        // let skia_renderpass = ManuallyDrop::new(VkSkiaRenderPass::new(
        //     &device,
        //     &swapchain,
        //     &mut skia_context,
        // )?);
        //
        // for plugin in &mut plugins {
        //     plugin.swapchain_created(&device, &swapchain)?;
        // }

        Ok(Renderer {
            api,
            resource_manager,
            swapchain_helper,
            graphics_queue,
            skia_material_pass,
            coordinate_system,
            plugins,
        })
    }

    // pub fn skia_context(&self) -> &skia_safe::gpu::Context {
    //     &self.skia_context.context
    // }

    /// Call to render a frame. This can block for certain presentation modes. This will rebuild
    /// the swapchain if necessary.
    pub fn draw<F: FnOnce(&mut skia_safe::Canvas, CoordinateSystemHelper)>(
        &mut self,
        window: &dyn HasRawWindowHandle,
        window_size: RafxExtents2D,
        f: F,
    ) -> RafxResult<()> {
        let frame = self.swapchain_helper.acquire_next_image(
            window_size.width,
            window_size.height,
            None,
        )?;

        self.resource_manager.on_frame_complete()?;

        let mut command_pool = self
            .resource_manager
            .dyn_command_pool_allocator()
            .allocate_dyn_pool(
                &self.graphics_queue,
                &RafxCommandPoolDef { transient: false },
                0,
            )?;

        let command_buffer = command_pool.allocate_dyn_command_buffer(&RafxCommandBufferDef {
            is_secondary: false,
        })?;

        command_buffer.begin()?;

        command_buffer.cmd_resource_barrier(
            &[],
            &[RafxTextureBarrier {
                texture: frame.swapchain_texture(),
                array_slice: None,
                mip_slice: None,
                src_state: RafxResourceState::PRESENT,
                dst_state: RafxResourceState::RENDER_TARGET,
                queue_transition: RafxBarrierQueueTransition::None,
            }],
        )?;

        command_buffer.cmd_begin_render_pass(
            &[RafxColorRenderTargetBinding {
                texture: frame.swapchain_texture(),
                load_op: RafxLoadOp::Clear,
                store_op: RafxStoreOp::Store,
                clear_value: RafxColorClearValue([0.0, 0.0, 0.0, 0.0]),
                mip_slice: Default::default(),
                array_slice: Default::default(),
                resolve_target: Default::default(),
                resolve_store_op: Default::default(),
                resolve_mip_slice: Default::default(),
                resolve_array_slice: Default::default(),
            }],
            None,
        )?;

        // self.draw_debug(
        //     &command_buffer,
        //     example_inspect_target,
        //     window.scale_factor() as f32,
        // )?;
        // self.draw_imgui(&command_buffer, imgui_draw_data)?;

        command_buffer.cmd_end_render_pass()?;

        command_buffer.cmd_resource_barrier(
            &[],
            &[RafxTextureBarrier {
                texture: frame.swapchain_texture(),
                array_slice: None,
                mip_slice: None,
                src_state: RafxResourceState::RENDER_TARGET,
                dst_state: RafxResourceState::PRESENT,
                queue_transition: RafxBarrierQueueTransition::None,
            }],
        )?;

        command_buffer.end()?;

        frame.present(&self.graphics_queue, &[&command_buffer])?;

        Ok(())
    }

    fn load_material_pass(
        resource_context: &ResourceContext,
        cooked_vertex_shader_bytes: &[u8],
        cooked_fragment_shader_bytes: &[u8],
        fixed_function_state: FixedFunctionState,
    ) -> RafxResult<MaterialPass> {
        let cooked_vertex_shader_stage =
            bincode::deserialize::<CookedShaderPackage>(cooked_vertex_shader_bytes)
                .map_err(|x| format!("Failed to deserialize cooked shader: {:?}", x))?;
        let vertex_shader_module = resource_context
            .resources()
            .get_or_create_shader_module_from_cooked_package(&cooked_vertex_shader_stage)?;
        let vertex_entry_point = cooked_vertex_shader_stage
            .find_entry_point("main")
            .unwrap()
            .clone();

        // Create the fragment shader module and find the entry point
        let cooked_fragment_shader_stage =
            bincode::deserialize::<CookedShaderPackage>(cooked_fragment_shader_bytes)
                .map_err(|x| format!("Failed to deserialize cooked shader: {:?}", x))?;
        let fragment_shader_module = resource_context
            .resources()
            .get_or_create_shader_module_from_cooked_package(&cooked_fragment_shader_stage)?;
        let fragment_entry_point = cooked_fragment_shader_stage
            .find_entry_point("main")
            .unwrap()
            .clone();

        let fixed_function_state = Arc::new(fixed_function_state);

        let material_pass = MaterialPass::new(
            &resource_context,
            fixed_function_state,
            vec![vertex_shader_module, fragment_shader_module],
            &[&vertex_entry_point, &fragment_entry_point],
        )?;

        Ok(material_pass)
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        debug!("destroying Renderer");
        self.graphics_queue.wait_for_queue_idle().unwrap();
        debug!("destroyed Renderer");
    }
}


rafx::nodes::declare_render_phase!(
    OpaqueRenderPhase,
    OPAQUE_RENDER_PHASE_INDEX,
    opaque_render_phase_sort_submit_nodes
);

fn opaque_render_phase_sort_submit_nodes(submit_nodes: Vec<SubmitNode>) -> Vec<SubmitNode> {
    // No sort needed
    submit_nodes
}