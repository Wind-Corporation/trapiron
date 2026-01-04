use std::time::Duration;

use crate::logic::Logic;

/// The floating-point type used for world state.
pub type Float = f32;

/// A Float 2D vector for world state.
pub type Vec2 = glam::f32::Vec2;

/// A Float 3D vector for world state.
pub type Vec3 = glam::f32::Vec3;

/// A Float 4D vector for world state.
pub type Vec4 = glam::f32::Vec4;

/// A Float 3x3 matrix vector for world state.
pub type Mat3 = glam::f32::Mat3;

/// A Float 4x4 matrix vector for world state.
pub type Mat4 = glam::f32::Mat4;

/// A Float 3x4 matrix vector (equivalent to mat4x3 in GLSL) for world state.
pub type Affine3 = glam::f32::Affine3A;

pub enum Event {
    Tick,
    PresentationTick { duration: Duration },
    MoveCamera { direction: Vec3 },
    RotateCamera { yaw: Float, pitch: Float },
}

#[derive(Default)]
pub struct Camera {
    pub pos: Vec3,
    pub yaw: Float,
    pub pitch: Float,
    pub vel: Vec3,
    pub control: Vec3,
}

pub struct World {
    pub camera: Camera,
}

impl World {
    pub fn new() -> Self {
        Self {
            camera: Default::default(),
        }
    }

    pub fn process(&mut self, event: Event, _logic: &Logic) {
        match event {
            Event::Tick => (),
            Event::MoveCamera { direction } => {
                self.camera.control = direction;
            }
            _ => (),
        }
    }

    pub fn process_presentation(&mut self, event: &Event, _logic: &Logic) {
        match event {
            Event::PresentationTick { duration } => {
                let dt = duration.as_secs_f32();

                const CONTROL_ACCELERATION: Float = 50.0;
                const CONTROL_SPEED: Float = 5.0;
                let target =
                    Mat3::from_rotation_z(self.camera.yaw) * self.camera.control * CONTROL_SPEED;

                let dv = target - self.camera.vel;
                let dv = dv.clamp_length_max(CONTROL_ACCELERATION * dt);
                self.camera.vel += dv;

                self.camera.pos += self.camera.vel * dt;
                self.camera.vel *= 0.25f32.powf(dt);
            }
            Event::RotateCamera { yaw, pitch } => {
                self.camera.yaw = *yaw;
                self.camera.pitch = *pitch;
            }
            _ => (),
        }
    }
}
