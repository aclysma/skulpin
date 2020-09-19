// This example shows how to use the renderer with glfw directly.
use skulpin::{skia_safe, LogicalSize, RendererBuilder, CoordinateSystemHelper};

use skulpin::glfw;
use glfw::{Context, WindowEvent, Key};

fn main() {
    // Setup logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

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

    let mut glfw_window = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let (mut window, events) = glfw_window
        .create_window(900, 600, "Skulpin", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.make_current();

    let renderer = RendererBuilder::new()
        .use_vulkan_debug_layer(false)
        .coordinate_system(skulpin::CoordinateSystem::VisibleRange(
            visible_range,
            scale_to_fit,
        ))
        .build(&skulpin_renderer_glfw::GlfwWindow::new(&window));

    // Check if there were error setting up vulkan
    if let Err(e) = renderer {
        println!("Error during renderer construction: {:?}", e);
        return;
    }

    let mut renderer = renderer.unwrap();

    log::info!("renderer created");

    let mut frame_count = 0;

    while !window.should_close() {
        glfw_window.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            log::info!("{:?}", event);
            match event {
                //
                // Halt if the user requests to close the window
                //
                WindowEvent::Close => window.set_should_close(true),

                //
                // Close if the escape key is hit
                //
                WindowEvent::Key(key, _scancode, action, _modifiers) => {
                    log::info!("Key Down {:?} {:?}", key, action);
                    if key == Key::Escape {
                        window.set_should_close(true)
                    }
                }

                _ => {}
            }
        }

        //
        // Redraw
        //
        renderer
            .draw(
                &skulpin_renderer_glfw::GlfwWindow::new(&window),
                |canvas, coordinate_system_helper| {
                    draw(canvas, &coordinate_system_helper, frame_count);
                    frame_count += 1;
                },
            )
            .unwrap();
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
