//! Representation of game state.
//!
//! In Trapiron, whenever there is a player character, a [World] is available; each block-based
//! puzzles is stored in a [Level]. Both World and Level are only concerned with current game state,
//! replays and resets are implemented separately.
//!
//! Changes in worlds and levels occur only in reaction to [_events_](Event), such as the player
//! activating a button or a tick occurring. Events are serializable and reactions are
//! deterministic, enabling a system of verifiable replays.
//!
//! Non-tick events usually carry a change in player intent, while ticks act the intent and its
//! consequences out: a jump input enters simulation space as a non-tick event that only applies a
//! change in velocity to the player, whereas all movement that results from it is the result of
//! ensuing ticks.
//!
//! ## Ticks
//!
//! World and level states evolve with discrete updates called ticks. There are two kinds of ticks:
//! _logic ticks_ for most game mechanics and _presentation ticks_ for a few properties that should
//! ideally update every frame.
//!
//! **Logic ticks** occur with [fixed frequency (TPS)](TARGET_TPS) in simulation time. They are
//! predictable to the player and happen rarely enough to be useful for implementing most game
//! mechanics.
//!
//! **Presentation ticks** exist to update a few specific things such as camera movement at FPS
//! speeds. They have variable time in simulation, selected to roughly match realtime. As TPS is
//! slow compared to FPS, without presentation ticks, certain interactions would have significant
//! jerkiness or input lag.

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

/// A recorded change that can be applied to a [World].
#[derive(Debug)]
pub enum Event {
    /// A logic tick has occurred. Logic tick duration is fixed, see [`TARGET_TPS`].
    Tick,

    /// A presentation tick has occurred.
    PresentationTick {
        /// Step in simulation time that this tick corresponds to.
        duration: Duration,
    },

    /// tmp
    MoveCamera { direction: Vec3 },

    /// tmp: this should not be an event, it's a presentation setting like FoV or gamma
    RotateCamera { yaw: Float, pitch: Float },
}

/// Expected number of logic ticks per simulation second.
pub const TARGET_TPS: u32 = 20;

/// tmp: camera should be bound to player OR noclip. Maybe even cutscenes.
#[derive(Default)]
pub struct Camera {
    pub pos: Vec3,
    pub yaw: Float,
    pub pitch: Float,
    pub vel: Vec3,

    /// Target velocity in camera frame of reference
    pub control: Vec3,
}

/// State of game simulation.
///
/// Contains, directly or indirectly, the entire state of in-game world, including [levels](Level)
/// and their states, static environments, progress information and everything else in game logic
/// that isn't hardcoded.
///
/// Updated by discrete events, including logic and presentation ticks. See module description for
/// more details.
pub struct World {
    pub camera: Camera,
}

impl World {
    /// Create a new empty world.
    pub fn new() -> Self {
        Self {
            camera: Default::default(),
        }
    }

    /// Process an event related to a logic tick.
    pub fn process(&mut self, event: Event, _logic: &Logic) {
        match event {
            Event::Tick => (),
            Event::MoveCamera { direction } => {
                self.camera.control = direction;
            }
            _ => (),
        }
    }

    /// Process an event related to a presentation tick.
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
