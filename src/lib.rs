//! Skia + Vulkan = Skulpin
//!
//! This crate provides an easy option for drawing hardware-accelerated 2D by combining vulkan and
//! skia.
//!
//! Two windowing backends are supported out of the box - winit and sdl2.
//!
//! Currently there are two ways to use this library.
//!
//! # skulpin::App
//!
//! Implement the AppHandler trait and launch the app. It's simple but not as flexible as using
//! the renderer directly and handling the window manually.
//!
//! Utility classes are provided that make handling input and measuring time easier.
//!
//! # skulpin::Renderer
//!
//! You manage the window and event loop yourself. Then add the renderer to draw to it.
//!
//! This is the most flexible way to use the library
//!
//!

// Export these crates so that downstream crates can easily use the same version of them as we do
pub use skulpin_renderer::rafx;
pub use skulpin_renderer::skia_safe;
pub use skulpin_renderer::skia_bindings;

pub use skulpin_renderer::RendererBuilder;
pub use skulpin_renderer::RendererPlugin;
pub use skulpin_renderer::Renderer;
pub use skulpin_renderer::CoordinateSystemHelper;
pub use skulpin_renderer::CoordinateSystem;
pub use skulpin_renderer::Size;
pub use skulpin_renderer::LogicalSize;
pub use skulpin_renderer::PhysicalSize;

// #[cfg(feature = "skulpin_winit")]
// pub use skulpin_renderer_winit;
#[cfg(feature = "winit_app")]
pub use skulpin_app_winit as app;
#[cfg(feature = "winit_app")]
pub use skulpin_app_winit::winit;
// #[cfg(feature = "skulpin_winit")]
// pub use skulpin_renderer_winit::winit;
// #[cfg(feature = "skulpin_winit")]
// pub use skulpin_renderer_winit::WinitWindow;
//
// #[cfg(feature = "skulpin_sdl2")]
// pub use skulpin_renderer_sdl2;
// #[cfg(feature = "skulpin_sdl2")]
// pub use skulpin_renderer_sdl2::sdl2;
// #[cfg(feature = "skulpin_sdl2")]
// pub use skulpin_renderer_sdl2::Sdl2Window;
