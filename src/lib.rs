
#[macro_use]
extern crate log;

mod app;
pub use app::InputState;
pub use app::MouseButton; // This is the same type as winit::event::MouseButton
pub use app::VirtualKeyCode; // This is the same type as winit::event::VirtualKeyCode
pub use app::ElementState; // This is the same type as winit::event::ElementState
pub use app::MouseDragState;
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

// Export these crates so that downstream crates can easily use the same version of them as we do
pub use ash;
pub use glam;
pub use skia_safe;
pub use winit;
