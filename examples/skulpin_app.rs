// This example shows how to use the "app" helpers to get a window open and drawing with minimal code
// It's not as flexible as working with winit directly, but it's quick and simple

use skulpin::CoordinateSystem;
use skulpin::LogicalSize;
use skulpin::skia_safe;

use skulpin::app::AppBuilder;
use skulpin::app::AppUpdateArgs;
use skulpin::app::AppDrawArgs;
use skulpin::app::AppError;
use skulpin::app::AppHandler;
use skulpin::app::VirtualKeyCode;
use std::ffi::CString;

fn main() {
    // Setup logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let example_app = ExampleApp::new();

    // Set up the coordinate system to be fixed at 900x600, and use this as the default window size
    // This means the drawing code can be written as though the window is always 900x600. The
    // output will be automatically scaled so that it's always visible.
    let logical_size = LogicalSize::new(900, 600);
    let visible_range = skulpin::skia_safe::Rect {
        left: 0.0,
        right: logical_size.width as f32,
        top: 0.0,
        bottom: logical_size.height as f32,
    };
    let scale_to_fit = skulpin::skia_safe::matrix::ScaleToFit::Center;

    AppBuilder::new()
        .app_name(CString::new("Skulpin Example App").unwrap())
        .use_vulkan_debug_layer(false)
        .inner_size(logical_size)
        .coordinate_system(CoordinateSystem::VisibleRange(visible_range, scale_to_fit))
        .run(example_app);
}

struct ExampleApp {}

impl ExampleApp {
    pub fn new() -> Self {
        ExampleApp {}
    }
}

impl AppHandler for ExampleApp {
    fn update(&mut self, update_args: AppUpdateArgs) {
        let input_state = update_args.input_state;
        let app_control = update_args.app_control;

        if input_state.is_key_down(VirtualKeyCode::Escape) {
            app_control.enqueue_terminate_process();
        }
    }

    fn draw(&mut self, draw_args: AppDrawArgs) {
        let time_state = draw_args.time_state;
        let canvas = draw_args.canvas;

        // Generally would want to clear data every time we draw
        canvas.clear(skia_safe::Color::from_argb(0, 0, 0, 255));

        // Floating point value constantly moving between 0..1 to generate some movement
        let f = ((time_state.update_count() as f32 / 30.0).sin() + 1.0) / 2.0;

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

    fn fatal_error(&mut self, error: &AppError) {
        println!("{}", error);
    }
}
