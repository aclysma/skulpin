use std::mem;
use ash::vk;
use ash::prelude::VkResult;
use std::ffi::CString;
use std::mem::ManuallyDrop;

use ash::version::DeviceV1_0;

use super::VkDevice;
use super::VkSwapchain;
use crate::offset_of;
use super::SwapchainInfo;
use super::VkQueueFamilyIndices;
use crate::VkBuffer;
use crate::skia_support::{VkSkiaContext, VkSkiaSurface};

#[derive(Clone, Debug, Copy)]
struct Vertex {
    pos: [f32; 2],
    tex_coord: [f32; 2],
}

const VERTEX_LIST: [Vertex; 4] = [
    Vertex {
        pos: [-1.0, -1.0],
        tex_coord: [0.0, 0.0],
    },
    Vertex {
        pos: [1.0, -1.0],
        tex_coord: [1.0, 0.0],
    },
    Vertex {
        pos: [1.0, 1.0],
        tex_coord: [1.0, 1.0],
    },
    Vertex {
        pos: [-1.0, 1.0],
        tex_coord: [0.0, 1.0],
    },
];

const INDEX_LIST: [u16; 6] = [0, 1, 2, 2, 3, 0];

struct FixedFunctionState<'a> {
    vertex_input_assembly_state_info: vk::PipelineInputAssemblyStateCreateInfoBuilder<'a>,
    vertex_input_state_info: vk::PipelineVertexInputStateCreateInfoBuilder<'a>,
    viewport_state_info: vk::PipelineViewportStateCreateInfoBuilder<'a>,
    rasterization_info: vk::PipelineRasterizationStateCreateInfoBuilder<'a>,
    multisample_state_info: vk::PipelineMultisampleStateCreateInfoBuilder<'a>,
    color_blend_state_info: vk::PipelineColorBlendStateCreateInfoBuilder<'a>,
    dynamic_state_info: vk::PipelineDynamicStateCreateInfoBuilder<'a>,
}

struct PipelineResources {
    pipeline_layout: vk::PipelineLayout,
    renderpass: vk::RenderPass,
    pipeline: vk::Pipeline,
}

/// Handles the skia renderpass
pub struct VkSkiaRenderPass {
    pub device: ash::Device, // This struct is not responsible for releasing this
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub pipeline_layout: vk::PipelineLayout,
    pub renderpass: vk::RenderPass,
    pub pipeline: vk::Pipeline,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub vertex_buffer: ManuallyDrop<VkBuffer>,
    pub index_buffer: ManuallyDrop<VkBuffer>,
    pub skia_surfaces: Vec<VkSkiaSurface>,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    pub image_sampler: vk::Sampler,
}

impl VkSkiaRenderPass {
    pub fn new(
        device: &VkDevice,
        swapchain: &VkSwapchain,
        skia_context: &mut VkSkiaContext,
    ) -> VkResult<Self> {
        let descriptor_set_layout = Self::create_descriptor_set_layout(&device.logical_device)?;

        let pipeline_resources =
            Self::create_fixed_function_state(&swapchain.swapchain_info, |fixed_function_state| {
                Self::create_renderpass_create_info(
                    &swapchain.swapchain_info,
                    |renderpass_create_info| {
                        Self::create_pipeline(
                            &device.logical_device,
                            &swapchain.swapchain_info,
                            fixed_function_state,
                            renderpass_create_info,
                            descriptor_set_layout,
                        )
                    },
                )
            })?;

        let pipeline_layout = pipeline_resources.pipeline_layout;
        let renderpass = pipeline_resources.renderpass;
        let pipeline = pipeline_resources.pipeline;

        let framebuffers = Self::create_framebuffers(
            &device.logical_device,
            &swapchain.swapchain_image_views,
            &swapchain.swapchain_info,
            pipeline_resources.renderpass,
        )?;

        let command_pool =
            Self::create_command_pool(&device.logical_device, &device.queue_family_indices)?;

        let command_buffers = Self::create_command_buffers(
            &device.logical_device,
            &swapchain.swapchain_info,
            command_pool,
        )?;

        let vertex_buffer = Self::create_vertex_buffer(
            &device.logical_device,
            device.queues.graphics_queue,
            command_pool,
            &device.memory_properties,
        )?;

        let index_buffer = Self::create_index_buffer(
            &device.logical_device,
            device.queues.graphics_queue,
            command_pool,
            &device.memory_properties,
        )?;

        info!(
            "Create skia surfaces with extent: {:?}",
            swapchain.swapchain_info.extents
        );

        // Force the skia surface size to be >0, otherwise the surface will fail to be created.
        // I could fix this by not creating the canvas at all and having the upstream code
        // check if the canvas is valid, but this is simpler.
        let mut skia_surface_extents = swapchain.swapchain_info.extents;
        skia_surface_extents.width = skia_surface_extents.width.max(1);
        skia_surface_extents.height = skia_surface_extents.height.max(1);

        let mut skia_surfaces = Vec::with_capacity(swapchain.swapchain_info.image_count);
        for _ in 0..swapchain.swapchain_info.image_count {
            skia_surfaces.push(VkSkiaSurface::new(
                device,
                skia_context,
                skia_surface_extents,
            )?)
        }

        let image_sampler = VkSkiaSurface::create_sampler(&device.logical_device)?;

        let descriptor_pool = Self::create_descriptor_pool(
            &device.logical_device,
            swapchain.swapchain_info.image_count as u32,
        )?;

        let descriptor_sets = Self::create_descriptor_sets(
            &device.logical_device,
            descriptor_pool,
            descriptor_set_layout,
            swapchain.swapchain_info.image_count,
            image_sampler,
            &skia_surfaces,
        )?;

        for i in 0..swapchain.swapchain_info.image_count {
            Self::record_command_buffer(
                &device.logical_device,
                &swapchain.swapchain_info,
                renderpass,
                framebuffers[i],
                pipeline,
                pipeline_layout,
                command_buffers[i],
                vertex_buffer.buffer,
                index_buffer.buffer,
                descriptor_sets[i],
                &skia_surfaces[i],
            )?;
        }

        Ok(VkSkiaRenderPass {
            device: device.logical_device.clone(),
            descriptor_set_layout,
            pipeline_layout,
            renderpass,
            pipeline,
            framebuffers,
            command_pool,
            command_buffers,
            vertex_buffer,
            index_buffer,
            skia_surfaces,
            descriptor_pool,
            descriptor_sets,
            image_sampler,
        })
    }

    fn create_descriptor_set_layout(
        logical_device: &ash::Device
    ) -> VkResult<vk::DescriptorSetLayout> {
        let descriptor_set_layout_bindings = [vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build()];

        let descriptor_set_layout_create_info =
            vk::DescriptorSetLayoutCreateInfo::builder().bindings(&descriptor_set_layout_bindings);

        unsafe {
            logical_device.create_descriptor_set_layout(&descriptor_set_layout_create_info, None)
        }
    }

    fn create_fixed_function_state<F: FnMut(&FixedFunctionState) -> VkResult<PipelineResources>>(
        swapchain_info: &SwapchainInfo,
        mut f: F,
    ) -> VkResult<PipelineResources> {
        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let vertex_input_binding_descriptions = [vk::VertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }];
        let vertex_input_attribute_descriptions = [
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Vertex, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Vertex, tex_coord) as u32,
            },
        ];

        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&vertex_input_attribute_descriptions)
            .vertex_binding_descriptions(&vertex_input_binding_descriptions);

        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: swapchain_info.extents.width as f32,
            height: swapchain_info.extents.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain_info.extents,
        }];

        let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
            .scissors(&scissors)
            .viewports(&viewports);

        let rasterization_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .line_width(1.0)
            .polygon_mode(vk::PolygonMode::FILL);

        // Skip depth/stencil testing

        let multisample_state_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        // Applies to the current framebuffer
        let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::all())
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .build()];

        // Applies globally
        let color_blend_state_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .attachments(&color_blend_attachment_states);

        let dynamic_state = vec![/*vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR, vk::DynamicState::LINE_WIDTH*/];
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state);

        let fixed_function_state = FixedFunctionState {
            vertex_input_assembly_state_info,
            vertex_input_state_info,
            viewport_state_info,
            rasterization_info,
            multisample_state_info,
            color_blend_state_info,
            dynamic_state_info,
        };

        f(&fixed_function_state)
    }

    fn create_renderpass_create_info<
        F: FnMut(&vk::RenderPassCreateInfo) -> VkResult<PipelineResources>,
    >(
        swapchain_info: &SwapchainInfo,
        mut f: F,
    ) -> VkResult<PipelineResources> {
        let renderpass_attachments = [vk::AttachmentDescription::builder()
            .format(swapchain_info.surface_format.format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build()];

        let color_attachment_refs = [vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];

        let subpasses = [vk::SubpassDescription::builder()
            .color_attachments(&color_attachment_refs)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .build()];

        let dependencies = [vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::default())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            )
            .build()];

        let renderpass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&renderpass_attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        f(&renderpass_create_info)
    }

    fn create_pipeline(
        logical_device: &ash::Device,
        _swapchain_info: &SwapchainInfo,
        fixed_function_state: &FixedFunctionState,
        renderpass_create_info: &vk::RenderPassCreateInfo,
        descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> VkResult<PipelineResources> {
        //
        // Load Shaders
        //
        let vertex_shader_module = Self::load_shader_module(
            logical_device,
            &include_bytes!("../shaders/skia.vert.spv")[..],
        )?;

        let fragment_shader_module = Self::load_shader_module(
            logical_device,
            &include_bytes!("../shaders/skia.frag.spv")[..],
        )?;

        let shader_entry_name = CString::new("main").unwrap();
        let shader_stage_create_infos = [
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vertex_shader_module)
                .name(&shader_entry_name)
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragment_shader_module)
                .name(&shader_entry_name)
                .build(),
        ];

        let descriptor_set_layouts = [descriptor_set_layout];

        let layout_create_info =
            vk::PipelineLayoutCreateInfo::builder().set_layouts(&descriptor_set_layouts);

        let pipeline_layout: vk::PipelineLayout =
            unsafe { logical_device.create_pipeline_layout(&layout_create_info, None)? };

        let renderpass: vk::RenderPass =
            unsafe { logical_device.create_render_pass(renderpass_create_info, None)? };

        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stage_create_infos)
            .vertex_input_state(&fixed_function_state.vertex_input_state_info)
            .input_assembly_state(&fixed_function_state.vertex_input_assembly_state_info)
            .viewport_state(&fixed_function_state.viewport_state_info)
            .rasterization_state(&fixed_function_state.rasterization_info)
            .multisample_state(&fixed_function_state.multisample_state_info)
            .color_blend_state(&fixed_function_state.color_blend_state_info)
            .dynamic_state(&fixed_function_state.dynamic_state_info)
            .layout(pipeline_layout)
            .render_pass(renderpass);

        let pipeline = unsafe {
            match logical_device.create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[pipeline_info.build()],
                None,
            ) {
                Ok(result) => Ok(result[0]),
                Err(e) => Err(e.1),
            }?
        };

        //
        // Destroy shader modules. They don't need to be kept around once the pipeline is built
        //
        unsafe {
            logical_device.destroy_shader_module(vertex_shader_module, None);
            logical_device.destroy_shader_module(fragment_shader_module, None);
        }

        Ok(PipelineResources {
            pipeline_layout,
            renderpass,
            pipeline,
        })
    }

    fn load_shader_module(
        logical_device: &ash::Device,
        data: &[u8],
    ) -> VkResult<vk::ShaderModule> {
        let mut spv_file = std::io::Cursor::new(data);
        //TODO: Pass this error up
        let code =
            super::util::read_spv(&mut spv_file).expect("Failed to read vertex shader spv file");
        let shader_info = vk::ShaderModuleCreateInfo::builder().code(&code);

        unsafe { logical_device.create_shader_module(&shader_info, None) }
    }

    fn create_framebuffers(
        logical_device: &ash::Device,
        swapchain_image_views: &[vk::ImageView],
        swapchain_info: &SwapchainInfo,
        renderpass: vk::RenderPass,
    ) -> VkResult<Vec<vk::Framebuffer>> {
        let mut framebuffers = Vec::with_capacity(swapchain_image_views.len());

        for swapchain_image_view in swapchain_image_views {
            let framebuffer_attachments = [*swapchain_image_view];
            let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(renderpass)
                .attachments(&framebuffer_attachments)
                .width(swapchain_info.extents.width)
                .height(swapchain_info.extents.height)
                .layers(1);

            let framebuffer =
                unsafe { logical_device.create_framebuffer(&framebuffer_create_info, None)? };

            framebuffers.push(framebuffer);
        }

        Ok(framebuffers)
    }

    fn create_command_pool(
        logical_device: &ash::Device,
        queue_family_indices: &VkQueueFamilyIndices,
    ) -> VkResult<vk::CommandPool> {
        info!(
            "Creating command pool with queue family index {}",
            queue_family_indices.graphics_queue_family_index
        );
        let pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family_indices.graphics_queue_family_index);

        unsafe { logical_device.create_command_pool(&pool_create_info, None) }
    }

    fn create_command_buffers(
        logical_device: &ash::Device,
        swapchain_info: &SwapchainInfo,
        command_pool: vk::CommandPool,
    ) -> VkResult<Vec<vk::CommandBuffer>> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(swapchain_info.image_count as u32)
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        unsafe { logical_device.allocate_command_buffers(&command_buffer_allocate_info) }
    }

    fn create_vertex_buffer(
        logical_device: &ash::Device,
        queue: vk::Queue,
        command_pool: vk::CommandPool,
        device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
    ) -> VkResult<ManuallyDrop<VkBuffer>> {
        VkBuffer::new_from_slice_device_local(
            logical_device,
            device_memory_properties,
            queue,
            command_pool,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            &VERTEX_LIST,
        )
    }

    fn create_index_buffer(
        logical_device: &ash::Device,
        queue: vk::Queue,
        command_pool: vk::CommandPool,
        device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
    ) -> VkResult<ManuallyDrop<VkBuffer>> {
        VkBuffer::new_from_slice_device_local(
            logical_device,
            device_memory_properties,
            queue,
            command_pool,
            vk::BufferUsageFlags::INDEX_BUFFER,
            &INDEX_LIST,
        )
    }

    fn create_descriptor_pool(
        logical_device: &ash::Device,
        swapchain_image_count: u32,
    ) -> VkResult<vk::DescriptorPool> {
        let pool_sizes = [vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(swapchain_image_count)
            .build()];

        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_sizes)
            .max_sets(swapchain_image_count);

        unsafe { logical_device.create_descriptor_pool(&descriptor_pool_info, None) }
    }

    fn create_descriptor_sets(
        logical_device: &ash::Device,
        descriptor_pool: vk::DescriptorPool,
        descriptor_set_layout: vk::DescriptorSetLayout,
        swapchain_image_count: usize,
        image_sampler: vk::Sampler,
        skia_surface: &[VkSkiaSurface],
    ) -> VkResult<Vec<vk::DescriptorSet>> {
        // DescriptorSetAllocateInfo expects an array with an element per set
        let descriptor_set_layouts = vec![descriptor_set_layout; swapchain_image_count];

        let alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(descriptor_set_layouts.as_slice());

        let descriptor_sets = unsafe { logical_device.allocate_descriptor_sets(&alloc_info) }?;

        for i in 0..swapchain_image_count {
            let descriptor_image_infos = [vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(skia_surface[i].image_view)
                .sampler(image_sampler)
                .build()];

            let descriptor_writes = [vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_sets[i])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&descriptor_image_infos)
                .build()];

            unsafe {
                logical_device.update_descriptor_sets(&descriptor_writes, &[]);
            }
        }

        Ok(descriptor_sets)
    }

    #[allow(clippy::too_many_arguments)]
    fn record_command_buffer(
        logical_device: &ash::Device,
        swapchain_info: &SwapchainInfo,
        renderpass: vk::RenderPass,
        framebuffer: vk::Framebuffer,
        pipeline: vk::Pipeline,
        pipeline_layout: vk::PipelineLayout,
        command_buffer: vk::CommandBuffer,
        vertex_buffer: vk::Buffer,
        index_buffer: vk::Buffer,
        descriptor_set: vk::DescriptorSet,
        skia_surface: &VkSkiaSurface,
    ) -> VkResult<()> {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder();

        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        }];

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(renderpass)
            .framebuffer(framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: swapchain_info.extents,
            })
            .clear_values(&clear_values);

        // Implicitly resets the command buffer
        unsafe {
            logical_device.begin_command_buffer(command_buffer, &command_buffer_begin_info)?;

            let image = VkSkiaSurface::get_image_from_skia_texture(&skia_surface.texture);

            Self::add_image_barrier(
                logical_device,
                command_buffer,
                image,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            );

            logical_device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            logical_device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline,
            );

            logical_device.cmd_bind_vertex_buffers(
                command_buffer,
                0, // first binding
                &[vertex_buffer],
                &[0], // offsets
            );

            logical_device.cmd_bind_index_buffer(
                command_buffer,
                index_buffer,
                0, // offset
                vk::IndexType::UINT16,
            );

            logical_device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline_layout,
                0,
                &[descriptor_set],
                &[],
            );

            logical_device.cmd_draw_indexed(command_buffer, INDEX_LIST.len() as u32, 1, 0, 0, 0);
            logical_device.cmd_end_render_pass(command_buffer);

            Self::add_image_barrier(
                logical_device,
                command_buffer,
                image,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            );

            logical_device.end_command_buffer(command_buffer)
        }
    }

    fn add_image_barrier(
        logical_device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        image: vk::Image,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) {
        let image_memory_barrier = ash::vk::ImageMemoryBarrier::builder()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(
                ash::vk::ImageSubresourceRange::builder()
                    .aspect_mask(ash::vk::ImageAspectFlags::COLOR)
                    .level_count(1 /*VK_REMAINING_MIP_LEVELS*/)
                    .layer_count(1 /*VK_REMAINING_ARRAY_LAYERS*/)
                    .build(),
            )
            .build();

        unsafe {
            logical_device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::BY_REGION,
                &[],                     //memory_barriers,
                &[],                     //buffer_memory_barriers,
                &[image_memory_barrier], //image_memory_barriers
            );
        }
    }

    pub fn skia_surface(
        &mut self,
        index: usize,
    ) -> &mut VkSkiaSurface {
        &mut self.skia_surfaces[index]
    }
}

impl Drop for VkSkiaRenderPass {
    fn drop(&mut self) {
        debug!("destroying VkSkiaRenderPass");

        unsafe {
            self.device.destroy_sampler(self.image_sampler, None);

            ManuallyDrop::drop(&mut self.vertex_buffer);
            ManuallyDrop::drop(&mut self.index_buffer);

            self.device.destroy_command_pool(self.command_pool, None);

            for framebuffer in &self.framebuffers {
                self.device.destroy_framebuffer(*framebuffer, None);
            }

            self.device.destroy_pipeline(self.pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.renderpass, None);

            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            self.device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
        }

        debug!("destroyed VkSkiaRenderPass");
    }
}
