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

mod app;
pub use app::AppError;
pub use app::InputState;
pub use app::MouseDragState;

// These are re-exported winit types
pub use app::VirtualKeyCode; // This is the same type as winit::event::VirtualKeyCode
pub use app::MouseButton; // This is the same type as winit::event::MouseButton
pub use app::ElementState; // This is the same type as winit::event::ElementState
pub use app::LogicalSize; // This is the same type as winit::dpi::LogicalSize
pub use app::PhysicalSize; // This is the same type as winit::dpi::PhysicalSize
pub use app::LogicalPosition; // This is the same type as winit::dpi::LogicalPosition
pub use app::PhysicalPosition; // This is the same type as winit::dpi::PhysicalPosition

pub use app::TimeState;
pub use app::AppControl;
pub use app::PeriodicEvent;
pub use app::ScopeTimer;
pub use app::App;
pub use app::AppBuilder;
pub use app::AppHandler;
pub use app::AppUpdateArgs;
pub use app::AppDrawArgs;

mod renderer;
pub use renderer::RendererBuilder;
pub use renderer::Renderer;
pub use renderer::PresentMode;
pub use renderer::PhysicalDeviceType;
pub use renderer::CoordinateSystemHelper;
pub use renderer::CoordinateSystem;
pub use renderer::CreateRendererError;

#[cfg(feature = "with_imgui")]
pub use renderer::ImguiManager;

// Export these crates so that downstream crates can easily use the same version of them as we do
pub use ash;
pub use skia_safe;
pub use winit;

#[cfg(feature = "with_imgui")]
pub use imgui;
