//! Handles input tracking and provides an easy way to detect clicks, dragging, etc.

// Re-export winit types
pub use winit::event::VirtualKeyCode;
pub use winit::event::MouseButton;
pub use winit::event::ElementState;
pub use winit::dpi::LogicalSize;
pub use winit::dpi::PhysicalSize;
pub use winit::dpi::LogicalPosition;
pub use winit::dpi::PhysicalPosition;

use super::AppControl;
use winit::window::Window;

/// Encapsulates the state of a mouse drag
#[derive(Copy, Clone, Debug)]
pub struct MouseDragState {
    /// Logical position where the drag began
    pub begin_position: LogicalPosition,

    /// Logical position where the drag ended
    pub end_position: LogicalPosition,

    /// Amount of mouse movement in the previous frame
    pub previous_frame_delta: LogicalPosition,

    /// Amount of mouse movement in total
    pub accumulated_frame_delta: LogicalPosition,
}

/// State of input devices. This is maintained by processing events from winit
pub struct InputState {
    window_size: LogicalSize,
    dpi_factor: f64,

    key_is_down: [bool; Self::KEYBOARD_BUTTON_COUNT],
    key_just_down: [bool; Self::KEYBOARD_BUTTON_COUNT],
    key_just_up: [bool; Self::KEYBOARD_BUTTON_COUNT],

    mouse_position: LogicalPosition,
    mouse_button_is_down: [bool; Self::MOUSE_BUTTON_COUNT],
    mouse_button_just_down: [Option<LogicalPosition>; Self::MOUSE_BUTTON_COUNT],
    mouse_button_just_up: [Option<LogicalPosition>; Self::MOUSE_BUTTON_COUNT],

    mouse_button_just_clicked: [Option<LogicalPosition>; Self::MOUSE_BUTTON_COUNT],

    mouse_button_went_down_position: [Option<LogicalPosition>; Self::MOUSE_BUTTON_COUNT],
    mouse_button_went_up_position: [Option<LogicalPosition>; Self::MOUSE_BUTTON_COUNT],

    mouse_drag_in_progress: [Option<MouseDragState>; Self::MOUSE_BUTTON_COUNT],
    mouse_drag_just_finished: [Option<MouseDragState>; Self::MOUSE_BUTTON_COUNT],
}

impl InputState {
    /// Number of keyboard buttons we will track. Any button with a higher virtual key code will be
    /// ignored
    pub const KEYBOARD_BUTTON_COUNT: usize = 255;

    /// Number of mouse buttons we will track. Any button with a higher index will be ignored.
    pub const MOUSE_BUTTON_COUNT: usize = 7;

    /// Distance in LogicalPosition units that the mouse has to be dragged to be considered a drag
    /// rather than a click
    const MIN_DRAG_DISTANCE: f64 = 2.0;
}

impl InputState {
    /// Create a new input state to track the given window
    pub fn new(window: &Window) -> InputState {
        InputState {
            window_size: window.inner_size(),
            dpi_factor: window.hidpi_factor(),
            key_is_down: [false; Self::KEYBOARD_BUTTON_COUNT],
            key_just_down: [false; Self::KEYBOARD_BUTTON_COUNT],
            key_just_up: [false; Self::KEYBOARD_BUTTON_COUNT],
            mouse_position: LogicalPosition::new(0.0, 0.0),
            mouse_button_is_down: [false; Self::MOUSE_BUTTON_COUNT],
            mouse_button_just_down: [None; Self::MOUSE_BUTTON_COUNT],
            mouse_button_just_up: [None; Self::MOUSE_BUTTON_COUNT],
            mouse_button_just_clicked: [None; Self::MOUSE_BUTTON_COUNT],
            mouse_button_went_down_position: [None; Self::MOUSE_BUTTON_COUNT],
            mouse_button_went_up_position: [None; Self::MOUSE_BUTTON_COUNT],
            mouse_drag_in_progress: [None; Self::MOUSE_BUTTON_COUNT],
            mouse_drag_just_finished: [None; Self::MOUSE_BUTTON_COUNT],
        }
    }

    //
    // Accessors
    //

    /// Current size of window
    pub fn window_size(&self) -> LogicalSize {
        self.window_size
    }

    /// The scaling factor due to high-dpi screens
    pub fn dpi_factor(&self) -> f64 {
        self.dpi_factor
    }

    /// Returns true if the given key is down
    pub fn is_key_down(
        &self,
        key: VirtualKeyCode,
    ) -> bool {
        if let Some(index) = Self::keyboard_button_to_index(key) {
            self.key_is_down[index]
        } else {
            false
        }
    }

    /// Returns true if the key went down during this frame
    pub fn is_key_just_down(
        &self,
        key: VirtualKeyCode,
    ) -> bool {
        if let Some(index) = Self::keyboard_button_to_index(key) {
            self.key_just_down[index]
        } else {
            false
        }
    }

    /// Returns true if the key went up during this frame
    pub fn is_key_just_up(
        &self,
        key: VirtualKeyCode,
    ) -> bool {
        if let Some(index) = Self::keyboard_button_to_index(key) {
            self.key_just_up[index]
        } else {
            false
        }
    }

    /// Get the current mouse position
    pub fn mouse_position(&self) -> LogicalPosition {
        self.mouse_position
    }

    /// Returns true if the given button is down
    pub fn is_mouse_down(
        &self,
        mouse_button: MouseButton,
    ) -> bool {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            self.mouse_button_is_down[index]
        } else {
            false
        }
    }

    /// Returns true if the button went down during this frame
    pub fn is_mouse_just_down(
        &self,
        mouse_button: MouseButton,
    ) -> bool {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            self.mouse_button_just_down[index].is_some()
        } else {
            false
        }
    }

    /// Returns the position the mouse just went down at, otherwise returns None
    pub fn mouse_just_down_position(
        &self,
        mouse_button: MouseButton,
    ) -> Option<LogicalPosition> {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            self.mouse_button_just_down[index]
        } else {
            None
        }
    }

    /// Returns true if the button went up during this frame
    pub fn is_mouse_just_up(
        &self,
        mouse_button: MouseButton,
    ) -> bool {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            self.mouse_button_just_up[index].is_some()
        } else {
            false
        }
    }

    /// Returns the position the mouse just went up at, otherwise returns None
    pub fn mouse_just_up_position(
        &self,
        mouse_button: MouseButton,
    ) -> Option<LogicalPosition> {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            self.mouse_button_just_up[index]
        } else {
            None
        }
    }

    /// Returns true if the button was just clicked. "Clicked" means the button went down and came
    /// back up without being moved much. If it was moved, it would be considered a drag.
    pub fn is_mouse_button_just_clicked(
        &self,
        mouse_button: MouseButton,
    ) -> bool {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            self.mouse_button_just_clicked[index].is_some()
        } else {
            false
        }
    }

    /// Returns the position the button was just clicked at, otherwise None. "Clicked" means the
    /// button went down and came back up without being moved much. If it was moved, it would be
    /// considered a drag.
    pub fn mouse_button_just_clicked_position(
        &self,
        mouse_button: MouseButton,
    ) -> Option<LogicalPosition> {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            self.mouse_button_just_clicked[index]
        } else {
            None
        }
    }

    /// Returns the position the button went down at previously. This could have been some time ago.
    pub fn mouse_button_went_down_position(
        &self,
        mouse_button: MouseButton,
    ) -> Option<LogicalPosition> {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            self.mouse_button_went_down_position[index]
        } else {
            None
        }
    }

    /// Returns the position the button went up at previously. This could have been some time ago.
    pub fn mouse_button_went_up_position(
        &self,
        mouse_button: MouseButton,
    ) -> Option<LogicalPosition> {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            self.mouse_button_went_up_position[index]
        } else {
            None
        }
    }

    /// Return true if the mouse is being dragged. (A drag means the button went down and mouse
    /// moved, but button hasn't come back up yet)
    pub fn is_mouse_drag_in_progress(
        &self,
        mouse_button: MouseButton,
    ) -> bool {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            self.mouse_drag_in_progress[index].is_some()
        } else {
            false
        }
    }

    /// Returns the mouse drag state if a drag is in process, otherwise None.
    pub fn mouse_drag_in_progress(
        &self,
        mouse_button: MouseButton,
    ) -> Option<MouseDragState> {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            self.mouse_drag_in_progress[index]
        } else {
            None
        }
    }

    /// Return true if a mouse drag completed in the previous frame, otherwise false
    pub fn is_mouse_drag_just_finished(
        &self,
        mouse_button: MouseButton,
    ) -> bool {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            self.mouse_drag_just_finished[index].is_some()
        } else {
            false
        }
    }

    /// Returns information about a mouse drag if it just completed, otherwise None
    pub fn mouse_drag_just_finished(
        &self,
        mouse_button: MouseButton,
    ) -> Option<MouseDragState> {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            self.mouse_drag_just_finished[index]
        } else {
            None
        }
    }

    //
    // Handlers for significant events
    //

    /// Call at the end of every frame. This clears events that were "just" completed.
    pub fn end_frame(&mut self) {
        for value in self.key_just_down.iter_mut() {
            *value = false;
        }

        for value in self.key_just_up.iter_mut() {
            *value = false;
        }

        for value in self.mouse_button_just_down.iter_mut() {
            *value = None;
        }

        for value in self.mouse_button_just_up.iter_mut() {
            *value = None;
        }

        for value in self.mouse_button_just_clicked.iter_mut() {
            *value = None;
        }

        for value in self.mouse_drag_just_finished.iter_mut() {
            *value = None;
        }

        for value in self.mouse_drag_in_progress.iter_mut() {
            if let Some(v) = value {
                v.previous_frame_delta = LogicalPosition::new(0.0, 0.0);
            }
        }
    }

    /// Call when DPI factor changes
    fn handle_hidpi_factor_changed(
        &mut self,
        dpi_factor: f64,
    ) {
        self.dpi_factor = dpi_factor;
    }

    /// Call when window size changes
    fn handle_window_size_changed(
        &mut self,
        window_size: LogicalSize,
    ) {
        self.window_size = window_size;
    }

    /// Call when a key event occurs
    fn handle_keyboard_event(
        &mut self,
        keyboard_button: VirtualKeyCode,
        button_state: ElementState,
    ) {
        if let Some(kc) = Self::keyboard_button_to_index(keyboard_button) {
            // Assign true if key is down, or false if key is up
            if button_state == ElementState::Pressed {
                if !self.key_is_down[kc] {
                    self.key_just_down[kc] = true;
                }
                self.key_is_down[kc] = true
            } else {
                if self.key_is_down[kc] {
                    self.key_just_up[kc] = true;
                }
                self.key_is_down[kc] = false
            }
        }
    }

    /// Call when a mouse button event occurs
    fn handle_mouse_button_event(
        &mut self,
        button: MouseButton,
        button_event: ElementState,
    ) {
        if let Some(button_index) = Self::mouse_button_to_index(button) {
            assert!(button_index < InputState::MOUSE_BUTTON_COUNT);

            // Update is down/up, just down/up
            match button_event {
                ElementState::Pressed => {
                    self.mouse_button_just_down[button_index] = Some(self.mouse_position);
                    self.mouse_button_is_down[button_index] = true;

                    self.mouse_button_went_down_position[button_index] = Some(self.mouse_position);
                }
                ElementState::Released => {
                    self.mouse_button_just_up[button_index] = Some(self.mouse_position);
                    self.mouse_button_is_down[button_index] = false;

                    self.mouse_button_went_up_position[button_index] = Some(self.mouse_position);

                    match self.mouse_drag_in_progress[button_index] {
                        Some(in_progress) => {
                            let delta = Self::subtract(
                                self.mouse_position,
                                Self::add(
                                    in_progress.begin_position,
                                    in_progress.accumulated_frame_delta,
                                ),
                            );
                            self.mouse_drag_just_finished[button_index] = Some(MouseDragState {
                                begin_position: in_progress.begin_position,
                                end_position: self.mouse_position,
                                previous_frame_delta: delta,
                                accumulated_frame_delta: Self::add(
                                    in_progress.accumulated_frame_delta,
                                    delta,
                                ),
                            });
                        }
                        None => {
                            self.mouse_button_just_clicked[button_index] = Some(self.mouse_position)
                        }
                    }

                    self.mouse_drag_in_progress[button_index] = None;
                }
            }
        }
    }

    /// Call when a mouse move occurs
    fn handle_mouse_move_event(
        &mut self,
        position: LogicalPosition,
    ) {
        //let old_mouse_position = self.mouse_position;

        // Update mouse position
        self.mouse_position = position;

        // Update drag in progress state
        for i in 0..Self::MOUSE_BUTTON_COUNT {
            if self.mouse_button_is_down[i] {
                self.mouse_drag_in_progress[i] = match self.mouse_drag_in_progress[i] {
                    None => {
                        match self.mouse_button_went_down_position[i] {
                            Some(went_down_position) => {
                                let min_drag_distance_met =
                                    Self::distance(went_down_position, self.mouse_position)
                                        > Self::MIN_DRAG_DISTANCE;
                                if min_drag_distance_met {
                                    // We dragged a non-trivial amount, start the drag
                                    Some(MouseDragState {
                                        begin_position: went_down_position,
                                        end_position: self.mouse_position,
                                        previous_frame_delta: Self::subtract(
                                            self.mouse_position,
                                            went_down_position,
                                        ),
                                        accumulated_frame_delta: Self::subtract(
                                            self.mouse_position,
                                            went_down_position,
                                        ),
                                    })
                                } else {
                                    // Mouse moved too small an amount to be considered a drag
                                    None
                                }
                            }

                            // We don't know where the mosue went down, so we can't start a drag
                            None => None,
                        }
                    }
                    Some(old_drag_state) => {
                        // We were already dragging, so just update the end position

                        let delta = Self::subtract(
                            self.mouse_position,
                            Self::add(
                                old_drag_state.begin_position,
                                old_drag_state.accumulated_frame_delta,
                            ),
                        );
                        Some(MouseDragState {
                            begin_position: old_drag_state.begin_position,
                            end_position: self.mouse_position,
                            previous_frame_delta: delta,
                            accumulated_frame_delta: Self::add(
                                old_drag_state.accumulated_frame_delta,
                                delta,
                            ),
                        })
                    }
                };
            }
        }
    }

    /// Call when winit sends an event
    pub fn handle_winit_event<T>(
        &mut self,
        app_control: &mut AppControl,
        event: &winit::event::Event<T>,
        _window_target: &winit::event_loop::EventLoopWindowTarget<T>,
    ) {
        use winit::event::Event;
        use winit::event::WindowEvent;

        let mut is_close_requested = false;

        match event {
            // Close if the window is killed
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => is_close_requested = true,

            Event::WindowEvent {
                event: WindowEvent::HiDpiFactorChanged(hidpi_factor),
                ..
            } => {
                trace!("dpi scaling factor changed {:?}", hidpi_factor);
                self.handle_hidpi_factor_changed(*hidpi_factor);
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(window_size),
                ..
            } => self.handle_window_size_changed(*window_size),

            //Process keyboard input
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                trace!("keyboard input {:?}", input);
                if let Some(vk) = input.virtual_keycode {
                    self.handle_keyboard_event(vk, input.state);
                }
            }

            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        device_id,
                        state,
                        button,
                        ..
                    },
                ..
            } => {
                trace!(
                    "mouse button input {:?} {:?} {:?}",
                    device_id,
                    state,
                    button,
                );

                self.handle_mouse_button_event(*button, *state);
            }

            Event::WindowEvent {
                event:
                    WindowEvent::CursorMoved {
                        device_id,
                        position,
                        ..
                    },
                ..
            } => {
                trace!("mouse move input {:?} {:?}", device_id, position,);
                self.handle_mouse_move_event(*position);
            }

            // Ignore any other events
            _ => (),
        }

        if is_close_requested {
            trace!("close requested");
            app_control.enqueue_terminate_process();
        }
    }

    //
    // Helper functions
    //

    /// Convert the winit mouse button enum into a numerical index
    pub fn mouse_button_to_index(button: MouseButton) -> Option<usize> {
        let index = match button {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            MouseButton::Other(x) => (x as usize) + 3,
        };

        if index >= Self::MOUSE_BUTTON_COUNT {
            None
        } else {
            Some(index)
        }
    }

    /// Convert to the winit mouse button enum from a numerical index
    pub fn mouse_index_to_button(index: usize) -> Option<MouseButton> {
        if index >= Self::MOUSE_BUTTON_COUNT {
            None
        } else {
            let button = match index {
                0 => MouseButton::Left,
                1 => MouseButton::Right,
                2 => MouseButton::Middle,
                _ => MouseButton::Other((index - 3) as u8),
            };

            Some(button)
        }
    }

    /// Convert the winit virtual key code into a numerical index
    pub fn keyboard_button_to_index(button: VirtualKeyCode) -> Option<usize> {
        let index = button as usize;
        if index >= Self::KEYBOARD_BUTTON_COUNT {
            None
        } else {
            Some(index)
        }
    }

    /// Adds two logical positions (p0 + p1)
    fn add(
        p0: LogicalPosition,
        p1: LogicalPosition,
    ) -> LogicalPosition {
        LogicalPosition::new(p0.x + p1.x, p0.y + p1.y)
    }

    /// Subtracts two logical positions (p0 - p1)
    fn subtract(
        p0: LogicalPosition,
        p1: LogicalPosition,
    ) -> LogicalPosition {
        LogicalPosition::new(p0.x - p1.x, p0.y - p1.y)
    }

    /// Gets the distance between two logical positions
    fn distance(
        p0: LogicalPosition,
        p1: LogicalPosition,
    ) -> f64 {
        let x_diff = p1.x - p0.x;
        let y_diff = p1.y - p0.y;

        ((x_diff * x_diff) + (y_diff * y_diff)).sqrt()
    }
}
