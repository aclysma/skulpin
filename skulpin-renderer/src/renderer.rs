use std::ffi::CString;

// use ash::version::DeviceV1_0;
// use ash::prelude::VkResult;

use std::mem::ManuallyDrop;
// use ash::vk;

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

//use super::PresentMode;
//use super::PhysicalDeviceType;
use super::CoordinateSystemHelper;
use super::PhysicalSize;
use super::CoordinateSystem;
//use super::Window;
use rafx::api::{
    RafxFilterType, RafxAddressMode, RafxCompareOp, RafxMipMapMode, RafxSamplerDef,
    RafxImmutableSamplers, RafxImmutableSamplerKey, RafxResourceType, RafxShaderResource,
    RafxDeviceContext, RafxCommandBuffer, RafxResult, RafxApi, RafxExtents2D, RafxSwapchainDef,
    RafxSwapchainHelper, RafxQueueType, RafxCommandPoolDef, RafxCommandBufferDef,
    RafxShaderPackage, RafxShaderPackageMetal, RafxShaderPackageVulkan, RafxShaderStageDef,
    RafxShaderStageReflection, RafxShaderStageFlags, RafxVertexLayout, RafxVertexLayoutAttribute,
    RafxFormat, RafxVertexLayoutBuffer, RafxVertexAttributeRate, RafxRootSignatureDef,
    RafxGraphicsPipelineDef, RafxSampleCount, RafxPrimitiveTopology, RafxPipeline,
    RafxRootSignature, RafxCommandPool, RafxQueue,
};
use rafx::api::raw_window_handle::HasRawWindowHandle;
use std::path::Path;

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
    pipeline: RafxPipeline,
    root_signature: RafxRootSignature,
    command_buffers: Vec<RafxCommandBuffer>,
    command_pools: Vec<RafxCommandPool>,
    graphics_queue: RafxQueue,
    swapchain_helper: RafxSwapchainHelper,
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
        let mut command_pools = Vec::with_capacity(swapchain_helper.image_count());
        let mut command_buffers = Vec::with_capacity(swapchain_helper.image_count());

        for _ in 0..swapchain_helper.image_count() {
            let mut command_pool =
                graphics_queue.create_command_pool(&RafxCommandPoolDef { transient: true })?;

            let command_buffer = command_pool.create_command_buffer(&RafxCommandBufferDef {
                is_secondary: false,
            })?;

            command_pools.push(command_pool);
            command_buffers.push(command_buffer);
        }

        let mut vert_package = RafxShaderPackage {
            metal: None,
            vk: Some(RafxShaderPackageVulkan::SpvBytes(
                include_bytes!("../shaders/skia.vert.spv").to_vec(),
            )),
        };

        let mut frag_package = RafxShaderPackage {
            metal: None,
            vk: Some(RafxShaderPackageVulkan::SpvBytes(
                include_bytes!("../shaders/skia.frag.spv").to_vec(),
            )),
        };

        let vert_shader_module = device_context.create_shader_module(vert_package.module_def())?;
        let frag_shader_module = device_context.create_shader_module(frag_package.module_def())?;

        let vert_shader_stage_def = RafxShaderStageDef {
            shader_module: vert_shader_module,
            reflection: RafxShaderStageReflection {
                entry_point_name: "main".to_string(),
                shader_stage: RafxShaderStageFlags::VERTEX,
                compute_threads_per_group: None,
                resources: vec![],
            },
        };

        let frag_shader_stage_def = RafxShaderStageDef {
            shader_module: frag_shader_module,
            reflection: RafxShaderStageReflection {
                entry_point_name: "main".to_string(),
                shader_stage: RafxShaderStageFlags::FRAGMENT,
                compute_threads_per_group: None,
                resources: vec![RafxShaderResource {
                    name: Some("texSampler".to_string()),
                    set_index: 0,
                    binding: 0,
                    resource_type: RafxResourceType::COMBINED_IMAGE_SAMPLER,
                    ..Default::default()
                }],
            },
        };

        let shader =
            device_context.create_shader(vec![vert_shader_stage_def, frag_shader_stage_def])?;

        let sampler = device_context.create_sampler(&RafxSamplerDef {
            mag_filter: RafxFilterType::Linear,
            min_filter: RafxFilterType::Linear,
            address_mode_u: RafxAddressMode::Mirror,
            address_mode_v: RafxAddressMode::Mirror,
            address_mode_w: RafxAddressMode::Mirror,
            compare_op: RafxCompareOp::Never,
            mip_map_mode: RafxMipMapMode::Linear,
            max_anisotropy: 1.0,
            mip_lod_bias: 0.0,
        })?;

        let root_signature = device_context.create_root_signature(&RafxRootSignatureDef {
            shaders: &[shader.clone()],
            immutable_samplers: &[RafxImmutableSamplers {
                key: RafxImmutableSamplerKey::from_binding(0, 0),
                samplers: &[sampler],
            }],
        })?;

        let vertex_layout = RafxVertexLayout {
            attributes: vec![
                RafxVertexLayoutAttribute {
                    format: RafxFormat::R32G32_SFLOAT,
                    buffer_index: 0,
                    location: 0,
                    offset: 0,
                },
                RafxVertexLayoutAttribute {
                    format: RafxFormat::R32G32_SFLOAT,
                    buffer_index: 0,
                    location: 1,
                    offset: 8,
                },
            ],
            buffers: vec![RafxVertexLayoutBuffer {
                stride: 16,
                rate: RafxVertexAttributeRate::Vertex,
            }],
        };

        let pipeline = device_context.create_graphics_pipeline(&RafxGraphicsPipelineDef {
            shader: &shader,
            root_signature: &root_signature,
            vertex_layout: &vertex_layout,
            blend_state: &Default::default(),
            depth_state: &Default::default(),
            rasterizer_state: &Default::default(),
            color_formats: &[swapchain_helper.format()],
            sample_count: RafxSampleCount::SampleCount1,
            depth_stencil_format: None,
            primitive_topology: RafxPrimitiveTopology::TriangleStrip,
        })?;

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
            swapchain_helper,
            graphics_queue,
            command_pools,
            command_buffers,
            root_signature,
            pipeline,
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

        self.command_pools[frame.rotating_frame_index()].reset_command_pool()?;
        let command_buffer = &self.command_buffers[frame.rotating_frame_index()];
        command_buffer.begin()?;
        command_buffer.end()?;

        frame.present(&self.graphics_queue, &[&command_buffer])?;

        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        debug!("destroying Renderer");
        self.graphics_queue.wait_for_queue_idle();
        debug!("destroyed Renderer");
    }
}
