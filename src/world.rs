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
//! Ticks represent an advancement of simulation time by some duration. Non-tick events occur
//! instanteneously, though there is a well-defined order they occur in.
//!
//! **Logic ticks** occur with [fixed frequency (TPS)](TARGET_TPS) in simulation time. They are
//! predictable to the player and happen rarely enough to be useful for implementing most game
//! mechanics.
//!
//! **Presentation ticks** exist to update a few specific things such as camera movement at FPS
//! speeds. They have variable time in simulation, selected to roughly match realtime. As TPS is
//! slow compared to FPS, without presentation ticks, certain interactions would have significant
//! jerkiness or input lag.

pub mod array3;
pub mod vec_iter;

use std::time::Duration;

use ndarray::Array3;

use crate::{
    content::{self, Resources, block::Block},
    logic::Logic,
};

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

/// An unsigned integer 3D vector for world state.
pub type UVec3 = glam::u32::UVec3;

/// Euclidean angles yaw and pitch.
#[derive(Debug, Clone, Copy, Default)]
pub struct YawPitch {
    /// Left-to-right rotation, with negative values to the left and positive values to the right.
    pub yaw: Float,

    /// Down-to-up rotation, with negative values downwards and positive values upwards;
    /// zero corresponds to horizontal.
    pub pitch: Float,
}

/// A recorded change that can be applied to a [World].
#[derive(Debug, Clone)]
pub enum Event {
    /// A logic tick has occurred. Logic tick duration is fixed, see [`TARGET_TPS`].
    LogicTick,

    /// A presentation tick has occurred.
    PresentationTick {
        /// Step in simulation time that this tick corresponds to.
        duration: Duration,
    },

    /// tmp
    MoveCamera { direction: Vec3 },

    /// tmp
    SetCameraRotation { rotation: YawPitch },
}

/// Expected number of logic ticks per simulation second.
pub const TARGET_TPS: u32 = 20;

/// Desired simulation duration of a logic tick.
pub fn target_tick_duration() -> Duration {
    std::time::Duration::from_secs(1) / crate::world::TARGET_TPS
}

/// The state of a level: a portion of a [world](World) with a mutable block grid that can be
/// attempted.
pub struct Level {
    pub blocks: Array3<Block>,

    /// Location of the origin of the level in world coordinates.
    pub position: Vec3,

    /// Rotation of the level coordinate system around Z axis. Applied after position.
    pub yaw: Float,
}

impl Level {
    /// tmp
    pub fn new(rsrc: &Resources) -> Self {
        let block = |name: &str| {
            // Wow, this must be the filthiest code I ever wrote
            let ser = (name.chars().last().unwrap() as u32) - ('0' as u32);
            let serialized = content::block::Serialized(ser);
            let len = name.len();
            rsrc.blocks
                .get(&name[..len - 2])
                .unwrap()
                .instantiate(&serialized)
        };

        let mut result = Self {
            blocks: Array3::from_shape_fn((10, 10, 10), |_| Default::default()),
            position: Vec3::new(0., 5., 0.),
            yaw: 0.1,
        };

        for x in 0..10 {
            for y in 0..10 {
                result.blocks[[x, y, 0]] = block("stone:0");
                let dx = 5i32 - (x as i32);
                let dy = 5i32 - (y as i32);
                if dx * dx + dy * dy > 15 {
                    result.blocks[[x, y, 1]] = block("sand:0");
                }
            }
        }

        result
    }
}

/// tmp: camera should be bound to player OR noclip. Maybe even cutscenes.
#[derive(Default)]
pub struct Camera {
    pub pos: Vec3,
    pub rotation: YawPitch,
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
    pub levels: Vec<Level>,
    pub camera: Camera,
}

impl World {
    /// tmp
    pub fn new(rsrc: &Resources) -> Self {
        Self {
            levels: vec![Level::new(rsrc)],
            camera: Default::default(),
        }
    }

    /// Process an event related to a logic tick.
    pub fn process(&mut self, event: Event, _logic: &Logic) {
        match event {
            Event::LogicTick => (),
            Event::PresentationTick { duration } => {
                let dt: Float = duration.as_secs_f32();

                const CONTROL_ACCELERATION: Float = 50.0;
                const CONTROL_SPEED: Float = 5.0;

                let target = Mat3::from_rotation_z(-self.camera.rotation.yaw)
                    * self.camera.control
                    * CONTROL_SPEED;

                let dv = target - self.camera.vel;
                let dv = dv.clamp_length_max(CONTROL_ACCELERATION * dt);
                self.camera.vel += dv;

                self.camera.pos += self.camera.vel * dt;
                self.camera.vel *= (0.25 as Float).powf(dt);
            }
            Event::SetCameraRotation { rotation } => {
                self.camera.rotation = rotation;
            }
            Event::MoveCamera { direction } => {
                self.camera.control = direction;
            }
        }
    }
}
