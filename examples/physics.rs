// This example does a physics demo, because physics is fun :)

extern crate nalgebra as na;

use skulpin::skia_safe;

use skulpin::app::AppHandler;
use skulpin::app::AppUpdateArgs;
use skulpin::app::AppDrawArgs;
use skulpin::app::AppError;
use skulpin::app::AppBuilder;
use skulpin::app::VirtualKeyCode;

use skulpin::LogicalSize;

use std::ffi::CString;

// Used for physics
use na::Vector2;
use ncollide2d::shape::{Cuboid, ShapeHandle, Ball};
use nphysics2d::object::{
    ColliderDesc, RigidBodyDesc, DefaultBodySet, DefaultColliderSet, Ground, BodyPartHandle,
    DefaultBodyHandle,
};
use nphysics2d::force_generator::DefaultForceGeneratorSet;
use nphysics2d::joint::DefaultJointConstraintSet;
use nphysics2d::world::{DefaultMechanicalWorld, DefaultGeometricalWorld};

const GROUND_THICKNESS: f32 = 0.2;
const GROUND_HALF_EXTENTS_WIDTH: f32 = 3.0;
const BALL_RADIUS: f32 = 0.2;
const GRAVITY: f32 = -9.81;
const BALL_COUNT: usize = 5;

// Will contain all the physics simulation state
struct Physics {
    geometrical_world: DefaultGeometricalWorld<f32>,
    mechanical_world: DefaultMechanicalWorld<f32>,

    bodies: DefaultBodySet<f32>,
    colliders: DefaultColliderSet<f32>,
    joint_constraints: DefaultJointConstraintSet<f32>,
    force_generators: DefaultForceGeneratorSet<f32>,

    circle_body_handles: Vec<DefaultBodyHandle>,
}

impl Physics {
    fn new() -> Self {
        let geometrical_world = DefaultGeometricalWorld::<f32>::new();
        let mechanical_world = DefaultMechanicalWorld::new(Vector2::y() * GRAVITY);

        let mut bodies = DefaultBodySet::<f32>::new();
        let mut colliders = DefaultColliderSet::new();
        let joint_constraints = DefaultJointConstraintSet::<f32>::new();
        let force_generators = DefaultForceGeneratorSet::<f32>::new();

        // A rectangle that the balls will fall on
        let ground_shape = ShapeHandle::new(Cuboid::new(Vector2::new(
            GROUND_HALF_EXTENTS_WIDTH,
            GROUND_THICKNESS,
        )));

        // Build a static ground body and add it to the body set.
        let ground_body_handle = bodies.insert(Ground::new());

        // Build the collider.
        let ground_collider = ColliderDesc::new(ground_shape)
            .translation(Vector2::y() * -GROUND_THICKNESS)
            .build(BodyPartHandle(ground_body_handle, 0));

        // Add the collider to the collider set.
        colliders.insert(ground_collider);

        let ball_shape_handle = ShapeHandle::new(Ball::new(BALL_RADIUS));

        let shift = (BALL_RADIUS + ColliderDesc::<f32>::default_margin()) * 2.0;
        let centerx = shift * (BALL_COUNT as f32) / 2.0;
        let centery = shift / 2.0;
        let height = 3.0;

        let mut circle_body_handles = vec![];

        for i in 0usize..BALL_COUNT {
            for j in 0usize..BALL_COUNT {
                let x = i as f32 * shift - centerx;
                let y = j as f32 * shift + centery + height;

                // Build the rigid body.
                let rigid_body = RigidBodyDesc::new().translation(Vector2::new(x, y)).build();

                // Insert the rigid body to the body set.
                let rigid_body_handle = bodies.insert(rigid_body);

                // Build the collider.
                let ball_collider = ColliderDesc::new(ball_shape_handle.clone())
                    .density(1.0)
                    .build(BodyPartHandle(rigid_body_handle, 0));

                // Insert the collider to the body set.
                colliders.insert(ball_collider);

                circle_body_handles.push(rigid_body_handle);
            }
        }

        Physics {
            geometrical_world,
            mechanical_world,
            bodies,
            colliders,
            joint_constraints,
            force_generators,
            circle_body_handles,
        }
    }

    fn step(&mut self) {
        // Run the simulation.
        self.mechanical_world.step(
            &mut self.geometrical_world,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joint_constraints,
            &mut self.force_generators,
        );
    }
}

fn main() {
    // Setup logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let example_app = ExampleApp::new();

    AppBuilder::new()
        .app_name(CString::new("Skulpin Example App").unwrap())
        .use_vulkan_debug_layer(true)
        .inner_size(LogicalSize::new(900, 600))
        .run(example_app);
}

struct ExampleApp {
    last_fps_text_change: Option<std::time::Instant>,
    fps_text: String,
    physics: Physics,
    circle_colors: Vec<skia_safe::Paint>,
}

impl ExampleApp {
    pub fn new() -> Self {
        fn create_circle_paint(color: skia_safe::Color4f) -> skia_safe::Paint {
            let mut paint = skia_safe::Paint::new(color, None);
            paint.set_anti_alias(true);
            paint.set_style(skia_safe::paint::Style::Stroke);
            paint.set_stroke_width(0.02);
            paint
        }

        let circle_colors = vec![
            create_circle_paint(skia_safe::Color4f::new(0.2, 1.0, 0.2, 1.0)),
            create_circle_paint(skia_safe::Color4f::new(1.0, 1.0, 0.2, 1.0)),
            create_circle_paint(skia_safe::Color4f::new(1.0, 0.2, 0.2, 1.0)),
            create_circle_paint(skia_safe::Color4f::new(0.2, 0.2, 1.0, 1.0)),
        ];

        ExampleApp {
            last_fps_text_change: None,
            fps_text: "".to_string(),
            physics: Physics::new(),
            circle_colors,
        }
    }
}

impl AppHandler for ExampleApp {
    fn update(
        &mut self,
        update_args: AppUpdateArgs,
    ) {
        let time_state = update_args.time_state;
        let input_state = update_args.input_state;
        let app_control = update_args.app_control;

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

        // Refresh FPS text
        if update_text_string {
            let fps = time_state.updates_per_second();
            self.fps_text = format!("Fps: {:.1}", fps);
            self.last_fps_text_change = Some(now);
        }

        // Update physics
        self.physics.step();
    }

    fn draw(
        &mut self,
        draw_args: AppDrawArgs,
    ) {
        let coordinate_system_helper = draw_args.coordinate_system_helper;
        let canvas = draw_args.canvas;

        let x_half_extents = GROUND_HALF_EXTENTS_WIDTH * 1.5;
        let y_half_extents = x_half_extents
            / (coordinate_system_helper.surface_extents().width as f32
                / coordinate_system_helper.surface_extents().height as f32);

        coordinate_system_helper
            .use_visible_range(
                canvas,
                skia_safe::Rect {
                    left: -x_half_extents,
                    right: x_half_extents,
                    top: y_half_extents + 1.0,
                    bottom: -y_half_extents + 1.0,
                },
                skia_safe::matrix::ScaleToFit::Center,
            )
            .unwrap();

        // Generally would want to clear data every time we draw
        canvas.clear(skia_safe::Color::from_argb(0, 0, 0, 255));

        // Make a color to draw with
        let mut paint = skia_safe::Paint::new(skia_safe::Color4f::new(0.0, 1.0, 0.0, 1.0), None);
        paint.set_anti_alias(true);
        paint.set_style(skia_safe::paint::Style::Stroke);
        paint.set_stroke_width(0.02);

        canvas.draw_rect(
            skia_safe::Rect {
                left: -GROUND_HALF_EXTENTS_WIDTH,
                top: 0.0,
                right: GROUND_HALF_EXTENTS_WIDTH,
                bottom: -GROUND_THICKNESS,
            },
            &paint,
        );

        let mut i = 0;
        for circle_body in &self.physics.circle_body_handles {
            let position = self
                .physics
                .bodies
                .rigid_body(*circle_body)
                .unwrap()
                .position()
                .translation;

            let paint = &self.circle_colors[i % self.circle_colors.len()];

            canvas.draw_circle(
                skia_safe::Point::new(position.x, position.y),
                BALL_RADIUS,
                paint,
            );

            i += 1;
        }

        coordinate_system_helper.use_logical_coordinates(canvas);

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
    }

    fn fatal_error(
        &mut self,
        error: &AppError,
    ) {
        println!("{}", error);
    }
}
