use ash::vk;
use ash::prelude::VkResult;

use super::VkInstance;
use super::VkDevice;

use std::ffi::c_void;

use ash::version::EntryV1_0;
use ash::version::DeviceV1_0;
use ash::version::InstanceV1_0;

/// Handles setting up skia to use the same vulkan instance we initialize
pub struct VkSkiaContext {
    pub context: skia_safe::gpu::Context,
}

impl VkSkiaContext {
    pub fn new(
        instance: &VkInstance,
        device: &VkDevice,
    ) -> Self {
        use vk::Handle;

        let get_proc = |of| unsafe {
            match Self::get_proc(instance, of) {
                Some(f) => f as _,
                None => {
                    error!("resolve of {} failed", of.name().to_str().unwrap());
                    std::ptr::null()
                }
            }
        };

        info!(
            "Setting up skia backend context with queue family index {}",
            device.queue_family_indices.graphics_queue_family_index
        );

        let backend_context = unsafe {
            skia_safe::gpu::vk::BackendContext::new(
                instance.instance.handle().as_raw() as _,
                device.physical_device.as_raw() as _,
                device.logical_device.handle().as_raw() as _,
                (
                    device.queues.graphics_queue.as_raw() as _,
                    device.queue_family_indices.graphics_queue_family_index as usize,
                ),
                &get_proc,
            )
        };

        let context = skia_safe::gpu::Context::new_vulkan(&backend_context).unwrap();

        VkSkiaContext { context }
    }

    unsafe fn get_proc(
        instance: &VkInstance,
        of: skia_safe::gpu::vk::GetProcOf,
    ) -> Option<unsafe extern "system" fn() -> c_void> {
        use vk::Handle;

        match of {
            skia_safe::gpu::vk::GetProcOf::Instance(instance_proc, name) => {
                let ash_instance = vk::Instance::from_raw(instance_proc as _);
                instance.entry.get_instance_proc_addr(ash_instance, name)
            }
            skia_safe::gpu::vk::GetProcOf::Device(device_proc, name) => {
                let ash_device = vk::Device::from_raw(device_proc as _);
                instance.instance.get_device_proc_addr(ash_device, name)
            }
        }
    }
}

/// Wraps a skia surface/canvas that can be drawn on and makes the vulkan resources accessible
pub struct VkSkiaSurface {
    pub device: ash::Device, // VkDevice is responsible for cleaning this up

    pub surface: skia_safe::Surface,
    pub texture: skia_safe::gpu::BackendTexture,
    pub image_view: vk::ImageView,
}

impl VkSkiaSurface {
    pub fn get_image_from_skia_texture(texture: &skia_safe::gpu::BackendTexture) -> vk::Image {
        unsafe { std::mem::transmute(texture.vulkan_image_info().unwrap().image) }
    }

    pub fn new(
        device: &VkDevice,
        context: &mut VkSkiaContext,
        extent: vk::Extent2D,
    ) -> VkResult<Self> {
        // The "native" color type is based on platform. For example, on Windows it's BGR and on
        // MacOS it's RGB
        let color_type = skia_safe::ColorType::n32();
        let alpha_type = skia_safe::AlphaType::Premul;
        let color_space = None;

        let image_info = skia_safe::ImageInfo::new(
            (extent.width as i32, extent.height as i32),
            color_type,
            alpha_type,
            color_space,
        );

        let mut surface = skia_safe::Surface::new_render_target(
            &mut context.context,
            skia_safe::Budgeted::YES,
            &image_info,
            None,
            skia_safe::gpu::SurfaceOrigin::TopLeft,
            None,
            false,
        )
        .unwrap();

        let texture = surface
            .get_backend_texture(skia_safe::surface::BackendHandleAccess::FlushRead)
            .as_ref()
            .unwrap()
            .clone();
        let image = Self::get_image_from_skia_texture(&texture);

        // According to docs, kN32_SkColorType can only be kRGBA_8888_SkColorType or
        // kBGRA_8888_SkColorType. Whatever it is, we need to set up the image view with the
        // matching format
        let format = match color_type {
            skia_safe::ColorType::RGBA8888 => vk::Format::R8G8B8A8_UNORM,
            skia_safe::ColorType::BGRA8888 => vk::Format::B8G8R8A8_UNORM,
            _ => {
                warn!("Unexpected native color type {:?}", color_type);
                vk::Format::R8G8B8A8_UNORM
            }
        };

        let skia_tex_image_view_info = vk::ImageViewCreateInfo {
            view_type: vk::ImageViewType::TYPE_2D,
            format,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::G,
                b: vk::ComponentSwizzle::B,
                a: vk::ComponentSwizzle::A,
            },
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                level_count: 1,
                layer_count: 1,
                ..Default::default()
            },
            image,
            ..Default::default()
        };

        let image_view = unsafe {
            device
                .logical_device
                .create_image_view(&skia_tex_image_view_info, None)?
        };

        Ok(VkSkiaSurface {
            device: device.logical_device.clone(),
            surface,
            texture,
            image_view,
        })
    }

    /// Creates a sampler appropriate for rendering skia surfaces. We don't create one per surface
    /// since one can be shared among all code that renders surfaces
    pub fn create_sampler(logical_device: &ash::Device) -> VkResult<vk::Sampler> {
        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::MIRRORED_REPEAT)
            .address_mode_v(vk::SamplerAddressMode::MIRRORED_REPEAT)
            .address_mode_w(vk::SamplerAddressMode::MIRRORED_REPEAT)
            .anisotropy_enable(false)
            .max_anisotropy(1.0)
            .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::NEVER)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .mip_lod_bias(0.0)
            .min_lod(0.0)
            .max_lod(0.0);

        unsafe { logical_device.create_sampler(&sampler_info, None) }
    }
}

impl Drop for VkSkiaSurface {
    fn drop(&mut self) {
        unsafe {
            //self.device.destroy_sampler(self.sampler, None);
            self.device.destroy_image_view(self.image_view, None);
        }
    }
}
