//! Character state and related concepts.

use super::*;

/// State of the player character relevant to the game logic.
pub struct Character {
    /// Position of character (center of feet) in world coordinate frame.
    ///
    /// This property is updated once per presentation tick.
    pub position: Vec3,

    /// Speed (magnitude and direction) that the character is travelling at, in world coordinate
    /// frame.
    ///
    /// This property is updated once per presentation tick.
    pub velocity: Vec3,

    /// The direction the character is supposed to look.
    ///
    /// This property is updated once per presentation tick. Even when the player is controlling the
    /// character and using the player character camera, actual view direction of the camera may
    /// differ from this value due to non-input frames.
    pub rotation: YawPitch,

    /// Direction the character wishes to move in.
    ///
    /// todo: replace with vec2 + jump input when collisions are added
    ///
    /// This property is updated once per presentation tick.
    control: Vec3,
}

impl Character {
    /// tmp
    pub fn new() -> Self {
        Self {
            position: Default::default(),
            velocity: Default::default(),
            rotation: Default::default(),
            control: Default::default(),
        }
    }

    /// Get position of the character's eyes in world coordinate frame.
    ///
    /// This value should be used for camera positioning, raycasting, visiblity computations, etc.
    pub fn eye(&self) -> Vec3 {
        const EYE_POS: Vec3 = Vec3::new(0.0, 0.0, 1.65);
        self.position + EYE_POS
    }

    /// Handle an event and update self accordingly if necessary.
    pub fn process(&mut self, event: &Event) {
        match event {
            Event::PresentationTick { duration } => {
                let dt: Float = duration.as_secs_f32();

                const CONTROL_ACCELERATION: Float = 50.0;
                const CONTROL_SPEED: Float = 5.0;

                let target =
                    Mat3::from_rotation_z(-self.rotation.yaw) * self.control * CONTROL_SPEED;

                let dv = target - self.velocity;
                let dv = dv.clamp_length_max(CONTROL_ACCELERATION * dt);
                self.velocity += dv;

                self.position += self.velocity * dt;
                self.velocity *= (0.25 as Float).powf(dt); // TODO powf is not deterministic
            }
            Event::SetCameraRotation { rotation } => {
                self.rotation = *rotation;
            }
            Event::MoveCamera { direction } => {
                self.control = *direction;
            }
            _ => {}
        }
    }
}
