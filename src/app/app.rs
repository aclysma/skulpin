use super::app_control::AppControl;
use super::input_state::InputState;
use super::time_state::TimeState;
use super::time_state::TimeContext;
use super::util::PeriodicEvent;
use std::ffi::CString;

use crate::RendererBuilder;
use crate::renderer::PresentMode;
use crate::renderer::PhysicalDeviceType;
use winit::dpi::LogicalSize;

pub trait AppHandler {
    fn update(
        &mut self,
        app_control: &mut AppControl,
        input_state: &InputState,
        time_state: &TimeState,
    );

    fn draw(
        &mut self,
        app_control: &AppControl,
        input_state: &InputState,
        time_state: &TimeState,
        canvas: &mut skia_safe::Canvas,
    );
}

pub struct AppBuilder {
    logical_size: LogicalSize,
    renderer_builder: RendererBuilder,
}

impl AppBuilder {
    pub fn new() -> Self {
        AppBuilder {
            logical_size: LogicalSize::new(900.0, 600.0),
            renderer_builder: RendererBuilder::new(),
        }
    }

    pub fn logical_size(
        mut self,
        logical_size: LogicalSize,
    ) -> Self {
        self.logical_size = logical_size;
        self
    }

    pub fn app_name(
        mut self,
        app_name: CString,
    ) -> Self {
        self.renderer_builder = self.renderer_builder.app_name(app_name);
        self
    }

    pub fn use_vulkan_debug_layer(
        mut self,
        use_vulkan_debug_layer: bool,
    ) -> Self {
        self.renderer_builder = self
            .renderer_builder
            .use_vulkan_debug_layer(use_vulkan_debug_layer);
        self
    }

    pub fn present_mode_priority(
        mut self,
        present_mode_priority: Vec<PresentMode>,
    ) -> Self {
        self.renderer_builder = self
            .renderer_builder
            .present_mode_priority(present_mode_priority);
        self
    }

    pub fn physical_device_type_priority(
        mut self,
        physical_device_type_priority: Vec<PhysicalDeviceType>,
    ) -> Self {
        self.renderer_builder = self
            .renderer_builder
            .physical_device_type_priority(physical_device_type_priority);
        self
    }

    pub fn prefer_integrated_gpu(mut self) -> Self {
        self.renderer_builder = self.renderer_builder.prefer_integrated_gpu();
        self
    }

    pub fn prefer_discrete_gpu(mut self) -> Self {
        self.renderer_builder = self.renderer_builder.prefer_discrete_gpu();
        self
    }

    pub fn prefer_fifo_present_mode(mut self) -> Self {
        self.renderer_builder = self.renderer_builder.prefer_fifo_present_mode();
        self
    }

    pub fn prefer_mailbox_present_mode(mut self) -> Self {
        self.renderer_builder = self.renderer_builder.prefer_mailbox_present_mode();
        self
    }

    pub fn run<T: 'static + AppHandler>(
        &self,
        app_handler: T,
    ) -> Result<(), Box<dyn std::error::Error>> {
        App::run(app_handler, self.logical_size, &self.renderer_builder)
    }
}

pub struct App {}

impl App {
    //TODO: Since winit returns !, we should just take a callback here for handling errors instead
    // of returning
    pub fn run<T: 'static + AppHandler>(
        mut app_handler: T,
        logical_size: LogicalSize,
        renderer_builder: &RendererBuilder,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create the event loop
        let event_loop = winit::event_loop::EventLoop::<()>::with_user_event();

        // Create a single window
        let window = winit::window::WindowBuilder::new()
            .with_title("Skulpin")
            .with_inner_size(logical_size)
            .build(&event_loop)?;

        let mut app_control = AppControl::default();
        let mut time_state = TimeState::default();
        let mut input_state = InputState::new(&window);

        let mut renderer = renderer_builder.build(&window)?;

        // To print fps once per second
        let mut print_fps_event = PeriodicEvent::default();

        // Pass control of this thread to winit until the app terminates. If this app wants to quit,
        // the update loop should send the appropriate event via the channel
        event_loop.run(move |event, window_target, control_flow| {
            input_state.handle_winit_event(&mut app_control, &event, window_target);

            match event {
                winit::event::Event::EventsCleared => {
                    time_state.update(TimeContext::System);

                    if print_fps_event.try_take_event(
                        time_state.system().frame_start_instant,
                        std::time::Duration::from_secs(1),
                    ) {
                        debug!("fps: {}", time_state.system().fps);
                    }

                    app_handler.update(&mut app_control, &input_state, &time_state);

                    // Call this to mark the start of the next frame (i.e. "key just down" will return false)
                    input_state.end_frame();

                    // Queue a RedrawRequested event.
                    window.request_redraw();
                }
                winit::event::Event::WindowEvent {
                    event: winit::event::WindowEvent::RedrawRequested,
                    ..
                } => {
                    if let Err(e) = renderer.draw(&window, |canvas| {
                        app_handler.draw(&app_control, &input_state, &time_state, canvas);
                    }) {
                        //TODO: Handle Error
                        warn!("{:?}", e);
                        app_control.enqueue_terminate_process();
                    }
                }
                _ => {}
            }

            if app_control.should_terminate_process() {
                *control_flow = winit::event_loop::ControlFlow::Exit
            }
        });
    }
}
