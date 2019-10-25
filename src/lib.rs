
#[macro_use]
extern crate log;

#[macro_use]
extern crate num_derive;

#[macro_use]
extern crate strum_macros;

mod app_control;
pub use app_control::AppControl;

mod renderer;
use renderer::Renderer;

mod input_state;
pub use input_state::InputState;

mod winit_input_handler;
pub use winit_input_handler::WinitInputHandler;

mod time_state;
pub use time_state::TimeState;
pub use time_state::TimeContext;

mod util;
pub use util::ScopeTimer;
pub use util::PeriodicEvent;

pub fn run() -> Result<(), Box<dyn std::error::Error>>  {
    // Create the event loop
    let event_loop = winit::event_loop::EventLoop::<()>::with_user_event();

    // Create a single window
    let window = winit::window::WindowBuilder::new()
        .with_title("Vulkan Tutorial")
        .with_inner_size(winit::dpi::LogicalSize::new(1300.0, 900.0))
        .build(&event_loop)?;

    let mut app_control = AppControl::default();
    let mut input_state = InputState::default();
    let mut input_handler = WinitInputHandler::new();
    let mut time_state = TimeState::default();
    let mut renderer = Renderer::new(&window);

    // To print fps once per second
    let mut print_fps_event = PeriodicEvent::default();

    // Pass control of this thread to winit until the app terminates. If this app wants to quit,
    // the update loop should send the appropriate event via the channel
    event_loop.run(move |event, window_target, control_flow| {
        //println!("enter");
        match event {
            winit::event::Event::EventsCleared => {

                time_state.update(TimeContext::System);

                if print_fps_event.try_take_event(time_state.system().frame_start_instant, std::time::Duration::from_secs_f32(1.0)) {
                    info!("fps: {}", time_state.system().fps);
                }

                update(&mut app_control, &mut input_state, &time_state);

                // Call this to mark the start of the next frame (i.e. "key just down" will return false)
                input_state.end_frame();

                //println!("events cleared");
                // Queue a RedrawRequested event.
                window.request_redraw();
            },
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::RedrawRequested,
                ..
            } => {
                renderer.draw(&window, &time_state, &input_state);

            },
            _ => input_handler.handle_input(&mut app_control, &mut input_state, event, window_target)
        }

        if app_control.should_terminate_process() {
            *control_flow = winit::event_loop::ControlFlow::Exit
        }
    });
}

fn update(
    _app_control: &mut AppControl,
    _input_state: &mut InputState,
    _time_state: &TimeState
) {

}

