//! Skia + Vulkan = Skulpin
//!
//! This crate provides an easy option for drawing hardware-accelerated 2D by combining vulkan and
//! skia. (And a dash of winit!)
//!
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

#[macro_use]
extern crate log;

pub mod app;

//mod renderer;
pub use skulpin_renderer::RendererBuilder;
pub use skulpin_renderer::Renderer;
pub use skulpin_renderer::PresentMode;
pub use skulpin_renderer::PhysicalDeviceType;
pub use skulpin_renderer::CoordinateSystemHelper;
pub use skulpin_renderer::CoordinateSystem;
pub use skulpin_renderer::CreateRendererError;
pub use skulpin_renderer::Size;
pub use skulpin_renderer::LogicalSize;
pub use skulpin_renderer::PhysicalSize;
pub use skulpin_renderer::Window;

pub use skulpin_renderer_winit::WinitWindow;
pub use skulpin_renderer_sdl2::Sdl2Window;

// Export these crates so that downstream crates can easily use the same version of them as we do
pub use ash;
pub use skia_safe;
pub use sdl2;
