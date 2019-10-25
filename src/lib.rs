
#[macro_use]
extern crate log;

#[macro_use]
extern crate num_derive;

#[macro_use]
extern crate strum_macros;

mod app;
pub use app::InputState;
pub use app::KeyboardButton;
pub use app::KeyboardButtonEvent;
pub use app::MouseButton;
pub use app::MouseButtonEvent;
pub use app::MouseDragState;
pub use app::TimeState;
pub use app::AppControl;
pub use app::PeriodicEvent;
pub use app::ScopeTimer;
pub use app::run; //TODO: Make this a struct with builder?

mod renderer;
pub use renderer::RendererBuilder;
pub use renderer::Renderer;

