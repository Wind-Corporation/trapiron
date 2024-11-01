//! Drawing context and other concepts for graphical output.
//!
//! ## Object heirarchy
//! - [`Dcf`]: The user-facing drawing API and a RAII configurator of volatile drawing parameters.
//!   - [`State`]: Volatile drawing parameters such world transform.
//!   - [`Context`]: A hidden container for rarely changed render settings and resources.
//!     - [`Gui`]
//!     - [`backend::DrawContext`](super::backend::DrawContext): Backend-specific container of
//!       render resources.

use super::{Affine3, Gui, Mat4, OpaqueColor, Vec2, Vec3};

/// An active render operation.
///
/// A single instance of this object exists while a frame is being rendered.
pub(super) struct Context<'a> {
    /// The [`Gui`] instance.
    pub gui: &'a mut Gui,

    /// The implementation provided by the backend.
    pub backend: super::backend::DrawContext<'a>,

    /// The viewport size for this frame.
    ///
    /// See [`Dcf::size`] for a more specific definition.
    pub size: Vec2,

    /// The time moment that draw logic should use for this frame.
    pub time: std::time::Instant,

    /// The current render settings.
    ///
    /// May infrequently change during one frame render.
    pub settings: Settings,
}

/// Mutable state used by drawing operations.
///
/// Unlike [`Settings`], these parameters are expected to change frequently while a single frame is
/// rendered.
///
/// See [`Dcf`].
#[derive(Clone)]
pub struct State {
    /// The transform from model coordinates to world coordinates, i.e. the position, scale and
    /// rotation of a `Primitive` relative to the distant light sources.
    ///
    /// This value is used for lighting computations.
    pub world_transform: Affine3,

    /// A global color multiplier.
    ///
    /// All pixel colors will be multiplied by this color in RGB space without gamma correction.
    pub color_multiplier: OpaqueColor,
}

impl Default for State {
    fn default() -> Self {
        Self {
            world_transform: Affine3::IDENTITY,
            color_multiplier: OpaqueColor::rgb(Vec3::splat(1.0)),
        }
    }
}

/// A proxy for draw calls available to [`Drawable`].
///
/// Each instance a `Dcf` corresponds to particular immutable settings for drawing operations,
/// stored in a [`State`]. This data is primarily used by
/// [`Primitive::draw`](super::Primitive::draw), but it is also accessible via [`Dcf::state`].
///
/// `Dcf` values are immutable, but a child frame with mutated state can be created. This
/// corresponds to pushing a frame onto the state stack. The child frame will restore settings by
/// popping a single `State` off of the stack when it is dropped.
///
/// The name stands for _Draw Context Frame_, referring to frames of the state stack.
///
/// To prevent confusion, using a `Dcf` that does not represent the top of the state stack is
/// disallowed at compile time.
pub struct Dcf<'a, 'b> {
    /// The underlying draw context that is "shared" between all frames.
    ///
    /// The reference is owned by the `Dcf` at the top of the stack.
    pub(super) ctxt: &'a mut Context<'b>,

    /// The state of the frame.
    ///
    /// Psych! The state stack _is_ the call stack. Don't count on it, though: it is an
    /// implementation detail.
    ///
    /// For a single `Dcf`, this is an immutable field.
    state: State,
}

impl<'a, 'b> Dcf<'a, 'b> {
    /// Applies `func` to the state of this frame and pushes the result as a new frame.
    ///
    /// Does not alter the state associated with this frame; `func` is effectively undone when the
    /// returned value is dropped.
    ///
    /// `func` should mutate the provided [`State`] in place; it is operating on a mutable copy.
    pub fn apply<'c, F>(&'c mut self, func: F) -> Dcf<'c, 'b>
    where
        F: FnOnce(&mut State),
    {
        let mut state = self.state.clone();
        func(&mut state);
        Dcf {
            ctxt: &mut self.ctxt,
            state,
        }
    }

    /// Returns the time instant that draw logic should use.
    pub fn time(&self) -> &std::time::Instant {
        &self.ctxt.time
    }

    /// Returns the size of the viewport in pixels that draw logic should use.
    ///
    /// Returns the size of the renderable area ("window client area", "clip space", "viewport")
    /// measured in logical pixels (not necessarily physical display pixels). The dimensions are
    /// positive but not necessarily integer values.
    pub fn size(&self) -> Vec2 {
        self.ctxt.size
    }

    /// Returns a reference to the [`Gui`] instance.
    pub fn gui(&mut self) -> &mut Gui {
        &mut self.ctxt.gui
    }

    /// Returns the immutable [`State`] of this draw context frame.
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Returns a reference to the current [settings](Settings) associated with the underlying
    /// render context.
    ///
    /// Note that unlike [`State`], a single [`Settings`] value is shared by every `Dcf` during
    /// a single frame render.
    pub fn settings(&self) -> &Settings {
        &self.ctxt.settings
    }

    /// Changes the current [settings](Settings) associated with the underlying render context.
    ///
    /// Note that unlike [`State`], a single [`Settings`] value is shared by every `Dcf` during
    /// a single frame render.
    pub fn set_settings(&mut self, new_settings: Settings) {
        self.ctxt.settings = new_settings;
    }

    /// In a new frame, applies the `transform` to _world transform_.
    ///
    /// See [`Dcf::apply`] for details.
    pub fn tfed<'c>(&'c mut self, transform: Affine3) -> Dcf<'c, 'b> {
        self.apply(|s| s.world_transform *= transform)
    }

    /// In a new frame, applies a translation such that (0; 0; 0) maps to `new_zero` in this frame.
    ///
    /// See [`Dcf::apply`] for details.
    pub fn shifted<'c>(&'c mut self, new_zero: Vec3) -> Dcf<'c, 'b> {
        self.tfed(Affine3::from_translation(new_zero))
    }

    /// In a new frame, scales such that a unit cube has dimentions `new_units` in this frame.
    ///
    /// See [`Dcf::apply`] for details.
    pub fn scaled<'c>(&'c mut self, new_units: Vec3) -> Dcf<'c, 'b> {
        self.tfed(Affine3::from_scale(new_units))
    }

    /// In a new frame, applies an additional color multiplier filter to rendered primitives.
    ///
    /// See [`Dcf::apply`] for details.
    pub fn colored<'c>(&'c mut self, filter: &OpaqueColor) -> Dcf<'c, 'b> {
        self.apply(|s| s.color_multiplier.0 *= filter.0)
    }

    /// Creates the first frame from a raw [`Context`].
    pub(super) fn new(ctxt: &'a mut Context<'b>) -> Self {
        Self {
            ctxt,
            state: Default::default(),
        }
    }
}

/// Mostly static parameters used by drawing operations.
///
/// Unlike [`State`], these values are expected to mostly stay constant while a single frame is
/// rendered, though they may change once or twice.
#[derive(Clone, Default)]
pub struct Settings {
    /// The transform from world coordinates to view coordinates.
    ///
    /// The inverse of the camera pose. This value is ignored for lighting computations.
    pub view_transform: Affine3,

    /// The transform from view coordinates to screen coordinates; normally an orthographic or a
    /// perspective projection.
    ///
    /// Transforms 3D camera-centric coordinates to 2D screen-based normalized coordinates in the
    /// [-1;+1] range for X and Y.
    pub screen_transform: Mat4,
}

/// Something that can be rendered onto the screen.
pub trait Drawable {
    /// Draws this object using the provided draw context frame.
    fn draw(&mut self, dcf: &mut Dcf);
}
