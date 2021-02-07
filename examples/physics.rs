// This example does a physics demo, because physics is fun :)

use skulpin::skia_safe;

use skulpin::app::AppHandler;
use skulpin::app::AppUpdateArgs;
use skulpin::app::AppDrawArgs;
use skulpin::app::AppError;
use skulpin::app::AppBuilder;
use skulpin::app::VirtualKeyCode;

use skulpin::LogicalSize;

// Used for physics
type Vector2 = rapier2d::na::Vector2<f32>;
use rapier2d::dynamics::{
    JointSet, RigidBodySet, IntegrationParameters, RigidBodyBuilder, RigidBodyHandle,
};
use rapier2d::geometry::{BroadPhase, NarrowPhase, ColliderSet, ColliderBuilder};
use rapier2d::pipeline::PhysicsPipeline;

// use ncollide2d::shape::{Cuboid, ShapeHandle, Ball};
// use nphysics2d::object::{
//     ColliderDesc, RigidBodyDesc, DefaultBodySet, DefaultColliderSet, Ground, BodyPartHandle,
//     DefaultBodyHandle,
// };
// use nphysics2d::force_generator::DefaultForceGeneratorSet;
// use nphysics2d::joint::DefaultJointConstraintSet;
// use nphysics2d::world::{DefaultMechanicalWorld, DefaultGeometricalWorld};

const GROUND_THICKNESS: f32 = 0.2;
const GROUND_HALF_EXTENTS_WIDTH: f32 = 3.0;
const BALL_RADIUS: f32 = 0.2;
const GRAVITY: f32 = -9.81;
const BALL_COUNT: usize = 5;

// Will contain all the physics simulation state
struct Physics {
    // geometrical_world: DefaultGeometricalWorld<f32>,
    // mechanical_world: DefaultMechanicalWorld<f32>,
    //
    // bodies: DefaultBodySet<f32>,
    // colliders: DefaultColliderSet<f32>,
    // joint_constraints: DefaultJointConstraintSet<f32>,
    // force_generators: DefaultForceGeneratorSet<f32>,
    //
    circle_body_handles: Vec<RigidBodyHandle>,

    physics_pipeline: PhysicsPipeline,
    gravity: Vector2,
    integration_parameters: IntegrationParameters,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    joint_set: JointSet,

    last_update: std::time::Instant,
    accumulated_time: f64,
}

impl Physics {
    fn new() -> Self {
        //
        // Basic physics system setup
        //
        let physics_pipeline = PhysicsPipeline::new();
        let gravity = Vector2::new(0.0, GRAVITY);
        let integration_parameters = IntegrationParameters::default();
        let broad_phase = BroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();
        let joint_set = JointSet::new();

        //
        // Create the "ground"
        //
        let ground_body = RigidBodyBuilder::new_static()
            .translation(0.0, -GROUND_THICKNESS)
            .build();

        let ground_body_handle = rigid_body_set.insert(ground_body);

        let ground_collider =
            ColliderBuilder::cuboid(GROUND_HALF_EXTENTS_WIDTH, GROUND_THICKNESS).build();
        collider_set.insert(ground_collider, ground_body_handle, &mut rigid_body_set);

        //
        // Create falling objects
        //
        let shift = (BALL_RADIUS + 0.01) * 2.0;
        let centerx_base = shift * (BALL_COUNT as f32) / 2.0;
        let centery = shift / 2.0;
        let height = 3.0;

        let mut circle_body_handles = vec![];

        for i in 0usize..BALL_COUNT {
            for j in 0usize..BALL_COUNT {
                // Vary the x so the balls don't stack
                let centerx = if j % 2 == 0 {
                    centerx_base + 0.1
                } else {
                    centerx_base - 0.1
                };

                let x = i as f32 * shift - centerx;
                let y = j as f32 * shift + centery + height;

                let rigid_body = RigidBodyBuilder::new_dynamic().translation(x, y).build();

                let rigid_body_handle = rigid_body_set.insert(rigid_body);

                let ball_collider = ColliderBuilder::ball(BALL_RADIUS).density(1.0).build();

                // Insert the collider to the body set.
                collider_set.insert(ball_collider, rigid_body_handle, &mut rigid_body_set);

                circle_body_handles.push(rigid_body_handle);
            }
        }

        let last_update = std::time::Instant::now();

        Physics {
            physics_pipeline,
            gravity,
            integration_parameters,
            broad_phase,
            narrow_phase,
            rigid_body_set,
            collider_set,
            joint_set,
            circle_body_handles,
            last_update,
            accumulated_time: 0.0,
        }
    }

    fn update(&mut self) {
        let now = std::time::Instant::now();
        let time_since_last_update = (now - self.last_update).as_secs_f64();
        self.accumulated_time += time_since_last_update;
        self.last_update = now;

        const STEP_TIME: f64 = 1.0 / 60.0;
        while self.accumulated_time > STEP_TIME {
            self.accumulated_time -= STEP_TIME;

            // Run the simulation.
            self.physics_pipeline.step(
                &self.gravity,
                &self.integration_parameters,
                &mut self.broad_phase,
                &mut self.narrow_phase,
                &mut self.rigid_body_set,
                &mut self.collider_set,
                &mut self.joint_set,
                None,
                None,
                &(),
            );
        }
    }
}

fn main() {
    // Setup logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let example_app = ExampleApp::new();

    AppBuilder::new()
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
        self.physics.update();
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
        canvas.clear(skia_safe::Color::from_argb(255, 0, 0, 0));

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

        for (i, circle_body) in self.physics.circle_body_handles.iter().enumerate() {
            let position = self
                .physics
                .rigid_body_set
                .get(*circle_body)
                .unwrap()
                .position()
                .translation;

            let paint = &self.circle_colors[i % self.circle_colors.len()];

            canvas.draw_circle(
                skia_safe::Point::new(position.x, position.y),
                BALL_RADIUS,
                paint,
            );
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
