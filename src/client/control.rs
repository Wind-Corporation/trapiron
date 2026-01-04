use std::collections::VecDeque;

use winit::{event::ElementState, keyboard::KeyCode};

use crate::world::{Event, Vec3};

#[derive(Default)]
pub struct Control {
    keyboard_camera_move_state: Vec3,

    accumulator: VecDeque<Event>,
}

impl Control {
    pub fn new() -> Self {
        Self {
            accumulator: VecDeque::with_capacity(64),
            ..Default::default()
        }
    }

    pub fn fetch_into(&mut self, out: &mut Vec<Event>) {
        out.reserve(self.accumulator.len());
        while let Some(event) = self.accumulator.pop_back() {
            out.push(event);
        }
    }

    pub fn on_input(&mut self, input: crate::gui::Input) {
        use crate::gui::Input::*;

        match input {
            Keyboard(key_event) => {
                if key_event.repeat {
                    return;
                }

                if let Some((_, dmove)) = [
                    (KeyCode::KeyW, Vec3::X),
                    (KeyCode::KeyS, -Vec3::X),
                    (KeyCode::KeyA, Vec3::Y),
                    (KeyCode::KeyD, -Vec3::Y),
                    (KeyCode::Space, Vec3::Z),
                    (KeyCode::ShiftLeft, -Vec3::Z),
                ]
                .iter()
                .find(|s| key_event.physical_key == s.0)
                {
                    let mut dmove = *dmove;
                    if key_event.state == ElementState::Released {
                        dmove *= -1f32;
                    };
                    self.keyboard_camera_move_state += dmove;
                    self.accumulator.push_back(Event::MoveCamera {
                        direction: self.keyboard_camera_move_state.clamp_length_max(1f32),
                    });
                }
            }
        }
    }
}
