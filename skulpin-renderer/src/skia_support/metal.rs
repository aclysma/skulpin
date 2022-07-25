use rafx::api::*;
use rafx::framework::*;
use rafx::api::metal::RafxRawImageMetal;
use skia_safe::gpu::mtl;
use foreign_types_shared::{ForeignType, ForeignTypeRef};

pub struct MtlSkiaContext {
    pub context: skia_safe::gpu::DirectContext,
}

impl MtlSkiaContext {
    pub fn new(
        device_context: &RafxDeviceContext,
        queue: &RafxQueue,
    ) -> Self {
        let mtl_device_context = device_context.metal_device_context().unwrap();
        let mtl_queue_handle = queue.metal_queue().unwrap();
        let mtl_device = mtl_device_context.device();

        let backend_context = unsafe {
            mtl::BackendContext::new(
                mtl_device.as_ptr() as mtl::Handle,
                mtl_queue_handle.metal_queue().as_ptr() as mtl::Handle,
                std::ptr::null(),
            )
        };

        let context = skia_safe::gpu::DirectContext::new_metal(&backend_context, None).unwrap();

        MtlSkiaContext { context }
    }
}

pub struct MtlSkiaSurface {
    pub device_context: RafxDeviceContext,
    pub image_view: ResourceArc<ImageViewResource>,
    pub surface: skia_safe::Surface,
    pub texture: skia_safe::gpu::BackendTexture,
}

impl MtlSkiaSurface {
    pub fn get_image_from_skia_texture(
        texture: &skia_safe::gpu::BackendTexture
    ) -> metal_rs::Texture {
        unsafe { std::mem::transmute(texture.metal_texture_info().unwrap().texture()) }
    }

    pub fn new(
        resource_manager: &ResourceManager,
        context: &mut MtlSkiaContext,
        extents: RafxExtents2D,
    ) -> RafxResult<Self> {
        assert!(extents.width > 0);
        assert!(extents.height > 0);
        // The "native" color type is based on platform. For example, on Windows it's BGR and on
        // MacOS it's RGB
        let color_type = skia_safe::ColorType::N32;
        let alpha_type = skia_safe::AlphaType::Premul;
        let color_space = Some(skia_safe::ColorSpace::new_srgb_linear());

        let image_info = skia_safe::ImageInfo::new(
            (extents.width as i32, extents.height as i32),
            color_type,
            alpha_type,
            color_space,
        );

        let mut surface = skia_safe::Surface::new_render_target(
            &mut context.context,
            skia_safe::Budgeted::Yes,
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
            skia_safe::ColorType::RGBA8888 => RafxFormat::R8G8B8A8_UNORM,
            skia_safe::ColorType::BGRA8888 => RafxFormat::B8G8R8A8_UNORM,
            _ => {
                warn!("Unexpected native color type {:?}", color_type);
                RafxFormat::R8G8B8A8_UNORM
            }
        };

        let device_context = resource_manager.device_context();

        let raw_image = RafxRawImageMetal::Owned(image);

        let image = rafx::api::metal::RafxTextureMetal::from_existing(
            device_context.metal_device_context().unwrap(),
            Some(raw_image),
            &RafxTextureDef {
                extents: RafxExtents3D {
                    width: extents.width,
                    height: extents.height,
                    depth: 1,
                },
                format,
                resource_type: RafxResourceType::TEXTURE,
                sample_count: RafxSampleCount::SampleCount1,
                ..Default::default()
            },
        )?;

        let image = resource_manager
            .resources()
            .insert_image(RafxTexture::Metal(image));
        let image_view = resource_manager
            .resources()
            .get_or_create_image_view(&image, None)?;

        Ok(MtlSkiaSurface {
            device_context: device_context.clone(),
            surface,
            texture,
            image_view,
        })
    }
}
