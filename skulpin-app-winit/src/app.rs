//! Contains the main types a user needs to interact with to configure and run a skulpin app

use crate::ash;
use crate::skia_safe;
use crate::winit;

use super::app_control::AppControl;
use super::input_state::InputState;
use super::time_state::TimeState;
use super::util::PeriodicEvent;
use std::ffi::CString;

use skulpin_renderer::LogicalSize;
use skulpin_renderer::Size;
use skulpin_renderer::RendererBuilder;
use skulpin_renderer::CreateRendererError;
use skulpin_renderer::CoordinateSystem;
use skulpin_renderer::CoordinateSystemHelper;
use skulpin_renderer::PresentMode;
use skulpin_renderer::PhysicalDeviceType;
use skulpin_renderer_winit::WinitWindow;

/// Represents an error from creating the renderer
#[derive(Debug)]
pub enum AppError {
    CreateRendererError(CreateRendererError),
    VkError(ash::vk::Result),
    WinitError(winit::error::OsError),
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            AppError::CreateRendererError(ref e) => Some(e),
            AppError::VkError(ref e) => Some(e),
            AppError::WinitError(ref e) => Some(e),
        }
    }
}

impl core::fmt::Display for AppError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        match *self {
            AppError::CreateRendererError(ref e) => e.fmt(fmt),
            AppError::VkError(ref e) => e.fmt(fmt),
            AppError::WinitError(ref e) => e.fmt(fmt),
        }
    }
}

impl From<CreateRendererError> for AppError {
    fn from(result: CreateRendererError) -> Self {
        AppError::CreateRendererError(result)
    }
}

impl From<ash::vk::Result> for AppError {
    fn from(result: ash::vk::Result) -> Self {
        AppError::VkError(result)
    }
}

impl From<winit::error::OsError> for AppError {
    fn from(result: winit::error::OsError) -> Self {
        AppError::WinitError(result)
    }
}

pub struct AppUpdateArgs<'a, 'b, 'c> {
    pub app_control: &'a mut AppControl,
    pub input_state: &'b InputState,
    pub time_state: &'c TimeState,
}

pub struct AppDrawArgs<'a, 'b, 'c, 'd> {
    pub app_control: &'a AppControl,
    pub input_state: &'b InputState,
    pub time_state: &'c TimeState,
    pub canvas: &'d mut skia_safe::Canvas,
    pub coordinate_system_helper: CoordinateSystemHelper,
}

/// A skulpin app requires implementing the AppHandler. A separate update and draw call must be
/// implemented.
///
/// `update` is called when winit provides a `winit::event::Event::MainEventsCleared` message
///
/// `draw` is called when winit provides a `winit::event::RedrawRequested` message
///
/// I would recommend putting general logic you always want to run in the `update` and just
/// rendering code in the `draw`.
pub trait AppHandler {
    /// Called frequently, this is the intended place to put non-rendering logic
    fn update(&mut self, update_args: AppUpdateArgs);

    /// Called frequently, this is the intended place to put drawing code
    fn draw(&mut self, draw_args: AppDrawArgs);

    fn fatal_error(&mut self, error: &AppError);
}

/// Used to configure the app behavior and create the app
pub struct AppBuilder {
    inner_size: Size,
    window_title: String,
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
            inner_size: LogicalSize::new(900, 600).into(),
            window_title: "Skulpin".to_string(),
            renderer_builder: RendererBuilder::new(),
        }
    }

    /// Specifies the inner size of the window. Both physical and logical coordinates are accepted.
    pub fn inner_size<S: Into<Size>>(mut self, inner_size: S) -> Self {
        self.inner_size = inner_size.into();
        self
    }

    /// Specifies the title that the window will be created with
    pub fn window_title<T: Into<String>>(mut self, window_title: T) -> Self {
        self.window_title = window_title.into();
        self
    }

    /// Name of the app. This is passed into the vulkan layer. I believe it can hint things to the
    /// vulkan driver, but it's unlikely this makes a real difference. Still a good idea to set this
    /// to something meaningful though.
    pub fn app_name(mut self, app_name: CString) -> Self {
        self.renderer_builder = self.renderer_builder.app_name(app_name);
        self
    }

    /// If true, initialize the vulkan debug layers. This will require the vulkan SDK to be
    /// installed and the app will fail to launch if it isn't. This turns on ALL logging. For
    /// more control, see `validation_layer_debug_report_flags()`
    pub fn use_vulkan_debug_layer(mut self, use_vulkan_debug_layer: bool) -> Self {
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

    /// Determine the coordinate system to use for the canvas. This can be overridden by using the
    /// canvas sizer passed into the draw callback
    pub fn coordinate_system(mut self, coordinate_system: CoordinateSystem) -> Self {
        self.renderer_builder = self.renderer_builder.coordinate_system(coordinate_system);
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
    pub fn present_mode_priority(mut self, present_mode_priority: Vec<PresentMode>) -> Self {
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
    /// This does not return because winit does not return. For consistency, we use the
    /// fatal_error() callback on the passed in AppHandler.
    pub fn run<T: 'static + AppHandler>(self, app_handler: T) -> ! {
        App::run(
            app_handler,
            self.inner_size,
            self.window_title.clone(),
            self.renderer_builder,
        )
    }
}

/// Constructed by `AppBuilder` which immediately calls `run`.
pub struct App {}

impl App {
    /// Runs the app. This is called by `AppBuilder::run`. This does not return because winit does
    /// not return. For consistency, we use the fatal_error() callback on the passed in AppHandler.
    pub fn run<T: 'static + AppHandler>(
        mut app_handler: T,
        inner_size: Size,
        window_title: String,
        renderer_builder: RendererBuilder,
    ) -> ! {
        // Create the event loop
        let event_loop = winit::event_loop::EventLoop::<()>::with_user_event();

        let winit_size = match inner_size {
            Size::Physical(physical_size) => winit::dpi::Size::Physical(
                winit::dpi::PhysicalSize::new(physical_size.width, physical_size.height),
            ),
            Size::Logical(logical_size) => winit::dpi::Size::Logical(winit::dpi::LogicalSize::new(
                logical_size.width as f64,
                logical_size.height as f64,
            )),
        };

        // Create a single window
        let window_result = winit::window::WindowBuilder::new()
            .with_title(window_title)
            .with_inner_size(winit_size)
            .build(&event_loop);

        let winit_window = match window_result {
            Ok(window) => window,
            Err(e) => {
                warn!("Passing WindowBuilder::build() error to app {}", e);

                let app_error = e.into();
                app_handler.fatal_error(&app_error);

                // Exiting in this way is consistent with how we will exit if we fail within the
                // input loop
                std::process::exit(0);
            }
        };

        let window = WinitWindow::new(&winit_window);

        let mut app_control = AppControl::default();
        let mut time_state = TimeState::new();
        let mut input_state = InputState::new(&winit_window);

        let renderer_result = renderer_builder.build(&window);
        let mut renderer = match renderer_result {
            Ok(renderer) => renderer,
            Err(e) => {
                warn!("Passing RendererBuilder::build() error to app {}", e);

                let app_error = e.into();
                app_handler.fatal_error(&app_error);

                // Exiting in this way is consistent with how we will exit if we fail within the
                // input loop
                std::process::exit(0);
            }
        };

        // To print fps once per second
        let mut print_fps_event = PeriodicEvent::default();

        // Pass control of this thread to winit until the app terminates. If this app wants to quit,
        // the update loop should send the appropriate event via the channel
        event_loop.run(move |event, window_target, control_flow| {
            let window = WinitWindow::new(&winit_window);
            input_state.handle_winit_event(&mut app_control, &event, window_target);

            match event {
                winit::event::Event::MainEventsCleared => {
                    time_state.update();

                    if print_fps_event.try_take_event(
                        time_state.current_instant(),
                        std::time::Duration::from_secs(1),
                    ) {
                        debug!("fps: {}", time_state.updates_per_second());
                    }

                    app_handler.update(AppUpdateArgs {
                        app_control: &mut app_control,
                        input_state: &input_state,
                        time_state: &time_state,
                    });

                    // Call this to mark the start of the next frame (i.e. "key just down" will return false)
                    input_state.end_frame();

                    // Queue a RedrawRequested event.
                    winit_window.request_redraw();
                }
                winit::event::Event::RedrawRequested(_window_id) => {
                    if let Err(e) = renderer.draw(&window, |canvas, coordinate_system_helper| {
                        app_handler.draw(AppDrawArgs {
                            app_control: &app_control,
                            input_state: &input_state,
                            time_state: &time_state,
                            canvas,
                            coordinate_system_helper,
                        });
                    }) {
                        warn!("Passing Renderer::draw() error to app {}", e);
                        app_handler.fatal_error(&e.into());
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
