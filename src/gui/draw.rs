use super::{backend, Gui, OpaqueColor};

/// An active render operation.
///
/// A single instance of this object exists while a frame is being rendered.
///
/// Use [`DrawContext::start_3`] to obtain a `Dcf` that can be used for draw calls.
pub struct DrawContext<'a> {
    /// The [`Gui`] instance.
    pub(crate) gui: &'a mut Gui,

    /// The implementation provided by the backend.
    pub(crate) backend: backend::DrawContext<'a>,

    /// The time moment that draw logic should use for this frame.
    pub(crate) time: std::time::Instant,
}

impl<'a> DrawContext<'a> {
    /// Begins drawing operations in 3D.
    ///
    /// Creates the first `Dcf` that will serve as the basis for the 3D draw state stack. After it
    /// is dropped, all changes to the drawing environment will be reset.
    pub fn start_3<'b>(&'b mut self) -> Dcf<'b, 'a> {
        Dcf {
            ctxt: self,
            state: Default::default(),
        }
    }

    /// Returns the time instant that draw logic should use.
    pub fn time(&self) -> &std::time::Instant {
        &self.time
    }

    /// Returns a reference to the [`Gui`] instance.
    pub fn gui(&mut self) -> &mut Gui {
        self.gui
    }
}

/// Mutable state used by drawing operations in 3D contexts.
///
/// See [`Dcf`].
#[derive(Clone)]
pub struct DcState {
    /// The transform from model coordinates to world coordinates, i.e. the position, scale and
    /// rotation of a `Primitive` relative to the distant light sources.
    ///
    /// This value is used for lighting computations.
    pub world_transform: glam::Affine3A,

    /// A global color multiplier.
    ///
    /// All pixel colors will be multiplied by this color in RGB space without gamma correction.
    pub color_multiplier: OpaqueColor,
}

impl Default for DcState {
    fn default() -> Self {
        Self {
            world_transform: glam::Affine3A::IDENTITY,
            color_multiplier: OpaqueColor::rgb(glam::Vec3::splat(1.0)),
        }
    }
}

/// A proxy for draw calls available to [`Drawable`].
///
/// Each instance a `Dcf` corresponds to particular immutable settings for drawing operations,
/// stored in a [`DcState`]. This data is primarily used by _PrimitiveN::draw_, but it is also
/// accessible via [`Dcf::state`].
///
/// `Dcf` values are immutable, but a child frame with mutated state can be created. This
/// corresponds to pushing a frame onto the state stack. The child frame will restore settings by
/// popping a single `DcState` off of the stack when it is dropped.
///
/// The name stands for _Draw Context Frame_, referring to frames of the state stack.
///
/// To prevent confusion, using a `Dcf` that does not represent the top of the state stack is
/// disallowed at compile time.
pub struct Dcf<'a, 'b> {
    /// The underlying draw context that is "shared" between all frames.
    ///
    /// The reference is owned by the `Dcf` at the top of the stack.
    pub ctxt: &'a mut DrawContext<'b>,

    /// The state of the frame.
    ///
    /// Psych! The state stack _is_ the call stack. Don't count on it, though: it is an
    /// implementation detail.
    ///
    /// For a single `Dcf`, this is an immutable field.
    state: DcState,
}

impl<'a, 'b> Dcf<'a, 'b> {
    /// Applies `func` to the state of this frame and pushes the result as a new frame.
    ///
    /// Does not alter the state associated with this frame; `func` is effectively undone when the
    /// returned value is dropped.
    ///
    /// `func` should mutate the provided [`DcState`] in place; it is operating on a mutable copy.
    pub fn apply<'c, F>(&'c mut self, func: F) -> Dcf<'c, 'b>
    where
        F: FnOnce(&mut DcState),
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
        self.ctxt.time()
    }

    /// Returns a reference to the [`Gui`] instance.
    pub fn gui(&mut self) -> &mut Gui {
        self.ctxt.gui()
    }

    /// Returns the immutable [`DcState`] of this draw context frame.
    pub fn state(&self) -> &DcState {
        &self.state
    }

    /// In a new frame, applies the `transform` to _world transform_.
    ///
    /// See [`Dcf::apply`] for details.
    pub fn tfed<'c>(&'c mut self, transform: glam::Affine3A) -> Dcf<'c, 'b> {
        self.apply(|s| s.world_transform *= transform)
    }

    /// In a new frame, applies a translation such that (0; 0; 0) maps to `new_zero` in this frame.
    ///
    /// See [`Dcf::apply`] for details.
    pub fn shifted<'c>(&'c mut self, new_zero: glam::Vec3) -> Dcf<'c, 'b> {
        self.tfed(glam::Affine3A::from_translation(new_zero))
    }

    /// In a new frame, scales such that a unit cube has dimentions `new_units` in this frame.
    ///
    /// See [`Dcf::apply`] for details.
    pub fn scaled<'c>(&'c mut self, new_units: glam::Vec3) -> Dcf<'c, 'b> {
        self.tfed(glam::Affine3A::from_scale(new_units))
    }

    /// In a new frame, applies an additional color multiplier filter to rendered primitives.
    ///
    /// See [`Dcf::apply`] for details.
    pub fn colored<'c>(&'c mut self, filter: &OpaqueColor) -> Dcf<'c, 'b> {
        self.apply(|s| s.color_multiplier.0 *= filter.0)
    }
}

/// Something that can be rendered in a 3D context.
pub trait Drawable {
    /// Draws this object using the provided draw context frame.
    fn draw(&mut self, dcf: &mut Dcf);
}
