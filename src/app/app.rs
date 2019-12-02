//! Contains the main types a user needs to interact with to configure and run a skulpin app

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

/// A skulpin app requires implementing the AppHandler. A separate update and draw call must be
/// implemented.
///
/// `update` is called when winit provides a `winit::event::Event::EventsCleared` message
///
/// `draw` is called when winit provides a `winit::event::WindowEvent::RedrawRequested` message
///
/// I would recommend putting general logic you always want to run in the `update` and just
/// rendering code in the `draw`.
pub trait AppHandler {
    /// Called frequently, this is the intended place to put non-rendering logic
    fn update(
        &mut self,
        app_control: &mut AppControl,
        input_state: &InputState,
        time_state: &TimeState,
    );

    /// Called frequently, this is the intended place to put drawing code
    fn draw(
        &mut self,
        app_control: &AppControl,
        input_state: &InputState,
        time_state: &TimeState,
        canvas: &mut skia_safe::Canvas,
    );
}

/// Used to configure the app behavior and create the app
pub struct AppBuilder {
    logical_size: LogicalSize,
    renderer_builder: RendererBuilder,
}

impl Default for AppBuilder {
    fn default() -> Self {
        AppBuilder::new()
    }
}

impl AppBuilder {
    /// Construct the app builder initialized with default options
    pub fn new() -> Self {
        AppBuilder {
            logical_size: LogicalSize::new(900.0, 600.0),
            renderer_builder: RendererBuilder::new(),
        }
    }

    /// Specifies the logical size of the window. The physical size of the window will depend on
    /// dpi settings. For example, a 500x300 window on a 2x dpi screen could be 1000x600 in
    /// physical pixel size
    pub fn logical_size(
        mut self,
        logical_size: LogicalSize,
    ) -> Self {
        self.logical_size = logical_size;
        self
    }

    /// Name of the app. This is passed into the vulkan layer. I believe it can hint things to the
    /// vulkan driver, but it's unlikely this makes a real difference. Still a good idea to set this
    /// to something meaningful though.
    pub fn app_name(
        mut self,
        app_name: CString,
    ) -> Self {
        self.renderer_builder = self.renderer_builder.app_name(app_name);
        self
    }

    /// If true, initialize the vulkan debug layers. This will require the vulkan SDK to be
    /// installed and the app will fail to launch if it isn't. This turns on ALL logging. For
    /// more control, see `validation_layer_debug_report_flags()`
    pub fn use_vulkan_debug_layer(
        mut self,
        use_vulkan_debug_layer: bool,
    ) -> Self {
        self.renderer_builder = self
            .renderer_builder
            .use_vulkan_debug_layer(use_vulkan_debug_layer);
        self
    }

    /// Sets the desired debug layer flags. If any flag is set, the vulkan debug layers will be
    /// loaded, which requires the Vulkan SDK to be installed. The app will fail to launch if it
    /// isn't.
    pub fn validation_layer_debug_report_flags(
        mut self,
        validation_layer_debug_report_flags: ash::vk::DebugReportFlagsEXT,
    ) -> Self {
        self.renderer_builder = self
            .renderer_builder
            .validation_layer_debug_report_flags(validation_layer_debug_report_flags);
        self
    }

    /// Specify which PresentMode is preferred. Some of this is hardware/platform dependent and
    /// it's a good idea to read the Vulkan spec. You
    ///
    /// `present_mode_priority` should be a list of desired present modes, in descending order of
    /// preference. In other words, passing `[Mailbox, Fifo]` will direct Skulpin to use mailbox
    /// where available, but otherwise use `Fifo`.
    ///
    /// Since `Fifo` is always available, this is the mode that will be chosen if no desired mode is
    /// available.
    pub fn present_mode_priority(
        mut self,
        present_mode_priority: Vec<PresentMode>,
    ) -> Self {
        self.renderer_builder = self
            .renderer_builder
            .present_mode_priority(present_mode_priority);
        self
    }

    /// Specify which type of physical device is preferred. It's recommended to read the Vulkan spec
    /// to understand precisely what these types mean
    ///
    /// `physical_device_type_priority` should be a list of desired present modes, in descending
    /// order of preference. In other words, passing `[Discrete, Integrated]` will direct Skulpin to
    /// use the discrete GPU where available, otherwise integrated.
    ///
    /// If the desired device type can't be found, Skulpin will try to use whatever device is
    /// available. By default `Discrete` is favored, then `Integrated`, then anything that's
    /// available. It could make sense to favor `Integrated` over `Discrete` when minimizing
    /// power consumption is important. (Although I haven't tested this myself)
    pub fn physical_device_type_priority(
        mut self,
        physical_device_type_priority: Vec<PhysicalDeviceType>,
    ) -> Self {
        self.renderer_builder = self
            .renderer_builder
            .physical_device_type_priority(physical_device_type_priority);
        self
    }

    /// Easy shortcut to set device type priority to `Integrated`, then `Discrete`, then any.
    pub fn prefer_integrated_gpu(mut self) -> Self {
        self.renderer_builder = self.renderer_builder.prefer_integrated_gpu();
        self
    }

    /// Easy shortcut to set device type priority to `Discrete`, then `Integrated`, than any.
    /// (This is the default behavior)
    pub fn prefer_discrete_gpu(mut self) -> Self {
        self.renderer_builder = self.renderer_builder.prefer_discrete_gpu();
        self
    }

    /// Prefer using `Fifo` presentation mode. This presentation mode is always available on a
    /// device that complies with the vulkan spec.
    pub fn prefer_fifo_present_mode(mut self) -> Self {
        self.renderer_builder = self.renderer_builder.prefer_fifo_present_mode();
        self
    }

    /// Prefer using `Mailbox` presentation mode, and fall back to `Fifo` when not available.
    pub fn prefer_mailbox_present_mode(mut self) -> Self {
        self.renderer_builder = self.renderer_builder.prefer_mailbox_present_mode();
        self
    }

    /// Start the app. `app_handler` must be an implementation of [skulpin::app::AppHandler].
    pub fn run<T: 'static + AppHandler>(
        &self,
        app_handler: T,
    ) -> Result<(), Box<dyn std::error::Error>> {
        App::run(app_handler, self.logical_size, &self.renderer_builder)
    }
}

/// Constructed by `AppBuilder` which immediately calls `run`.
pub struct App {}

impl App {
    /// Runs the app. This is called by `AppBuilder::run`
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
        let mut time_state = TimeState::new();
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
