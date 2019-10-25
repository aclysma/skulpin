

use std::io;
use ash::vk;
use ash::version::DeviceV1_0;

pub fn find_memorytype_index(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    required_property_flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    for (index, ref memory_type) in memory_prop.memory_types.iter().enumerate() {
        let type_supported = (memory_req.memory_type_bits & (1<<index)) != 0;
        let flags_supported = (memory_type.property_flags & required_property_flags) == required_property_flags;

        if type_supported && flags_supported {
            return Some(index as u32);
        }
    }

    None
}

pub fn read_spv<R: io::Read + io::Seek>(x: &mut R) -> io::Result<Vec<u32>> {
    let size = x.seek(io::SeekFrom::End(0))?;
    if size % 4 != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "input length not divisible by 4",
        ));
    }
    if size > usize::max_value() as u64 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "input too long"));
    }
    let words = (size / 4) as usize;
    let mut result = Vec::<u32>::with_capacity(words);
    x.seek(io::SeekFrom::Start(0))?;
    unsafe {
        x.read_exact(std::slice::from_raw_parts_mut(
            result.as_mut_ptr() as *mut u8,
            words * 4,
        ))?;
        result.set_len(words);
    }
    const MAGIC_NUMBER: u32 = 0x07230203;
    if result.len() > 0 && result[0] == MAGIC_NUMBER.swap_bytes() {
        for word in &mut result {
            *word = word.swap_bytes();
        }
    }
    if result.len() == 0 || result[0] != MAGIC_NUMBER {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "input missing SPIR-V magic number",
        ));
    }
    Ok(result)
}

pub fn submit_single_use_command_buffer<F : Fn(&vk::CommandBuffer)>(
    logical_device: &ash::Device,
    queue: &vk::Queue,
    command_pool: &vk::CommandPool,
    f: F
) {
    let alloc_info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(*command_pool)
        .command_buffer_count(1);

    let begin_info = vk::CommandBufferBeginInfo::builder()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    let command_buffer = unsafe {
        let command_buffer = logical_device.allocate_command_buffers(&alloc_info).unwrap()[0];

        logical_device
            .begin_command_buffer(command_buffer, &begin_info)
            .expect("Begin commandbuffer");

        f(&command_buffer);

        logical_device.end_command_buffer(command_buffer).unwrap();

        command_buffer
    };

    let command_buffers = [command_buffer];
    let submit_info = vk::SubmitInfo::builder()
        .command_buffers(&command_buffers);

    unsafe {
        logical_device.queue_submit(*queue, &[submit_info.build()], vk::Fence::null()).unwrap();
        logical_device.device_wait_idle().unwrap();

        logical_device.free_command_buffers(*command_pool, &command_buffers);
    }
}
/*
pub fn transition_image_layout(
    logical_device: &ash::Device,
    queue: &vk::Queue,
    command_pool: &vk::CommandPool,
    image: &vk::Image,
    _format: vk::Format,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout
) {
    super::util::submit_single_use_command_buffer(logical_device, queue, command_pool, |command_buffer| {

        struct SyncInfo {
            src_access_mask: vk::AccessFlags,
            dst_access_mask: vk::AccessFlags,
            src_stage: vk::PipelineStageFlags,
            dst_stage: vk::PipelineStageFlags,
        }

        let sync_info = match (old_layout, new_layout) {
            (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => {
                SyncInfo {
                    src_access_mask: vk::AccessFlags::empty(),
                    dst_access_mask: vk::AccessFlags::TRANSFER_WRITE,
                    src_stage: vk::PipelineStageFlags::TOP_OF_PIPE,
                    dst_stage: vk::PipelineStageFlags::TRANSFER,
                }
            },
            (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => {
                SyncInfo {
                    src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
                    dst_access_mask: vk::AccessFlags::SHADER_READ,
                    src_stage: vk::PipelineStageFlags::TRANSFER,
                    dst_stage: vk::PipelineStageFlags::FRAGMENT_SHADER,
                }
            },
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
            .image(*image)
            .subresource_range(*subresource_range)
            .src_access_mask(sync_info.src_access_mask)
            .dst_access_mask(sync_info.dst_access_mask);

        unsafe {
            logical_device.cmd_pipeline_barrier(
                *command_buffer,
                sync_info.src_stage,
                sync_info.dst_stage,
                vk::DependencyFlags::BY_REGION,
                &[],
                &[],
                &[*barrier_info]); //TODO: Can remove build() by using *?
        }
    });
}
*/
/*
pub fn copy_buffer_to_image(
    logical_device: &ash::Device,
    queue: &vk::Queue,
    command_pool: &vk::CommandPool,
    buffer: &vk::Buffer,
    image: &vk::Image,
    extent: &vk::Extent3D
) {
    super::util::submit_single_use_command_buffer(logical_device, queue, command_pool, |command_buffer| {
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
                *command_buffer,
                *buffer,
                *image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[*image_copy]
            );
        }
    });
}
*/