
#[macro_use]
extern crate log;

#[macro_use]
extern crate num_derive;

#[macro_use]
extern crate strum_macros;

mod app_control;
pub use app_control::AppControl;

mod renderer;
pub use renderer::RendererBuilder;
pub use renderer::Renderer;

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
        .with_title("Skulpin")
        .with_inner_size(winit::dpi::LogicalSize::new(1300.0, 900.0))
        .build(&event_loop)?;

    let mut app_control = AppControl::default();
    let mut input_state = InputState::default();
    let mut input_handler = WinitInputHandler::new();
    let mut time_state = TimeState::default();
    let mut renderer = Renderer::new(&window, false);

    // To print fps once per second
    let mut print_fps_event = PeriodicEvent::default();

    // Pass control of this thread to winit until the app terminates. If this app wants to quit,
    // the update loop should send the appropriate event via the channel
    event_loop.run(move |event, window_target, control_flow| {
        //println!("enter");
        match event {
            winit::event::Event::EventsCleared => {

                time_state.update(TimeContext::System);

                if print_fps_event.try_take_event(
                    time_state.system().frame_start_instant,
                    std::time::Duration::from_secs_f32(1.0)
                ) {
                    info!("fps: {}", time_state.system().fps);
                }

                update(
                    &mut app_control,
                    &mut input_state,
                    &time_state);

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
                renderer.draw(&window, |canvas| {
                    draw(
                        &app_control,
                        &input_state,
                        &time_state,
                        canvas);
                });
            },
            _ => input_handler.handle_input(
                &mut app_control,
                &mut input_state,
                event,
                window_target)
        }

        if app_control.should_terminate_process() {
            *control_flow = winit::event_loop::ControlFlow::Exit
        }
    });
}

fn update(
    _app_control: &mut AppControl,
    _input_state: &InputState,
    _time_state: &TimeState
) {

}

fn draw(
    _app_control: &AppControl,
    _input_state: &InputState,
    time_state: &TimeState,
    canvas: &mut skia_safe::Canvas,
) {
    canvas.clear(skia_safe::Color::from_argb(0, 0, 0, 255));

    // Floating point value constantly moving between 0..1 to generate some movement
    let f = ((time_state.system().frame_count as f32 / 30.0).sin() + 1.0) / 2.0;

    let mut paint = skia_safe::Paint::new(skia_safe::Color4f::new(1.0 - f, 0.0, f, 1.0), None);
    paint.set_anti_alias(true);
    paint.set_style(skia_safe::paint::Style::Stroke);
    paint.set_stroke_width(3.0);

    // Draw a line
    canvas.draw_line(
        skia_safe::Point::new(100.0, 600.0),
        skia_safe::Point::new(1300.0, 600.0),
        &paint
    );

    canvas.draw_circle(
        skia_safe::Point::new(
            100.0 + (f * 300.0),
            50.0 + (f * 300.0)
        ),
        50.0,
        &paint);

    let rect = skia_safe::Rect {
        left: 10.0,
        top: 10.0,
        right: 500.0,
        bottom: 500.0
    };
    canvas.draw_rect(rect, &paint);

    let mut paint_green = skia_safe::Paint::new(skia_safe::Color4f::new(0.0, 1.0, 0.0, 1.0), None);
    paint_green.set_anti_alias(true);
    paint_green.set_style(skia_safe::paint::Style::Stroke);
    paint_green.set_stroke_width(3.0);


    //canvas.draw_text();
    //TODO: draw_str
    //TODO: draw_bitmap

    let mut font = skia_safe::Font::default();
    font.set_size(500.0);

    canvas.draw_str("Here is a string", (130, 500), &font, &paint);
    canvas.draw_str("Here is a string", (150, 500), &font, &paint);
    canvas.draw_str("Here is a string", (160, 500), &font, &paint);
    canvas.draw_str("Here is a string", (170, 500), &font, &paint);
    canvas.draw_str("Here is a string", (180, 500), &font, &paint);
    canvas.draw_str("Here is a string", (190, 500), &font, &paint);
    canvas.draw_str("Here is a string", (200, 500), &font, &paint);
    canvas.draw_str("Here is a string", (210, 500), &font, &paint);
    canvas.draw_str("Here is a string", (220, 500), &font, &paint);
    canvas.draw_str("Here is a string", (230, 500), &font, &paint);
}
