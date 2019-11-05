
// This example shows a bit more interaction with mouse input

use skulpin::AppHandler;
use skulpin::AppControl;
use skulpin::InputState;
use skulpin::TimeState;
use skulpin::MouseButton;
use skulpin::VirtualKeyCode;
use skulpin::glam;
use std::ffi::CString;
use std::collections::VecDeque;

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

struct PreviousClick {
    position: glam::Vec2,
    time: std::time::Instant
}

impl PreviousClick {
    fn new(
        position: glam::Vec2,
        time: std::time::Instant
    )
        -> Self
    {
        PreviousClick {
            position,
            time
        }
    }
}

struct ExampleApp {
    last_fps_text_change: Option<std::time::Instant>,
    fps_text: String,
    previous_clicks: VecDeque<PreviousClick>
}

impl ExampleApp {
    pub fn new() -> Self {
        ExampleApp {
            last_fps_text_change: None,
            fps_text: "".to_string(),
            previous_clicks: VecDeque::new()
        }
    }
}

impl AppHandler for ExampleApp {
    fn update(
        &mut self,
        app_control: &mut AppControl,
        input_state: &InputState,
        time_state: &TimeState
    ) {
        let now = time_state.system().frame_start_instant;

        //
        // Quit if user hits escape
        //
        if input_state.is_key_down(VirtualKeyCode::Escape) {
            app_control.enqueue_terminate_process();
        }

        //
        // Update FPS once a second
        //
        let update_text_string = match self.last_fps_text_change {
            Some(last_update_instant) => {
                (now - last_update_instant).as_secs_f32() >= 1.0
            },
            None => true
        };

        if update_text_string {
            let fps = time_state.system().fps;
            self.fps_text = format!("Fps: {:.1}", fps);
            self.last_fps_text_change = Some(now);
        }

        while self.previous_clicks.len() > 0 &&
            (now - self.previous_clicks[0].time).as_secs_f32() >= 1.0 {
            self.previous_clicks.pop_front();
        }

        if input_state.is_mouse_just_down(MouseButton::Left) {
            let previous_click = PreviousClick::new(
                input_state.mouse_position(),
                now
            );

            self.previous_clicks.push_back(previous_click);
        }
    }

    fn draw(
        &mut self,
        _app_control: &AppControl,
        input_state: &InputState,
        time_state: &TimeState,
        canvas: &mut skia_safe::Canvas
    ) {
        let now = time_state.system().frame_start_instant;

        // Generally would want to clear data every time we draw
        canvas.clear(skia_safe::Color::from_argb(0, 0, 0, 255));

        // Make a color to draw with
        let mut paint = skia_safe::Paint::new(skia_safe::Color4f::new(0.0, 1.0, 0.0, 1.0), None);
        paint.set_anti_alias(true);
        paint.set_style(skia_safe::paint::Style::Stroke);
        paint.set_stroke_width(3.0);

        //
        // Draw current mouse position
        //
        let mouse_position = input_state.mouse_position();
        canvas.draw_circle(
            skia_safe::Point::new(
                mouse_position.x(),
                mouse_position.y()
            ),
            30.0,
            &paint
        );

        //
        // Draw previous mouse clicks
        //
        for previous_click in &self.previous_clicks {
            let age = now - previous_click.time;
            let age = age.as_secs_f32().min(1.0).max(0.0);

            // Make a color that fades out as the click is further in the past
            let mut paint = skia_safe::Paint::new(skia_safe::Color4f::new(0.0, 1.0 - age, 0.0, 1.0), None);
            paint.set_anti_alias(true);
            paint.set_style(skia_safe::paint::Style::Stroke);
            paint.set_stroke_width(3.0);

            canvas.draw_circle(
                skia_safe::Point::new(
                    previous_click.position.x(),
                    previous_click.position.y()
                ),
                50.0,
                &paint
            );
        }

        //
        // If mouse is being dragged, draw a line to show the drag
        //
        if let Some(drag) = input_state.mouse_drag_in_progress(MouseButton::Left) {
            canvas.draw_line(
                skia_safe::Point::new(drag.begin_position.x(), drag.begin_position.y()),
                skia_safe::Point::new(drag.end_position.x(), drag.end_position.y()),
                &paint
            );
        }

        //
        // Draw FPS text
        //
        let mut text_paint = skia_safe::Paint::new(skia_safe::Color4f::new(1.0, 1.0, 0.0, 1.0), None);
        text_paint.set_anti_alias(true);
        text_paint.set_style(skia_safe::paint::Style::StrokeAndFill);
        text_paint.set_stroke_width(2.0);

        let mut font = skia_safe::Font::default();
        font.set_size(50.0);
        canvas.draw_str(self.fps_text.clone(), (50, 200), &font, &text_paint);

        canvas.draw_str("Click and drag the mouse", (50, 300), &font, &text_paint);
    }
}
