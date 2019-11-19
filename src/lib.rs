
#[macro_use]
extern crate log;

mod app;
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

mod renderer;
pub use renderer::RendererBuilder;
pub use renderer::Renderer;
pub use renderer::PresentMode;
pub use renderer::PhysicalDeviceType;

// Export these crates so that downstream crates can easily use the same version of them as we do
pub use ash;
pub use skia_safe;
pub use winit;
