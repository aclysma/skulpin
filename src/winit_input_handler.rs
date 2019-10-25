
pub struct WinitInputHandler {
    hidpi_factor: f64
}

impl WinitInputHandler {

    pub fn new() -> Self {
        WinitInputHandler {
            hidpi_factor: 1.0
        }
    }

    pub fn handle_input<T>(
        &mut self,
        app_control: &mut crate::AppControl,
        input_state: &mut crate::input_state::InputState,
        event: winit::event::Event<T>,
        _window_target: &winit::event_loop::EventLoopWindowTarget<T>
    ) {
        use winit::event::Event;
        use winit::event::WindowEvent;
        use crate::input_state;

        let mut is_close_requested = false;

        match event {
            // Close if the window is killed
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => is_close_requested = true,

            // Close if the escape key is hit
            Event::WindowEvent {
                event:
                WindowEvent::KeyboardInput {
                    input:
                    winit::event::KeyboardInput {
                        virtual_keycode:
                        Some(winit::event::VirtualKeyCode::Escape),
                        ..
                    },
                    ..
                },
                ..
            } => is_close_requested = true,

            Event::WindowEvent {
                event: WindowEvent::HiDpiFactorChanged(hidpi_factor),
                ..
            } => {
                info!("dpi scaling factor changed {:?}", hidpi_factor);
                self.hidpi_factor = hidpi_factor;
                //TODO: fix old mouse positions? Could store as logical and only convert to physical
                // on demand
            }

            //Process keyboard input
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                info!("keyboard {:?}", input);
                if let Some(vk) = input.virtual_keycode {
                    let keyboard_button = input_state::KeyboardButton::new(vk as u32);

                    let keyboard_event = match input.state {
                        winit::event::ElementState::Pressed => input_state::KeyboardButtonEvent::Pressed,
                        winit::event::ElementState::Released => input_state::KeyboardButtonEvent::Released,
                    };

                    input_state.handle_keyboard_event(keyboard_button, keyboard_event);
                }
            }

            Event::WindowEvent {
                event:
                WindowEvent::MouseInput {
                    device_id,
                    state,
                    button,
                    modifiers,
                },
                ..
            } => {
                trace!(
                    "mouse {:?} {:?} {:?} {:?}",
                    device_id,
                    state,
                    button,
                    modifiers
                );

                let mouse_button = match button {
                    winit::event::MouseButton::Left => Some(crate::input_state::MouseButton::Left),
                    winit::event::MouseButton::Right => Some(crate::input_state::MouseButton::Right),
                    winit::event::MouseButton::Middle => Some(crate::input_state::MouseButton::Middle),
                    _ => None
                };

                let mouse_event = match state {
                    winit::event::ElementState::Pressed => crate::input_state::MouseButtonEvent::Pressed,
                    winit::event::ElementState::Released => crate::input_state::MouseButtonEvent::Released,
                };

                if let Some(mouse_button) = mouse_button {
                    input_state.handle_mouse_button_event(mouse_button, mouse_event);
                }
            }

            Event::WindowEvent {
                event:
                WindowEvent::CursorMoved {
                    device_id,
                    position,
                    modifiers,
                },
                ..
            } => {
                trace!("mouse {:?} {:?} {:?}", device_id, position, modifiers);
                let physical_position = position.to_physical(self.hidpi_factor);

                input_state.handle_mouse_move_event(glam::Vec2::new(physical_position.x as f32, physical_position.y as f32));
            }

            // Ignore any other events
            _ => (),
        }

        if is_close_requested {
            trace!("close requested");
            app_control.enqueue_terminate_process();
        }
    }
}
