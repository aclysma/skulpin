#[cfg(not(target_os = "macos"))]
mod vulkan;
#[cfg(not(target_os = "macos"))]
pub use vulkan::{VkSkiaContext as SkiaContext, VkSkiaSurface as SkiaSurface};

#[cfg(target_os = "macos")]
mod metal;
#[cfg(target_os = "macos")]
pub use metal::{MtlSkiaContext as SkiaContext, MtlSkiaSurface as SkiaSurface};
