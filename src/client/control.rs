//! Intepretation of GUI inputs as simulation controls.

use std::{collections::VecDeque, f32::consts::PI};

use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::world::{Event, Vec2, Vec3};

/// Logic and state of an interpreter of GUI inputs as in-game controls.
///
/// For example, converts a spacebar keystroke or a X controller button press into a jump input.
#[derive(Default)]
pub struct Control {
    /// Desired movement direction in camera frame of reference according to observed keyboard
    /// inputs.
    keyboard_camera_move_state: Vec3,

    /// tmp: this should not be an event, it's a presentation setting like FoV or gamma
    mouse_camera_rotate_state: Vec2,

    /// tmp
    accumulator: VecDeque<Event>,
}

impl Control {
    pub fn new() -> Self {
        Self {
            accumulator: VecDeque::with_capacity(64),
            ..Default::default()
        }
    }

    /// Provides simulation events accumulated from processed inputs. Every event is only returned
    /// by this method once.
    pub fn pending_events(&mut self) -> impl Iterator<Item = Event> {
        std::iter::from_fn(|| self.accumulator.pop_back())
    }

    /// Process a GUI _input_ and interpret it as a game control if applicable.
    ///
    /// Stores resulting control [events](Event) in an internal buffer, to be picked up later during
    /// [`fetch_into`](Self::fetch_into).
    pub fn on_input(&mut self, input: crate::gui::Input, gui: &mut crate::gui::Gui) {
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

                if let KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                    state: ElementState::Pressed,
                    repeat: false,
                    ..
                } = key_event
                {
                    gui.set_cursor_captured(!gui.cursor_captured());
                }
            }

            CapturedCursorMove { displacement } => {
                const SENSITIVITY: crate::gui::Float = 0.004;
                let yaw = &mut self.mouse_camera_rotate_state.x;
                let pitch = &mut self.mouse_camera_rotate_state.y;

                *yaw -= displacement.x * SENSITIVITY;
                *yaw %= 2f32 * PI;

                *pitch += displacement.y * SENSITIVITY;
                *pitch = pitch.clamp(-PI / 2f32, PI / 2f32);

                self.accumulator.push_back(Event::RotateCamera {
                    yaw: *yaw,
                    pitch: *pitch,
                });
            }
        }
    }
}
