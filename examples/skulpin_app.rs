
// This example shows how to use the "app" helpers to get a window open and drawing with minimal code
// It's not as flexible as working with winit directly, but it's quick and simple

use skulpin::AppHandler;
use skulpin::AppControl;
use skulpin::InputState;
use skulpin::TimeState;
use std::ffi::CString;

fn main() {
    // Setup logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let example_app = ExampleApp::new();

    skulpin::AppBuilder::new()
        .app_name(CString::new("Skulpin Example App").unwrap())
        .use_vulkan_debug_layer(true)
        .run(example_app)
        .expect("The app failed with an error");
}

struct ExampleApp {
}

impl ExampleApp {
    pub fn new() -> Self {
        ExampleApp {

        }
    }
}

impl AppHandler for ExampleApp {
    fn update(
        &mut self,
        _app_control: &mut AppControl,
        _input_state: &InputState,
        _time_state: &TimeState
    ) {

    }

    fn draw(
        &mut self,
        _app_control: &AppControl,
        _input_state: &InputState,
        time_state: &TimeState,
        canvas: &mut skia_safe::Canvas
    ) {
        // Generally would want to clear data every time we draw
        canvas.clear(skia_safe::Color::from_argb(0, 0, 0, 255));

        // Floating point value constantly moving between 0..1 to generate some movement
        let f = ((time_state.system().frame_count as f32 / 30.0).sin() + 1.0) / 2.0;

        // Make a color to draw with
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

        // Draw a circle
        canvas.draw_circle(
            skia_safe::Point::new(
                100.0 + (f * 300.0),
                50.0 + (f * 300.0)
            ),
            50.0,
            &paint
        );

        // Draw a rectangle
        canvas.draw_rect(
            skia_safe::Rect {
                left: 10.0,
                top: 10.0,
                right: 1400.0,
                bottom: 500.0
            },
            &paint
        );

        //TODO: draw_bitmap

        let mut font = skia_safe::Font::default();
        font.set_size(200.0);

        canvas.draw_str("Hello Skulpin", (130, 300), &font, &paint);
        canvas.draw_str("Hello Skulpin", (135, 305), &font, &paint);
        canvas.draw_str("Hello Skulpin", (140, 310), &font, &paint);
    }
}
