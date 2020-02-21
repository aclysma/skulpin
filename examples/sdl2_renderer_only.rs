// This example shows how to use the renderer directly. This allows full control of winit
// and the update loop

use skulpin::{CoordinateSystemHelper, RendererBuilder, PresentMode, CoordinateSystem, LogicalSize, Sdl2Window};
use std::time::Instant;
use sdl2::event::Event;
use sdl2::keyboard::Mod;
use sdl2::keyboard::Keycode;

fn main() {
    // Setup logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    // Setup SDL
    let sdl_context = sdl2::init().expect("Failed to initialize sdl2");
    let video_subsystem = sdl_context.video().expect("Failed to create sdl video subsystem");

    // Set up the coordinate system to be fixed at 900x600, and use this as the default window size
    // This means the drawing code can be written as though the window is always 900x600. The
    // output will be automatically scaled so that it's always visible.
    let logical_size = LogicalSize {
        width: 900,
        height: 600
    };
    let scale_to_fit = skulpin::skia_safe::matrix::ScaleToFit::Center;
    let visible_range = skulpin::skia_safe::Rect {
        left: 0.0,
        right: logical_size.width as f32,
        top: 0.0,
        bottom: logical_size.height as f32,
    };

    let mut sdl_window = video_subsystem.window("Skulpin", logical_size.width, logical_size.height)
        .position_centered()
        .allow_highdpi()
        .resizable()
        .vulkan()
        .build()
        .expect("Failed to create window");
    log::info!("window created");

    let window = Sdl2Window::new(&sdl_window);

    let mut renderer = RendererBuilder::new()
        .use_vulkan_debug_layer(true)
        .coordinate_system(skulpin::CoordinateSystem::VisibleRange(
            visible_range,
            scale_to_fit,
        ))
        .build(&window);

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
    let mut event_pump = sdl_context.event_pump().expect("Could not create sdl event pump");

    'running: loop {
        let frame_start = Instant::now();

        let mut ignore_text_input = false;
        for event in event_pump.poll_iter() {
            log::info!("{:?}", event);
            match event {
                //
                // Halt if the user requests to close the window
                //
                Event::Quit {..} => break 'running,

                //
                // Close if the escape key is hit
                //
                Event::KeyDown { keycode: Some(keycode), keymod: modifiers, .. } => {
                    log::info!("Key Down {:?} {:?}", keycode, modifiers);
                    if keycode == Keycode::Escape {
                        break 'running;
                    }
                },

                _ => {}
            }
        }

        //
        // Redraw
        //
        renderer.draw(&window, |canvas, coordinate_system_helper| {
            draw(canvas, coordinate_system_helper, frame_count);
            frame_count += 1;
        });
    }
}

/// Called when winit passes us a WindowEvent::RedrawRequested
fn draw(
    canvas: &mut skia_safe::Canvas,
    _coordinate_system_helper: &CoordinateSystemHelper,
    frame_count: i32,
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
        &paint,
    );

    // Draw a circle
    canvas.draw_circle(
        skia_safe::Point::new(200.0 + (f * 500.0), 420.0),
        50.0,
        &paint,
    );

    // Draw a rectangle
    canvas.draw_rect(
        skia_safe::Rect {
            left: 10.0,
            top: 10.0,
            right: 890.0,
            bottom: 590.0,
        },
        &paint,
    );

    //TODO: draw_bitmap

    let mut font = skia_safe::Font::default();
    font.set_size(100.0);

    canvas.draw_str("Hello Skulpin", (65, 200), &font, &paint);
    canvas.draw_str("Hello Skulpin", (68, 203), &font, &paint);
    canvas.draw_str("Hello Skulpin", (71, 206), &font, &paint);
}
