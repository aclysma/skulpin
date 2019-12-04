#[allow(clippy::module_inception)]
mod app;
pub use app::App;
pub use app::AppHandler;
pub use app::AppBuilder;
pub use app::AppError;

mod app_control;
pub use app_control::AppControl;

mod input_state;
pub use input_state::InputState;
pub use input_state::MouseDragState;

// These are re-exported winit types
pub use input_state::VirtualKeyCode;
pub use input_state::MouseButton;
pub use input_state::ElementState;
pub use input_state::LogicalSize;
pub use input_state::PhysicalSize;
pub use input_state::LogicalPosition;
pub use input_state::PhysicalPosition;

mod time_state;
pub use time_state::TimeState;
pub use time_state::TimeContext;

mod util;
pub use util::PeriodicEvent;
pub use util::ScopeTimer;
