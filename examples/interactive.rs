// This example shows a bit more interaction with mouse input

use skulpin::AppHandler;
use skulpin::CoordinateSystemHelper;
use skulpin::AppControl;
use skulpin::InputState;
use skulpin::TimeState;
use skulpin::MouseButton;
use skulpin::VirtualKeyCode;
use skulpin::LogicalPosition;
use skulpin::PhysicalPosition;
use skulpin::LogicalSize;

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
        .logical_size(LogicalSize::new(900.0, 600.0))
        .run(example_app);
}

struct PreviousClick {
    position: LogicalPosition<f64>,
    time: std::time::Instant,
}

impl PreviousClick {
    fn new(
        position: LogicalPosition<f64>,
        time: std::time::Instant,
    ) -> Self {
        PreviousClick { position, time }
    }
}

struct ExampleApp {
    last_fps_text_change: Option<std::time::Instant>,
    fps_text: String,
    previous_clicks: VecDeque<PreviousClick>,
}

impl ExampleApp {
    pub fn new() -> Self {
        ExampleApp {
            last_fps_text_change: None,
            fps_text: "".to_string(),
            previous_clicks: VecDeque::new(),
        }
    }
}

impl AppHandler for ExampleApp {
    fn update(
        &mut self,
        app_control: &mut AppControl,
        input_state: &InputState,
        time_state: &TimeState,
    ) {
        let now = time_state.current_instant();

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
            Some(last_update_instant) => (now - last_update_instant).as_secs_f32() >= 1.0,
            None => true,
        };

        if update_text_string {
            let fps = time_state.updates_per_second();
            self.fps_text = format!("Fps: {:.1}", fps);
            self.last_fps_text_change = Some(now);
        }

        //
        // Pop old clicks from the previous_clicks list
        //
        while !self.previous_clicks.is_empty()
            && (now - self.previous_clicks[0].time).as_secs_f32() >= 1.0
        {
            self.previous_clicks.pop_front();
        }

        //
        // Push new clicks onto the previous_clicks list
        //
        if input_state.is_mouse_just_down(MouseButton::Left) {
            let previous_click = PreviousClick::new(input_state.mouse_position(), now);

            self.previous_clicks.push_back(previous_click);
        }
    }

    fn draw(
        &mut self,
        _app_control: &AppControl,
        input_state: &InputState,
        time_state: &TimeState,
        canvas: &mut skia_safe::Canvas,
        _coordinate_system_helper: &CoordinateSystemHelper,
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
        let mouse_position = input_state.mouse_position();
        canvas.draw_circle(
            skia_safe::Point::new(mouse_position.x as f32, mouse_position.y as f32),
            15.0,
            &paint,
        );

        //
        // Draw previous mouse clicks
        //
        for previous_click in &self.previous_clicks {
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
        if let Some(drag) = input_state.mouse_drag_in_progress(MouseButton::Left) {
            let begin_position = drag.begin_position;
            let end_position = drag.end_position;

            canvas.draw_line(
                skia_safe::Point::new(begin_position.x as f32, begin_position.y as f32),
                skia_safe::Point::new(end_position.x as f32, end_position.y as f32),
                &paint,
            );
        }

        //
        // Draw FPS text
        //
        let mut text_paint =
            skia_safe::Paint::new(skia_safe::Color4f::new(1.0, 1.0, 0.0, 1.0), None);
        text_paint.set_anti_alias(true);
        text_paint.set_style(skia_safe::paint::Style::StrokeAndFill);
        text_paint.set_stroke_width(1.0);

        let mut font = skia_safe::Font::default();
        font.set_size(20.0);
        canvas.draw_str(self.fps_text.clone(), (50, 50), &font, &text_paint);
        canvas.draw_str("Click and drag the mouse", (50, 80), &font, &text_paint);
        canvas.draw_str(
            format!("dpi factor: {}", input_state.dpi_factor()),
            (50, 110),
            &font,
            &text_paint,
        );
        let physical_mouse_position: PhysicalPosition<f64> = input_state
            .mouse_position()
            .to_physical(input_state.dpi_factor());
        canvas.draw_str(
            format!(
                "mouse L: ({:.1} {:.1}) P: ({:.1} {:.1})",
                input_state.mouse_position().x,
                input_state.mouse_position().y,
                physical_mouse_position.x,
                physical_mouse_position.y
            ),
            (50, 140),
            &font,
            &text_paint,
        );
    }

    fn fatal_error(
        &mut self,
        error: &skulpin::AppError,
    ) {
        println!("{}", error);
    }
}
