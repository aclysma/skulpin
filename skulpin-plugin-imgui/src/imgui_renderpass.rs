use skulpin_renderer::ash;

use std::mem;
use ash::vk;
use ash::prelude::VkResult;
use std::ffi::CString;
use std::mem::ManuallyDrop;

use ash::version::DeviceV1_0;

use skulpin_renderer::VkDevice;
use skulpin_renderer::VkSwapchain;
use skulpin_renderer::offset_of;
use skulpin_renderer::SwapchainInfo;
use skulpin_renderer::VkQueueFamilyIndices;
use skulpin_renderer::VkBuffer;
use skulpin_renderer::util;

use super::VkImage;

#[derive(Clone, Debug, Copy)]
struct UniformBufferObject {
    mvp: [[f32; 4]; 4],
}

#[derive(Clone, Debug, Copy)]
struct Vertex {
    pos: [f32; 2],
    tex_coord: [f32; 2],
    color: [u8; 4],
}

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

pub struct VkImGuiRenderPassFontAtlas {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl VkImGuiRenderPassFontAtlas {
    pub fn new(texture: &imgui::FontAtlasTexture) -> Self {
        VkImGuiRenderPassFontAtlas {
            width: texture.width,
            height: texture.height,
            data: texture.data.to_vec(),
        }
    }
}

pub struct VkImGuiRenderPass {
    pub device: ash::Device, // This struct is not responsible for releasing this
    pub swapchain_info: SwapchainInfo,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub pipeline_layout: vk::PipelineLayout,
    pub renderpass: vk::RenderPass,
    pub pipeline: vk::Pipeline,
    pub frame_buffers: Vec<vk::Framebuffer>,
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub vertex_buffers: Vec<Vec<ManuallyDrop<VkBuffer>>>,
    pub index_buffers: Vec<Vec<ManuallyDrop<VkBuffer>>>,
    pub staging_vertex_buffers: Vec<Vec<ManuallyDrop<VkBuffer>>>,
    pub staging_index_buffers: Vec<Vec<ManuallyDrop<VkBuffer>>>,
    pub uniform_buffers: Vec<ManuallyDrop<VkBuffer>>,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    pub image: ManuallyDrop<VkImage>,
    pub image_view: vk::ImageView,
    pub image_sampler: vk::Sampler,
}

impl VkImGuiRenderPass {
    pub fn new(
        device: &VkDevice,
        swapchain: &VkSwapchain,
        font_atlas: &VkImGuiRenderPassFontAtlas,
    ) -> VkResult<Self> {
        let mut pipeline_resources = None;

        let descriptor_set_layout = Self::create_descriptor_set_layout(&device.logical_device)?;

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
                        |resources| {
                            pipeline_resources = Some(resources);
                        },
                    )
                },
            )
        })?;

        //TODO: Return error if not set
        let pipeline_resources = pipeline_resources.unwrap();
        let pipeline_layout = pipeline_resources.pipeline_layout;
        let renderpass = pipeline_resources.renderpass;
        let pipeline = pipeline_resources.pipeline;

        let frame_buffers = Self::create_framebuffers(
            &device.logical_device,
            &swapchain.swapchain_image_views,
            &swapchain.swapchain_info,
            pipeline_resources.renderpass,
        );

        let command_pool =
            Self::create_command_pool(&device.logical_device, &device.queue_family_indices)?;

        let command_buffers = Self::create_command_buffers(
            &device.logical_device,
            &swapchain.swapchain_info,
            command_pool,
        )?;

        let mut vertex_buffers = Vec::with_capacity(swapchain.swapchain_info.image_count);
        let mut index_buffers = Vec::with_capacity(swapchain.swapchain_info.image_count);
        let mut staging_vertex_buffers = Vec::with_capacity(swapchain.swapchain_info.image_count);
        let mut staging_index_buffers = Vec::with_capacity(swapchain.swapchain_info.image_count);
        for _ in 0..swapchain.swapchain_info.image_count {
            vertex_buffers.push(vec![]);
            index_buffers.push(vec![]);
            staging_vertex_buffers.push(vec![]);
            staging_index_buffers.push(vec![]);
        }

        let mut uniform_buffers = Vec::with_capacity(swapchain.swapchain_info.image_count);
        for _ in 0..swapchain.swapchain_info.image_count {
            uniform_buffers.push(Self::create_uniform_buffer(
                &device.logical_device,
                &device.memory_properties,
            )?)
        }

        let image = Self::load_image(
            &device.logical_device,
            device.queues.graphics_queue,
            command_pool,
            &device.memory_properties,
            font_atlas,
        )?;

        let image_view = Self::create_texture_image_view(&device.logical_device, image.image);

        let image_sampler = Self::create_texture_image_sampler(&device.logical_device);

        let descriptor_pool = Self::create_descriptor_pool(
            &device.logical_device,
            swapchain.swapchain_info.image_count as u32,
        )?;

        let descriptor_sets = Self::create_descriptor_sets(
            &device.logical_device,
            descriptor_pool,
            descriptor_set_layout,
            swapchain.swapchain_info.image_count,
            &uniform_buffers,
            image_view,
            image_sampler,
        )?;

        for i in 0..swapchain.swapchain_info.image_count {
            Self::record_command_buffer(
                None,
                &device.memory_properties,
                &device.logical_device,
                &swapchain.swapchain_info,
                renderpass,
                frame_buffers[i],
                pipeline,
                pipeline_layout,
                command_buffers[i],
                &mut vertex_buffers[i],
                &mut index_buffers[i],
                &mut staging_vertex_buffers[i],
                &mut staging_index_buffers[i],
                descriptor_sets[i],
            )?;
        }

        Ok(VkImGuiRenderPass {
            device: device.logical_device.clone(),
            swapchain_info: swapchain.swapchain_info.clone(),
            descriptor_set_layout,
            pipeline_layout,
            renderpass,
            pipeline,
            frame_buffers,
            command_pool,
            command_buffers,
            vertex_buffers,
            index_buffers,
            staging_vertex_buffers,
            staging_index_buffers,
            uniform_buffers,
            descriptor_pool,
            descriptor_sets,
            image,
            image_view,
            image_sampler,
        })
    }

    fn create_descriptor_set_layout(
        logical_device: &ash::Device,
    ) -> VkResult<vk::DescriptorSetLayout> {
        let descriptor_set_layout_bindings = [
            vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .build(),
            vk::DescriptorSetLayoutBinding::builder()
                .binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build(),
        ];

        let descriptor_set_layout_create_info =
            vk::DescriptorSetLayoutCreateInfo::builder().bindings(&descriptor_set_layout_bindings);

        unsafe {
            logical_device.create_descriptor_set_layout(&descriptor_set_layout_create_info, None)
        }
    }

    fn create_fixed_function_state<F: FnMut(&FixedFunctionState) -> VkResult<()>>(
        swapchain_info: &SwapchainInfo,
        mut f: F,
    ) -> VkResult<()> {
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
                offset: skulpin_renderer::offset_of!(Vertex, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Vertex, tex_coord) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 2,
                format: vk::Format::R8G8B8A8_UNORM,
                offset: offset_of!(Vertex, color) as u32,
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

        let dynamic_state = vec![vk::DynamicState::SCISSOR];
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

    fn create_renderpass_create_info<F: FnMut(&vk::RenderPassCreateInfo) -> VkResult<()>>(
        swapchain_info: &SwapchainInfo,
        mut f: F,
    ) -> VkResult<()> {
        let renderpass_attachments = [vk::AttachmentDescription::builder()
            .format(swapchain_info.surface_format.format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::LOAD)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::PRESENT_SRC_KHR)
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

    fn create_pipeline<F: FnMut(PipelineResources)>(
        logical_device: &ash::Device,
        _swapchain_info: &SwapchainInfo,
        fixed_function_state: &FixedFunctionState,
        renderpass_create_info: &vk::RenderPassCreateInfo,
        descriptor_set_layout: vk::DescriptorSetLayout,
        mut f: F,
    ) -> VkResult<()> {
        //
        // Load Shaders
        //
        let vertex_shader_module = Self::load_shader_module(
            logical_device,
            &include_bytes!("../shaders/imgui.vert.spv")[..],
        )?;

        let fragment_shader_module = Self::load_shader_module(
            logical_device,
            &include_bytes!("../shaders/imgui.frag.spv")[..],
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

        let resources = PipelineResources {
            pipeline_layout,
            renderpass,
            pipeline,
        };

        f(resources);
        Ok(())
    }

    fn load_shader_module(logical_device: &ash::Device, data: &[u8]) -> VkResult<vk::ShaderModule> {
        let mut spv_file = std::io::Cursor::new(data);
        //TODO: Pass this error up
        let code = skulpin_renderer::util::read_spv(&mut spv_file)
            .expect("Failed to read vertex shader spv file");
        let shader_info = vk::ShaderModuleCreateInfo::builder().code(&code);

        unsafe { logical_device.create_shader_module(&shader_info, None) }
    }

    fn create_framebuffers(
        logical_device: &ash::Device,
        swapchain_image_views: &[vk::ImageView],
        swapchain_info: &SwapchainInfo,
        renderpass: vk::RenderPass,
    ) -> Vec<vk::Framebuffer> {
        swapchain_image_views
            .iter()
            .map(|&swapchain_image_view| {
                let framebuffer_attachments = [swapchain_image_view];
                let frame_buffer_create_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(renderpass)
                    .attachments(&framebuffer_attachments)
                    .width(swapchain_info.extents.width)
                    .height(swapchain_info.extents.height)
                    .layers(1);

                unsafe {
                    //TODO: Pass this error up
                    logical_device
                        .create_framebuffer(&frame_buffer_create_info, None)
                        .unwrap()
                }
            })
            .collect()
    }

    fn create_command_pool(
        logical_device: &ash::Device,
        queue_family_indices: &VkQueueFamilyIndices,
    ) -> VkResult<vk::CommandPool> {
        log::info!(
            "Creating command pool with queue family index {}",
            queue_family_indices.graphics_queue_family_index
        );
        let pool_create_info = vk::CommandPoolCreateInfo::builder()
            .flags(
                vk::CommandPoolCreateFlags::TRANSIENT
                    | vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            )
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

    fn create_uniform_buffer(
        logical_device: &ash::Device,
        device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
    ) -> VkResult<ManuallyDrop<VkBuffer>> {
        let buffer = VkBuffer::new(
            logical_device,
            device_memory_properties,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            mem::size_of::<UniformBufferObject>() as u64,
        );

        Ok(ManuallyDrop::new(buffer?))
    }

    pub fn load_image(
        logical_device: &ash::Device,
        queue: vk::Queue,
        command_pool: vk::CommandPool,
        device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
        font_atlas: &VkImGuiRenderPassFontAtlas,
    ) -> VkResult<ManuallyDrop<VkImage>> {
        let extent = vk::Extent3D {
            width: font_atlas.width,
            height: font_atlas.height,
            depth: 1,
        };

        log::info!("Using imgui font atlas of size {:?}", extent);

        let mut staging_buffer = VkBuffer::new(
            logical_device,
            device_memory_properties,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            font_atlas.data.len() as u64,
        )?;

        staging_buffer.write_to_host_visible_buffer(&font_atlas.data)?;

        let image = VkImage::new(
            logical_device,
            device_memory_properties,
            extent,
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        transition_image_layout(
            logical_device,
            queue,
            command_pool,
            image.image,
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        )?;

        copy_buffer_to_image(
            logical_device,
            queue,
            command_pool,
            staging_buffer.buffer,
            image.image,
            &image.extent,
        )?;

        transition_image_layout(
            logical_device,
            queue,
            command_pool,
            image.image,
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        )?;

        Ok(ManuallyDrop::new(image))
    }

    pub fn create_texture_image_view(
        logical_device: &ash::Device,
        image: vk::Image,
    ) -> vk::ImageView {
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);

        let image_view_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_UNORM)
            .subresource_range(*subresource_range);

        unsafe {
            logical_device
                .create_image_view(&image_view_info, None)
                .unwrap()
        }
    }

    pub fn create_texture_image_sampler(logical_device: &ash::Device) -> vk::Sampler {
        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(false)
            .max_anisotropy(1.0)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .mip_lod_bias(0.0)
            .min_lod(0.0)
            .max_lod(0.0);

        unsafe { logical_device.create_sampler(&sampler_info, None).unwrap() }
    }

    fn create_descriptor_pool(
        logical_device: &ash::Device,
        swapchain_image_count: u32,
    ) -> VkResult<vk::DescriptorPool> {
        let pool_sizes = [
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(swapchain_image_count)
                .build(),
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(swapchain_image_count)
                .build(),
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::SAMPLED_IMAGE)
                .descriptor_count(swapchain_image_count)
                .build(),
        ];

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
        uniform_buffers: &[ManuallyDrop<VkBuffer>],
        image_view: vk::ImageView,
        image_sampler: vk::Sampler,
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
                .image_view(image_view)
                .sampler(image_sampler)
                .build()];

            let descriptor_buffer_infos = [vk::DescriptorBufferInfo::builder()
                .buffer(uniform_buffers[i as usize].buffer)
                .offset(0)
                .range(mem::size_of::<UniformBufferObject>() as u64)
                .build()];

            let descriptor_writes = [
                vk::WriteDescriptorSet::builder()
                    .dst_set(descriptor_sets[i as usize])
                    .dst_binding(0)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(&descriptor_buffer_infos)
                    .build(),
                vk::WriteDescriptorSet::builder()
                    .dst_set(descriptor_sets[i])
                    .dst_binding(1)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&descriptor_image_infos)
                    .build(),
            ];

            unsafe {
                logical_device.update_descriptor_sets(&descriptor_writes, &[]);
            }
        }

        Ok(descriptor_sets)
    }

    #[allow(clippy::too_many_arguments)]
    fn record_command_buffer(
        imgui_draw_data: Option<&imgui::DrawData>,
        device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
        logical_device: &ash::Device,
        swapchain_info: &SwapchainInfo,
        renderpass: vk::RenderPass,
        framebuffer: vk::Framebuffer,
        pipeline: vk::Pipeline,
        pipeline_layout: vk::PipelineLayout,
        command_buffer: vk::CommandBuffer,
        vertex_buffers: &mut Vec<ManuallyDrop<VkBuffer>>,
        index_buffers: &mut Vec<ManuallyDrop<VkBuffer>>,
        staging_vertex_buffers: &mut Vec<ManuallyDrop<VkBuffer>>,
        staging_index_buffers: &mut Vec<ManuallyDrop<VkBuffer>>,
        descriptor_set: vk::DescriptorSet,
    ) -> VkResult<()> {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder();

        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        }];

        fn drop_old_buffers(buffers: &mut Vec<ManuallyDrop<VkBuffer>>) {
            for b in buffers.iter_mut() {
                unsafe {
                    ManuallyDrop::drop(b);
                }
            }

            buffers.clear();
        }

        drop_old_buffers(vertex_buffers);
        drop_old_buffers(index_buffers);
        drop_old_buffers(staging_vertex_buffers);
        drop_old_buffers(staging_index_buffers);

        let mut draw_list_count = 0;
        if let Some(draw_data) = imgui_draw_data {
            for draw_list in draw_data.draw_lists() {
                let (vertex_buffer, staging_vertex_buffer) = {
                    let vertex_buffer_size = draw_list.vtx_buffer().len() as u64
                        * std::mem::size_of::<imgui::DrawVert>() as u64;
                    let mut staging_vertex_buffer = VkBuffer::new(
                        logical_device,
                        &device_memory_properties,
                        vk::BufferUsageFlags::TRANSFER_SRC,
                        vk::MemoryPropertyFlags::HOST_VISIBLE
                            | vk::MemoryPropertyFlags::HOST_COHERENT,
                        vertex_buffer_size,
                    )?;

                    staging_vertex_buffer.write_to_host_visible_buffer(draw_list.vtx_buffer())?;

                    let vertex_buffer = VkBuffer::new(
                        &logical_device,
                        &device_memory_properties,
                        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
                        vk::MemoryPropertyFlags::DEVICE_LOCAL,
                        vertex_buffer_size,
                    )?;

                    (vertex_buffer, staging_vertex_buffer)
                };

                //TODO: Duplicated code here
                let (index_buffer, staging_index_buffer) = {
                    let index_buffer_size = draw_list.idx_buffer().len() as u64
                        * std::mem::size_of::<imgui::DrawIdx>() as u64;
                    let mut staging_index_buffer = VkBuffer::new(
                        logical_device,
                        &device_memory_properties,
                        vk::BufferUsageFlags::TRANSFER_SRC,
                        vk::MemoryPropertyFlags::HOST_VISIBLE
                            | vk::MemoryPropertyFlags::HOST_COHERENT,
                        index_buffer_size,
                    )?;

                    staging_index_buffer.write_to_host_visible_buffer(draw_list.idx_buffer())?;

                    let index_buffer = VkBuffer::new(
                        &logical_device,
                        &device_memory_properties,
                        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
                        vk::MemoryPropertyFlags::DEVICE_LOCAL,
                        index_buffer_size,
                    )?;

                    (index_buffer, staging_index_buffer)
                };

                vertex_buffers.push(ManuallyDrop::new(vertex_buffer));
                staging_vertex_buffers.push(ManuallyDrop::new(staging_vertex_buffer));
                index_buffers.push(ManuallyDrop::new(index_buffer));
                staging_index_buffers.push(ManuallyDrop::new(staging_index_buffer));
                draw_list_count += 1;
            }
        }

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

            for i in 0..draw_list_count {
                {
                    let buffer_copy_info = [vk::BufferCopy::builder()
                        .size(staging_vertex_buffers[i].size)
                        .build()];

                    logical_device.cmd_copy_buffer(
                        command_buffer,
                        staging_vertex_buffers[i].buffer,
                        vertex_buffers[i].buffer,
                        &buffer_copy_info,
                    );
                }

                //TODO: Duplicated code here
                {
                    let buffer_copy_info = [vk::BufferCopy::builder()
                        .size(staging_index_buffers[i].size)
                        .build()];

                    logical_device.cmd_copy_buffer(
                        command_buffer,
                        staging_index_buffers[i].buffer,
                        index_buffers[i].buffer,
                        &buffer_copy_info,
                    );
                }
            }

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

            logical_device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline_layout,
                0,
                &[descriptor_set],
                &[],
            );

            let mut draw_list_index = 0;
            if let Some(draw_data) = imgui_draw_data {
                for draw_list in draw_data.draw_lists() {
                    logical_device.cmd_bind_vertex_buffers(
                        command_buffer,
                        0, // first binding
                        &[vertex_buffers[draw_list_index].buffer],
                        &[0], // offsets
                    );

                    logical_device.cmd_bind_index_buffer(
                        command_buffer,
                        index_buffers[draw_list_index].buffer,
                        0, // offset
                        vk::IndexType::UINT16,
                    );

                    let mut element_begin_index: u32 = 0;
                    for cmd in draw_list.commands() {
                        match cmd {
                            imgui::DrawCmd::Elements {
                                count,
                                cmd_params:
                                    imgui::DrawCmdParams {
                                        clip_rect,
                                        //texture_id,
                                        ..
                                    },
                            } => {
                                let element_end_index = element_begin_index + count as u32;

                                let scissors = vk::Rect2D {
                                    offset: vk::Offset2D {
                                        x: ((clip_rect[0] - draw_data.display_pos[0])
                                            * draw_data.framebuffer_scale[0])
                                            as i32,
                                        y: ((clip_rect[1] - draw_data.display_pos[1])
                                            * draw_data.framebuffer_scale[1])
                                            as i32,
                                    },
                                    extent: vk::Extent2D {
                                        width: ((clip_rect[2]
                                            - clip_rect[0]
                                            - draw_data.display_pos[0])
                                            * draw_data.framebuffer_scale[0])
                                            as u32,
                                        height: ((clip_rect[3]
                                            - clip_rect[1]
                                            - draw_data.display_pos[1])
                                            * draw_data.framebuffer_scale[1])
                                            as u32,
                                    },
                                };

                                logical_device.cmd_set_scissor(command_buffer, 0, &[scissors]);

                                logical_device.cmd_draw_indexed(
                                    command_buffer,
                                    element_end_index - element_begin_index,
                                    1,
                                    element_begin_index,
                                    0,
                                    0,
                                );

                                element_begin_index = element_end_index;
                            }
                            _ => panic!("unexpected draw command"),
                        }
                    }

                    draw_list_index += 1;
                }
            }

            logical_device.cmd_end_render_pass(command_buffer);

            logical_device.end_command_buffer(command_buffer)
        }
    }

    // This is almost copy-pasted from glam. I wanted to avoid pulling in the entire library for a
    // single function
    pub fn orthographic_rh_gl(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> [[f32; 4]; 4] {
        let a = 2.0 / (right - left);
        let b = 2.0 / (top - bottom);
        let c = -2.0 / (far - near);
        let tx = -(right + left) / (right - left);
        let ty = -(top + bottom) / (top - bottom);
        let tz = -(far + near) / (far - near);

        [
            [a, 0.0, 0.0, 0.0],
            [0.0, b, 0.0, 0.0],
            [0.0, 0.0, c, 0.0],
            [tx, ty, tz, 1.0],
        ]
    }

    //TODO: This is doing a GPU idle wait, would be better to integrate it into the command
    // buffer
    pub fn update_uniform_buffer(
        &mut self,
        swapchain_image_index: usize,
        extents: vk::Extent2D,
        hidpi_factor: f64,
    ) -> VkResult<()> {
        let proj = Self::orthographic_rh_gl(
            0.0,
            (extents.width as f64 / hidpi_factor) as f32,
            0.0,
            (extents.height as f64 / hidpi_factor) as f32,
            -100.0,
            100.0,
        );

        let ubo = UniformBufferObject { mvp: proj };

        self.uniform_buffers[swapchain_image_index].write_to_host_visible_buffer(&[ubo])
    }

    pub fn update(
        &mut self,
        device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
        imgui_draw_data: Option<&imgui::DrawData>,
        present_index: usize,
        hidpi_factor: f64,
    ) -> VkResult<()> {
        //TODO: Integrate this into the command buffer we create below
        self.update_uniform_buffer(present_index, self.swapchain_info.extents, hidpi_factor)?;

        Self::record_command_buffer(
            imgui_draw_data,
            device_memory_properties,
            &self.device,
            &self.swapchain_info,
            self.renderpass,
            self.frame_buffers[present_index],
            self.pipeline,
            self.pipeline_layout,
            self.command_buffers[present_index],
            &mut self.vertex_buffers[present_index],
            &mut self.index_buffers[present_index],
            &mut self.staging_vertex_buffers[present_index],
            &mut self.staging_index_buffers[present_index],
            self.descriptor_sets[present_index],
        )
    }
}

impl Drop for VkImGuiRenderPass {
    fn drop(&mut self) {
        log::debug!("destroying VkImGuiRenderPass");

        fn drop_all_buffer_lists(buffer_list: &mut Vec<Vec<ManuallyDrop<VkBuffer>>>) {
            for buffers in buffer_list {
                for mut b in &mut *buffers {
                    unsafe {
                        ManuallyDrop::drop(&mut b);
                    }
                }
            }
        }

        unsafe {
            self.device.destroy_sampler(self.image_sampler, None);
            self.device.destroy_image_view(self.image_view, None);
            ManuallyDrop::drop(&mut self.image);

            for uniform_buffer in &mut self.uniform_buffers {
                ManuallyDrop::drop(uniform_buffer);
            }

            drop_all_buffer_lists(&mut self.vertex_buffers);
            drop_all_buffer_lists(&mut self.index_buffers);
            drop_all_buffer_lists(&mut self.staging_vertex_buffers);
            drop_all_buffer_lists(&mut self.staging_index_buffers);

            self.device.destroy_command_pool(self.command_pool, None);

            for frame_buffer in &self.frame_buffers {
                self.device.destroy_framebuffer(*frame_buffer, None);
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

        log::debug!("destroyed VkImGuiRenderPass");
    }
}

pub fn transition_image_layout(
    logical_device: &ash::Device,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
    image: vk::Image,
    _format: vk::Format,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
) -> VkResult<()> {
    util::submit_single_use_command_buffer(logical_device, queue, command_pool, |command_buffer| {
        struct SyncInfo {
            src_access_mask: vk::AccessFlags,
            dst_access_mask: vk::AccessFlags,
            src_stage: vk::PipelineStageFlags,
            dst_stage: vk::PipelineStageFlags,
        }

        let sync_info = match (old_layout, new_layout) {
            (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => SyncInfo {
                src_access_mask: vk::AccessFlags::empty(),
                dst_access_mask: vk::AccessFlags::TRANSFER_WRITE,
                src_stage: vk::PipelineStageFlags::TOP_OF_PIPE,
                dst_stage: vk::PipelineStageFlags::TRANSFER,
            },
            (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => {
                SyncInfo {
                    src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
                    dst_access_mask: vk::AccessFlags::SHADER_READ,
                    src_stage: vk::PipelineStageFlags::TRANSFER,
                    dst_stage: vk::PipelineStageFlags::FRAGMENT_SHADER,
                }
            }
            _ => {
                // Layout transition not yet supported
                unimplemented!();
            }
        };

        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);

        let barrier_info = vk::ImageMemoryBarrier::builder()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(*subresource_range)
            .src_access_mask(sync_info.src_access_mask)
            .dst_access_mask(sync_info.dst_access_mask);

        unsafe {
            logical_device.cmd_pipeline_barrier(
                command_buffer,
                sync_info.src_stage,
                sync_info.dst_stage,
                vk::DependencyFlags::BY_REGION,
                &[],
                &[],
                &[*barrier_info],
            ); //TODO: Can remove build() by using *?
        }
    })
}

pub fn copy_buffer_to_image(
    logical_device: &ash::Device,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
    buffer: vk::Buffer,
    image: vk::Image,
    extent: &vk::Extent3D,
) -> VkResult<()> {
    util::submit_single_use_command_buffer(logical_device, queue, command_pool, |command_buffer| {
        let image_subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(0)
            .base_array_layer(0)
            .layer_count(1);

        let image_copy = vk::BufferImageCopy::builder()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(*image_subresource)
            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(*extent);

        unsafe {
            logical_device.cmd_copy_buffer_to_image(
                command_buffer,
                buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[*image_copy],
            );
        }
    })
}
