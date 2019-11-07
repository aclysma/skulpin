#[macro_use]
extern crate log;

mod app;
pub use app::InputState;
pub use app::MouseDragState;

// These are re-exported winit types
pub use app::ElementState; // This is the same type as winit::event::ElementState
pub use app::LogicalPosition; // This is the same type as winit::dpi::LogicalPosition
pub use app::LogicalSize; // This is the same type as winit::dpi::LogicalSize
pub use app::MouseButton; // This is the same type as winit::event::MouseButton
pub use app::PhysicalPosition;
pub use app::PhysicalSize; // This is the same type as winit::dpi::PhysicalSize
pub use app::VirtualKeyCode; // This is the same type as winit::event::VirtualKeyCode // This is the same type as winit::dpi::PhysicalPosition

pub use app::App;
pub use app::AppBuilder;
pub use app::AppControl;
pub use app::AppHandler;
pub use app::PeriodicEvent;
pub use app::ScopeTimer;
pub use app::TimeState;

mod renderer;
pub use renderer::Renderer;
pub use renderer::RendererBuilder;

// Export these crates so that downstream crates can easily use the same version of them as we do
pub use ash;
pub use skia_safe;
pub use winit;
