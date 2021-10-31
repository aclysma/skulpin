#[macro_use]
extern crate log;

mod app;
pub use app::App;
pub use app::AppHandler;
pub use app::AppBuilder;
pub use app::AppError;
pub use app::AppUpdateArgs;
pub use app::AppDrawArgs;

mod app_control;
pub use app_control::AppControl;

mod input_state;
pub use input_state::InputState;
pub use input_state::MouseDragState;

// These are re-exported winit types
pub use input_state::VirtualKeyCode;
pub use input_state::MouseButton;
pub use input_state::MouseScrollDelta;
pub use input_state::ElementState;
pub use input_state::LogicalSize;
pub use input_state::PhysicalSize;
pub use input_state::LogicalPosition;
pub use input_state::PhysicalPosition;
pub use input_state::Position;
pub use input_state::Size;

mod time_state;
pub use time_state::TimeState;
pub use time_state::TimeContext;

mod util;
pub use util::PeriodicEvent;
pub use util::ScopeTimer;

pub use skulpin_renderer::rafx;
pub use skulpin_renderer::skia_safe;

#[cfg(feature = "winit-21")]
pub use winit_21 as winit;
#[cfg(feature = "winit-22")]
pub use winit_22 as winit;
#[cfg(feature = "winit-23")]
pub use winit_23 as winit;
#[cfg(feature = "winit-24")]
pub use winit_24 as winit;
#[cfg(feature = "winit-25")]
pub use winit_25 as winit;
#[cfg(feature = "winit-latest")]
pub use winit_latest as winit;
