
// This example shows how to use the renderer directly. This allows full control of winit
// and the update loop

fn main() {
    // Setup logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    // Create the winit event loop
    let event_loop = winit::event_loop::EventLoop::<()>::with_user_event();

    // Create a single window
    let window = winit::window::WindowBuilder::new()
        .with_title("Skulpin")
        .with_inner_size(winit::dpi::LogicalSize::new(900.0, 600.0))
        .build(&event_loop)
        .expect("Failed to create window");

    // Create the renderer, which will draw to the window
    let renderer = skulpin::RendererBuilder::new()
        .use_vulkan_debug_layer(true)
        .build(&window);

    // Check if there were error setting up vulkan
    if let Err(e) = renderer {
        println!("Error during renderer construction: {:?}", e);
        return;
    }

    let mut renderer = renderer.unwrap();

    // Increment a frame count so we can render something that moves
    let mut frame_count = 0;

    // Start the window event loop. Winit will not return once run is called. We will get notified
    // when important events happen.
    event_loop.run(move |event, _window_target, control_flow| {
        match event {

            //
            // Halt if the user requests to close the window
            //
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = winit::event_loop::ControlFlow::Exit
            },

            //
            // Close if the escape key is hit
            //
            winit::event::Event::WindowEvent {
                event:
                winit::event::WindowEvent::KeyboardInput {
                    input:
                    winit::event::KeyboardInput {
                        virtual_keycode:
                        Some(winit::event::VirtualKeyCode::Escape),
                        ..
                    },
                    ..
                },
                ..
            } => {
                *control_flow = winit::event_loop::ControlFlow::Exit
            },

            //
            // Request a redraw any time we finish processing events
            //
            winit::event::Event::EventsCleared => {
                // Queue a RedrawRequested event.
                window.request_redraw();
            },

            //
            // Redraw
            //
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::RedrawRequested,
                ..
            } => {
                if let Err(e) = renderer.draw(&window, |canvas| {
                    draw(canvas, frame_count);
                    frame_count += 1;
                }) {
                    println!("Error during draw: {:?}", e);
                    *control_flow = winit::event_loop::ControlFlow::Exit
                }
            },

            //
            // Ignore all other events
            //
            _ => {}
        }
    });
}

/// Called when winit passes us a WindowEvent::RedrawRequested
fn draw(
    canvas: &mut skia_safe::Canvas,
    frame_count: i32
) {
    // Generally would want to clear data every time we draw
    canvas.clear(skia_safe::Color::from_argb(0, 0, 0, 255));

    // Floating point value constantly moving between 0..1 to generate some movement
    let f = ((frame_count as f32 / 30.0).sin() + 1.0) / 2.0;

    // Make a color to draw with
    let mut paint = skia_safe::Paint::new(skia_safe::Color4f::new(1.0 - f, 0.0, f, 1.0), None);
    paint.set_anti_alias(true);
    paint.set_style(skia_safe::paint::Style::Stroke);
    paint.set_stroke_width(2.0);

    // Draw a line
    canvas.draw_line(
        skia_safe::Point::new(100.0, 500.0),
        skia_safe::Point::new(800.0, 500.0),
        &paint
    );

    // Draw a circle
    canvas.draw_circle(
        skia_safe::Point::new(
            200.0 + (f * 500.0),
            420.0
        ),
        50.0,
        &paint
    );

    // Draw a rectangle
    canvas.draw_rect(
        skia_safe::Rect {
            left: 10.0,
            top: 10.0,
            right: 890.0,
            bottom: 590.0
        },
        &paint
    );

    //TODO: draw_bitmap

    let mut font = skia_safe::Font::default();
    font.set_size(100.0);

    canvas.draw_str("Hello Skulpin", (65, 200), &font, &paint);
    canvas.draw_str("Hello Skulpin", (68, 203), &font, &paint);
    canvas.draw_str("Hello Skulpin", (71, 206), &font, &paint);
}