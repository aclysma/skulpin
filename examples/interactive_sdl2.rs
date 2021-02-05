// This example uses the SDL2 renderer directly in an interactive way. The `interactive` demo
// that uses the app abstraction is a much cleaner/easier way to do the same thing, but uses winit
// instead.

use skulpin::skia_safe;
use skulpin::{CoordinateSystemHelper, RendererBuilder, LogicalSize};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::collections::VecDeque;
use sdl2::mouse::{MouseState, MouseButton};

use skulpin::app::TimeState;
use skulpin_app_winit::rafx::api::raw_window_handle::HasRawWindowHandle;
use skulpin_app_winit::rafx::api::RafxExtents2D;

#[derive(Clone, Copy)]
struct Position {
    x: i32,
    y: i32,
}

struct PreviousClick {
    position: Position,
    time: std::time::Instant,
}

impl PreviousClick {
    fn new(
        position: Position,
        time: std::time::Instant,
    ) -> Self {
        PreviousClick { position, time }
    }
}

struct ExampleAppState {
    last_fps_text_change: Option<std::time::Instant>,
    fps_text: String,
    current_mouse_state: MouseState,
    previous_mouse_state: MouseState,
    drag_start_position: Option<Position>,
    previous_clicks: VecDeque<PreviousClick>,
}

fn main() {
    // Setup logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    // Setup SDL
    let sdl_context = sdl2::init().expect("Failed to initialize sdl2");
    let video_subsystem = sdl_context
        .video()
        .expect("Failed to create sdl video subsystem");

    // Set up the coordinate system to be fixed at 900x600, and use this as the default window size
    // This means the drawing code can be written as though the window is always 900x600. The
    // output will be automatically scaled so that it's always visible.
    let logical_size = LogicalSize {
        width: 900,
        height: 600,
    };
    let scale_to_fit = skulpin::skia_safe::matrix::ScaleToFit::Center;
    let visible_range = skulpin::skia_safe::Rect {
        left: 0.0,
        right: logical_size.width as f32,
        top: 0.0,
        bottom: logical_size.height as f32,
    };

    let window = video_subsystem
        .window("Skulpin", logical_size.width, logical_size.height)
        .position_centered()
        .allow_highdpi()
        .resizable()
        .build()
        .expect("Failed to create window");
    log::info!("window created");

    let (window_width, window_height) = window.vulkan_drawable_size();

    let extents = RafxExtents2D {
        width: window_width,
        height: window_height
    };

    let renderer = RendererBuilder::new()
        .coordinate_system(skulpin::CoordinateSystem::VisibleRange(
            visible_range,
            scale_to_fit,
        ))
        .build(&window, extents);

    // Check if there were error setting up vulkan
    if let Err(e) = renderer {
        println!("Error during renderer construction: {:?}", e);
        return;
    }

    log::info!("renderer created");

    let mut renderer = renderer.unwrap();

    // Increment a frame count so we can render something that moves
    let mut frame_count = 0;

    log::info!("Starting window event loop");
    let mut event_pump = sdl_context
        .event_pump()
        .expect("Could not create sdl event pump");

    let initial_mouse_state = sdl2::mouse::MouseState::new(&event_pump);

    let mut app_state = ExampleAppState {
        last_fps_text_change: None,
        fps_text: "".to_string(),
        previous_clicks: Default::default(),
        current_mouse_state: initial_mouse_state,
        previous_mouse_state: initial_mouse_state,
        drag_start_position: None,
    };

    let mut time_state = skulpin::app::TimeState::new();

    'running: loop {
        time_state.update();

        for event in event_pump.poll_iter() {
            log::info!("{:?}", event);
            match event {
                //
                // Halt if the user requests to close the window
                //
                Event::Quit { .. } => break 'running,

                Event::MouseButtonDown {
                    mouse_btn, x, y, ..
                } => {
                    //
                    // Push new clicks onto the previous_clicks list
                    //
                    let now = time_state.current_instant();
                    if mouse_btn == MouseButton::Left {
                        let position = Position { x, y };
                        let previous_click = PreviousClick::new(position, now);
                        app_state.previous_clicks.push_back(previous_click);

                        app_state.drag_start_position = Some(position);
                    }
                }

                Event::MouseButtonUp { mouse_btn, .. } => {
                    //
                    // Clear the drag if left mouse is released
                    //
                    if mouse_btn == MouseButton::Left {
                        app_state.drag_start_position = None;
                    }
                }

                //
                // Close if the escape key is hit
                //
                Event::KeyDown {
                    keycode: Some(keycode),
                    keymod: modifiers,
                    ..
                } => {
                    //
                    // Quit if user hits escape
                    //
                    log::info!("Key Down {:?} {:?}", keycode, modifiers);
                    if keycode == Keycode::Escape {
                        break 'running;
                    }
                }

                _ => {}
            }
        }

        app_state.previous_mouse_state = app_state.current_mouse_state;
        app_state.current_mouse_state = MouseState::new(&event_pump);

        update(&mut app_state, &time_state);

        //
        // Redraw
        //
        renderer
            .draw(&window, |canvas, coordinate_system_helper| {
                draw(
                    &app_state,
                    &time_state,
                    canvas,
                    &coordinate_system_helper,
                    &window,
                );
                frame_count += 1;
            })
            .unwrap();
    }
}

fn update(
    app_state: &mut ExampleAppState,
    time_state: &TimeState,
) {
    let now = time_state.current_instant();

    //
    // Update FPS once a second
    //
    let update_text_string = match app_state.last_fps_text_change {
        Some(last_update_instant) => (now - last_update_instant).as_secs_f32() >= 1.0,
        None => true,
    };

    if update_text_string {
        let fps = time_state.updates_per_second();
        app_state.fps_text = format!("Fps: {:.1}", fps);
        app_state.last_fps_text_change = Some(now);
    }

    //
    // Pop old clicks from the previous_clicks list
    //
    while !app_state.previous_clicks.is_empty()
        && (now - app_state.previous_clicks[0].time).as_secs_f32() >= 1.0
    {
        app_state.previous_clicks.pop_front();
    }
}

fn draw(
    app_state: &ExampleAppState,
    time_state: &TimeState,
    canvas: &mut skia_safe::Canvas,
    _coordinate_system_helper: &CoordinateSystemHelper,
    window: &dyn HasRawWindowHandle,
) {
    let now = time_state.current_instant();

    // Generally would want to clear data every time we draw
    canvas.clear(skia_safe::Color::from_argb(0, 0, 0, 255));

    // Make a color to draw with
    let mut paint = skia_safe::Paint::new(skia_safe::Color4f::new(0.0, 1.0, 0.0, 1.0), None);
    paint.set_anti_alias(true);
    paint.set_style(skia_safe::paint::Style::Stroke);
    paint.set_stroke_width(2.0);

    //
    // Draw current mouse position.
    //
    canvas.draw_circle(
        skia_safe::Point::new(
            app_state.current_mouse_state.x() as f32,
            app_state.current_mouse_state.y() as f32,
        ),
        15.0,
        &paint,
    );

    //
    // Draw previous mouse clicks
    //
    for previous_click in &app_state.previous_clicks {
        let age = now - previous_click.time;
        let age = age.as_secs_f32().min(1.0).max(0.0);

        // Make a color that fades out as the click is further in the past
        let mut paint =
            skia_safe::Paint::new(skia_safe::Color4f::new(0.0, 1.0 - age, 0.0, 1.0), None);
        paint.set_anti_alias(true);
        paint.set_style(skia_safe::paint::Style::Stroke);
        paint.set_stroke_width(3.0);

        let position = previous_click.position;

        canvas.draw_circle(
            skia_safe::Point::new(position.x as f32, position.y as f32),
            25.0,
            &paint,
        );
    }

    //
    // If mouse is being dragged, draw a line to show the drag
    //
    if let Some(drag_start_position) = app_state.drag_start_position {
        canvas.draw_line(
            skia_safe::Point::new(drag_start_position.x as f32, drag_start_position.y as f32),
            skia_safe::Point::new(
                app_state.current_mouse_state.x() as f32,
                app_state.current_mouse_state.y() as f32,
            ),
            &paint,
        );
    }

    //
    // Draw FPS text
    //
    let mut text_paint = skia_safe::Paint::new(skia_safe::Color4f::new(1.0, 1.0, 0.0, 1.0), None);
    text_paint.set_anti_alias(true);
    text_paint.set_style(skia_safe::paint::Style::StrokeAndFill);
    text_paint.set_stroke_width(1.0);

    let mut font = skia_safe::Font::default();
    font.set_size(20.0);
    canvas.draw_str(app_state.fps_text.clone(), (50, 50), &font, &text_paint);
    canvas.draw_str("Click and drag the mouse", (50, 80), &font, &text_paint);

    let scale_factor = 1.0;

    canvas.draw_str(
        format!("scale factor: {}", scale_factor),
        (50, 110),
        &font,
        &text_paint,
    );

    let physical_mouse_position = (
        app_state.current_mouse_state.x(),
        app_state.current_mouse_state.y(),
    );
    let logical_mouse_position = (
        physical_mouse_position.0 as f64 / scale_factor,
        physical_mouse_position.1 as f64 / scale_factor,
    );
    canvas.draw_str(
        format!(
            "mouse L: ({:.1} {:.1}) P: ({:.1} {:.1})",
            logical_mouse_position.0,
            logical_mouse_position.1,
            physical_mouse_position.0,
            physical_mouse_position.1
        ),
        (50, 140),
        &font,
        &text_paint,
    );
}
