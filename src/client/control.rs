//! Intepretation of GUI inputs as simulation controls.

use std::collections::VecDeque;

use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{
    client::view::Parameters,
    world::{Event, Vec3},
};

/// Logic and state of an interpreter of GUI inputs as in-game controls.
///
/// For example, converts a spacebar keystroke or a X controller button press into a jump input.
#[derive(Default)]
pub struct Control {
    /// Events accumulated from decoded from inputs that have not been fetched yet.
    pending: VecDeque<Event>,

    /// Last Event::SetCameraRotation.
    ///
    /// Unlike other events, if multiple camera inputs occur between [Self::pending_events], newest
    /// event overwrites all previous events.
    pending_set_camera_rotation: Option<Event>,

    /// Desired movement direction in camera frame of reference according to observed keyboard
    /// inputs.
    keyboard_camera_move_state: Vec3,

    /// Camera rotation according to last camera control input.
    last_camera_rotation: crate::world::YawPitch,
}

impl Control {
    pub fn new() -> Self {
        Self {
            pending: VecDeque::with_capacity(64),
            ..Default::default()
        }
    }

    /// Provides simulation events accumulated from processed inputs. Every event is only returned
    /// by this method once.
    pub fn pending_events(&mut self) -> impl Iterator<Item = Event> {
        let set_camera_rotation = std::iter::from_fn(|| self.pending_set_camera_rotation.take());
        let other = std::iter::from_fn(|| self.pending.pop_back());

        other.chain(set_camera_rotation)
    }

    /// Adjust view parameters according to inputs.
    pub fn tweak_view_parameters(&mut self, params: &mut Parameters) {
        use super::view::Camera::*;

        match &mut params.camera {
            Free { rotation, .. } => {
                *rotation = crate::gui::Quat::from_euler(
                    glam::EulerRot::XYZ,
                    0.0,
                    self.last_camera_rotation.pitch,
                    self.last_camera_rotation.yaw,
                )
            }
        }
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
                    self.pending.push_back(Event::MoveCamera {
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
                use crate::gui::*;

                const SENSITIVITY: Float = 0.004;
                let state = &mut self.last_camera_rotation;

                state.yaw += displacement.x * SENSITIVITY;
                state.yaw %= 2.0 * PI;

                state.pitch += displacement.y * SENSITIVITY;
                state.pitch = state.pitch.clamp(-PI / 2.0, crate::gui::PI / 2.0);

                self.pending_set_camera_rotation =
                    Some(Event::SetCameraRotation { rotation: *state });
            }
        }
    }
}
