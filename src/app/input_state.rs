
pub use winit::event::VirtualKeyCode;
pub use winit::event::ElementState;
pub use winit::event::MouseButton;

impl InputState {
    pub const KEYBOARD_BUTTON_COUNT: usize = 255;
    pub const MOUSE_BUTTON_COUNT: usize = 7;
    const MIN_DRAG_DISTANCE : f32 = 2.0;
}

#[derive(Copy, Clone, Debug)]
pub struct MouseDragState {
    pub begin_position: glam::Vec2,
    pub end_position: glam::Vec2,
    pub previous_frame_delta: glam::Vec2,
    pub accumulated_frame_delta: glam::Vec2
}

pub struct InputState {
    key_is_down: [bool; Self::KEYBOARD_BUTTON_COUNT],
    key_just_down: [bool; Self::KEYBOARD_BUTTON_COUNT],
    key_just_up: [bool; Self::KEYBOARD_BUTTON_COUNT],

    mouse_position: glam::Vec2,
    mouse_button_is_down: [bool; Self::MOUSE_BUTTON_COUNT],
    mouse_button_just_down: [Option<glam::Vec2>; Self::MOUSE_BUTTON_COUNT],
    mouse_button_just_up: [Option<glam::Vec2>; Self::MOUSE_BUTTON_COUNT],

    mouse_button_just_clicked: [Option<glam::Vec2>; Self::MOUSE_BUTTON_COUNT],

    mouse_button_went_down_position: [Option<glam::Vec2>; Self::MOUSE_BUTTON_COUNT],
    mouse_button_went_up_position: [Option<glam::Vec2>; Self::MOUSE_BUTTON_COUNT],

    mouse_drag_in_progress: [Option<MouseDragState>; Self::MOUSE_BUTTON_COUNT],
    mouse_drag_just_finished: [Option<MouseDragState>; Self::MOUSE_BUTTON_COUNT],
}

impl Default for InputState {
    fn default() -> InputState {
        return InputState {
            key_is_down: [false; Self::KEYBOARD_BUTTON_COUNT],
            key_just_down: [false; Self::KEYBOARD_BUTTON_COUNT],
            key_just_up: [false; Self::KEYBOARD_BUTTON_COUNT],
            mouse_position: glam::Vec2::zero(),
            mouse_button_is_down: [false; Self::MOUSE_BUTTON_COUNT],
            mouse_button_just_down: [None; Self::MOUSE_BUTTON_COUNT],
            mouse_button_just_up: [None; Self::MOUSE_BUTTON_COUNT],
            mouse_button_just_clicked: [None; Self::MOUSE_BUTTON_COUNT],
            mouse_button_went_down_position: [None; Self::MOUSE_BUTTON_COUNT],
            mouse_button_went_up_position: [None; Self::MOUSE_BUTTON_COUNT],
            mouse_drag_in_progress: [None; Self::MOUSE_BUTTON_COUNT],
            mouse_drag_just_finished: [None; Self::MOUSE_BUTTON_COUNT],
        };
    }
}

impl InputState {

    fn mouse_button_to_index(button: MouseButton) -> Option<usize> {
        let index = match button {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            MouseButton::Other(x) => (x as usize) + 3
        };

        if index >= Self::MOUSE_BUTTON_COUNT {
            None
        } else {
            Some(index)
        }
    }

    fn keyboard_button_to_index(button: VirtualKeyCode) -> Option<usize> {
        let index = button as usize;
        if index >= Self::KEYBOARD_BUTTON_COUNT {
            None
        } else {
            Some(index)
        }
    }

    pub fn is_key_down(&self, key: VirtualKeyCode) -> bool {
        if let Some(index) = Self::keyboard_button_to_index(key) {
            return self.key_is_down[index];
        } else {
            false
        }
    }

    pub fn is_key_just_down(&self, key: VirtualKeyCode) -> bool {
        if let Some(index) = Self::keyboard_button_to_index(key) {
            return self.key_just_down[index];
        } else {
            false
        }
    }

    pub fn is_key_just_up(&self, key: VirtualKeyCode) -> bool {
        if let Some(index) = Self::keyboard_button_to_index(key) {
            return self.key_just_up[index];
        } else {
            false
        }
    }

    pub fn mouse_position(&self) -> glam::Vec2 {
        return self.mouse_position;
    }

    pub fn is_mouse_down(&self, mouse_button: MouseButton) -> bool {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            return self.mouse_button_is_down[index];
        } else {
            false
        }
    }

    pub fn is_mouse_just_down(&self, mouse_button: MouseButton) -> bool {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            return self.mouse_button_just_down[index].is_some();
        } else {
            false
        }
    }

    pub fn mouse_just_down_position(&self, mouse_button: MouseButton) -> Option<glam::Vec2> {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            return self.mouse_button_just_down[index];
        } else {
            None
        }
    }

    pub fn is_mouse_just_up(&self, mouse_button: MouseButton) -> bool {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            return self.mouse_button_just_up[index].is_some();
        } else {
            false
        }
    }

    pub fn mouse_just_up_position(&self, mouse_button: MouseButton) -> Option<glam::Vec2> {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            return self.mouse_button_just_up[index];
        } else {
            None
        }
    }

    pub fn is_mouse_button_just_clicked(&self, mouse_button: MouseButton) -> bool {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            return self.mouse_button_just_clicked[index].is_some();
        } else {
            false
        }
    }

    pub fn mouse_button_just_clicked_position(
        &self,
        mouse_button: MouseButton,
    ) -> Option<glam::Vec2> {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            return self.mouse_button_just_clicked[index];
        } else {
            None
        }
    }

    pub fn mouse_button_went_down_position(&self, mouse_button: MouseButton) -> Option<glam::Vec2> {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            return self.mouse_button_went_down_position[index];
        } else {
            None
        }
    }

    pub fn mouse_button_went_up_position(&self, mouse_button: MouseButton) -> Option<glam::Vec2> {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            return self.mouse_button_went_up_position[index];
        } else {
            None
        }
    }

    pub fn is_mouse_drag_in_progress(&self, mouse_button: MouseButton) -> bool {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            return self.mouse_drag_in_progress[index].is_some();
        } else {
            false
        }
    }

    pub fn mouse_drag_in_progress(&self, mouse_button: MouseButton) -> Option<MouseDragState> {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            return self.mouse_drag_in_progress[index];
        } else {
            None
        }
    }

    pub fn is_mouse_drag_just_finished(&self, mouse_button: MouseButton) -> bool {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            return self.mouse_drag_just_finished[index].is_some();
        } else {
            false
        }
    }

    pub fn mouse_drag_just_finished(&self, mouse_button: MouseButton) -> Option<MouseDragState> {
        if let Some(index) = Self::mouse_button_to_index(mouse_button) {
            return self.mouse_drag_just_finished[index];
        } else {
            None
        }
    }

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
                v.previous_frame_delta = glam::Vec2::zero();
            }
        }
    }

    pub fn handle_keyboard_event(
        &mut self,
        keyboard_button: VirtualKeyCode,
        button_state: ElementState
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

    pub fn handle_mouse_button_event(
        &mut self,
        button: MouseButton,
        button_event: ElementState
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

                            let delta = self.mouse_position - (in_progress.begin_position + in_progress.accumulated_frame_delta);
                            self.mouse_drag_just_finished[button_index] = Some(MouseDragState {
                                begin_position: in_progress.begin_position,
                                end_position: self.mouse_position,
                                previous_frame_delta: delta,
                                accumulated_frame_delta: in_progress.accumulated_frame_delta + delta
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

    pub fn handle_mouse_move_event(&mut self, position: glam::Vec2) {
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
                                    glam::Vec2::length(went_down_position - self.mouse_position)
                                        > Self::MIN_DRAG_DISTANCE;
                                if min_drag_distance_met {
                                    // We dragged a non-trivial amount, start the drag
                                    Some(MouseDragState {
                                        begin_position: went_down_position,
                                        end_position: self.mouse_position,
                                        previous_frame_delta: self.mouse_position - went_down_position,
                                        accumulated_frame_delta: self.mouse_position - went_down_position
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

                        let delta = self.mouse_position - (old_drag_state.begin_position + old_drag_state.accumulated_frame_delta);
                        Some(MouseDragState {
                            begin_position: old_drag_state.begin_position,
                            end_position: self.mouse_position,
                            previous_frame_delta: delta,
                            accumulated_frame_delta: old_drag_state.accumulated_frame_delta + delta
                        })
                    }
                };
            }
        }
    }
}
