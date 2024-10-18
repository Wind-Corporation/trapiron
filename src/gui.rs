//! Interface to the GUI backend.

use std::collections::HashMap;
use std::rc::{Rc, Weak};

pub mod backend_glium;

// To change the active backend, edit this line.
pub use backend_glium as backend;

pub mod asset;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Gui
//

/// A GUI backend.
///
/// This object manages the interactions with the underlying system libraries to provide graphics
/// rendering, input device events, and OS integration capabilities.
///
/// Due to the nature of most of these underlying systems, this object takes control over the main
/// thread. Use an object that implements Application to execute code at specific moments and to
/// react to events.
///
/// ## See also
/// backend::run
pub struct Gui {
    /// The implementation provided by the backend.
    backend: backend::Gui,

    /// The number of the frame that most recently started rendering.
    ///
    /// The counter is `0` before first frame, then it is incremented by one before invoking user
    /// code during each frame render.
    last_started_frame: u64,

    /// The moment this struct was constructed.
    start_time: std::time::Instant,

    /// All textures that have ever been created, keyed by texture name. Empty values should be
    /// treated as if they did not exist.
    ///
    /// Note that a [`Texture`] may become unreferenced, which will cause it to drop. This leaves an
    /// empty [`Weak`] in the map. Empty `Weak`s remain until the texture is re-created or until
    /// shutdown.
    texture_registry: HashMap<TextureId, Weak<Texture>>,
}

impl Gui {
    /// Wraps the provided backend implementation of Gui with the public-facing type.
    fn from(backend: backend::Gui) -> Self {
        Self {
            backend,
            last_started_frame: 0,
            start_time: std::time::Instant::now(),
            texture_registry: HashMap::new(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Drawing basics
//

/// An active render operation.
///
/// A single instance of this object exists while a frame is being rendered.
///
/// Use [`DrawContext::start_3`] to obtain a `Dcf` that can be used for draw calls.
pub struct DrawContext<'a> {
    /// The [`Gui`] instance.
    gui: &'a mut Gui,

    /// The implementation provided by the backend.
    backend: backend::DrawContext<'a>,

    /// The time moment that draw logic should use for this frame.
    time: std::time::Instant,
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
    ctxt: &'a mut DrawContext<'b>,

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

////////////////////////////////////////////////////////////////////////////////////////////////////
// Application
//

/// A business logic container for the GUI.
///
/// Objects of this type should directly or indirectly own all GUI-dependent resources such as
/// textures.
///
/// A single instance of a type implementing this trait will be instantiated by the GUI backend when
/// the GUI is ready, and this instance will be dropped just before the GUI shuts down.
///
/// ## See also
/// backend::run
pub trait Application {
    fn draw(&mut self, dcf: &mut DrawContext);
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Primitives
//

/// The floating-point type used for graphics computations.
pub type Float = f32;

/// The integer data type used to index into vertex arrays.
///
/// The current choice of `u16` limits the vertex arrays to a length of 65535.
pub type Index = u16;

/// A vertex of a 3D [`Primitive`].
#[derive(Copy, Clone)]
pub struct Vertex {
    /// The position (XYZ) of this vertex in its model's frame of reference.
    pub position: [Float; 3],

    /// The multiplicative color filter associated with this vertex.
    ///
    /// This is an RGB vector with values in range `[0; 1]` for each component.
    ///
    /// If a texture is active, the color vector extracted from the texture is multiplied
    /// component-wise with this vector. If no texture is bound, this color is used without
    /// modification instead. The filter is interpolated linearly between vertices.
    pub color_multiplier: [Float; 3],

    /// The coordinates in texture space associated with this vertex (the UV-mapping of the vertex).
    ///
    /// This value is ignored when no texture is used.
    pub texture_coords: [Float; 2],
}

/// The simplest 3D object that can be drawn to the screen directly.
///
/// A Primitive is a collection of vertices, connected into triangles according to an vertex index
/// list, that has a set of textures associated with it.
pub struct Primitive(backend::Primitive);

impl Drawable for Primitive {
    fn draw(&mut self, dcf: &mut Dcf) {
        self.0.draw(dcf);
    }
}

/// An error that might occur when creating a graphics primitive.
#[derive(Debug)]
pub enum PrimitiveError {
    /// The `indices` array contains an index that is out of bounds for the `vertices` array.
    IndexOutOfBounds {
        /// The offset into the `indices` array at which some invalid element was found.
        index_of_index: usize,
    },

    /// The `vertices` array is too large.
    TooManyVertices {
        /// The maximum allowed size of the `vertices` array.
        max_vertices: usize,
    },
}

impl Gui {
    /// Creates a new [3D graphics primitive](Primitive) from raw components.
    pub fn make_primitive(
        &mut self,
        vertices: &[Vertex],
        indices: &[Index],
        texture: Rc<Texture>,
    ) -> Result<Primitive, PrimitiveError> {
        self.backend.make_primitive(vertices, indices, texture)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Textures
//

/// A texture - an image that may be bound to geometry and drawn to the screen.
pub struct Texture(backend::Texture);

/// Texture group with options for texture loading.
///
/// It is expected that groups are `const` values, though this is not a hard requirement.
#[derive(Hash, PartialEq, Eq)]
pub struct TextureGroup {
    // Empty; will be expanded later
}

impl TextureGroup {
    /// Derives a [`TextureId`] that belongs to this group.
    pub fn id(&'static self, name: &'static str) -> TextureId {
        TextureId { name, group: self }
    }
}

/// The identifier of a [`Texture`] for loading.
///
/// The name is expected to be a string literal, and the group is expected to be a `const` value,
/// though these are not hard requirements.
///
/// Equally-named textures in different groups are considered different textures and are loaded
/// twice.
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct TextureId {
    /// The name of the texture.
    ///
    /// The texture will be loaded from `asset/gui/texture/{name}.png`.
    name: &'static str,

    /// The group that the texture belongs in.
    ///
    /// Texture group defines various options related to the texture.
    group: &'static TextureGroup,
}

impl Gui {
    /// Obtains a reference to a texture by its [id](TextureId), loading it if necessary.
    ///
    /// The method panics if the texture could not be loaded. Texture loading may occur more than
    /// once during program runtime.
    pub fn texture(&mut self, id: TextureId) -> Rc<Texture> {
        if let Some(ref mut weak) = self.texture_registry.get(&id) {
            if let Some(texture) = weak.upgrade() {
                return texture;
            }
        }

        let name = id.name;
        crate::crash::with_context(("Loading texture", || name), || {
            let image = asset::load_image(id.name);
            let texture = Rc::new(self.backend.make_texture(image, &id));
            self.texture_registry.insert(id, Rc::downgrade(&texture));
            texture
        })
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Colors
//

/// A color without transparency information.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct OpaqueColor(glam::Vec3);

impl OpaqueColor {
    /// Creates a new color from an RGB triplet.
    ///
    /// Expected channel values are `[0; 1]`, but this is not a strict requirement.
    pub fn rgb(rgb: glam::Vec3) -> Self {
        Self(rgb)
    }
}
