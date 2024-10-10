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
/// Use [`DrawContext::start_2`] or [`DrawContext::start_3`] to obtain a `Dcf` that can be used
/// for draw calls.
pub struct DrawContext<'a> {
    /// The [`Gui`] instance.
    gui: &'a mut Gui,

    /// The implementation provided by the backend.
    backend: backend::DrawContext<'a>,
}

impl<'a> DrawContext<'a> {
    /// Begins drawing operations in 2D.
    ///
    /// Creates the first `Dcf2` that will serve as the basis for the 2D draw state stack. After it
    /// is dropped, all changes to the drawing environment will be reset.
    pub fn start_2<'b>(&'b mut self) -> Dcf2<'b, 'a> {
        Dcf2 {
            ctxt: self,
            state: Default::default(),
        }
    }

    /// Begins drawing operations in 3D.
    ///
    /// Creates the first `Dcf2` that will serve as the basis for the 3D draw state stack. After it
    /// is dropped, all changes to the drawing environment will be reset.
    pub fn start_3<'b>(&'b mut self) -> Dcf3<'b, 'a> {
        Dcf3 {
            ctxt: self,
            state: Default::default(),
        }
    }
}

pub trait DcState: Clone {}

/// Mutable state used by drawing operations in 3D contexts.
///
/// See [`Dcf3`].
#[derive(Clone, Default)]
pub struct DcState3 {
    /// The transform from model coordinates to world coordinates, i.e. the position, scale and
    /// rotation of a `Primitive3` relative to the distant light sources.
    ///
    /// This value is used for lighting computations.
    pub world_transform: glam::Affine3A,
}

impl DcState for DcState3 {}

/// Mutable state used by drawing operations in 2D contexts.
///
/// See [`Dcf2`].
#[derive(Clone, Default)]
pub struct DcState2 {
    /// The transform from model coordinates to canvas coordinates, i.e. the position, scale and
    /// rotation of a `Primitive2` in the canvas coordinate system.
    pub transform: glam::Affine2,
}

impl DcState for DcState2 {}

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
pub struct Dcf<'a, 'b, S: DcState> {
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
    state: S,
}

impl<'a, 'b, S: DcState> Dcf<'a, 'b, S> {
    /// Returns the immutable [`DcState`] of this draw context frame.
    pub fn state(&self) -> &S {
        &self.state
    }

    /// Applies `func` to the state of this frame and pushes the result as a new frame.
    ///
    /// Does not alter the state associated with this frame; `func` is effectively undone when the
    /// returned value is dropped.
    ///
    /// `func` should mutate the provided [`DcState`] in place; it is operating on a mutable copy.
    pub fn apply<'c, F>(&'c mut self, func: F) -> Dcf<'c, 'b, S>
    where
        F: FnOnce(&mut S),
    {
        let mut state = self.state.clone();
        func(&mut state);
        Dcf {
            ctxt: &mut self.ctxt,
            state,
        }
    }
}

/// A `Dcf` for 3D contexts.
pub type Dcf3<'a, 'b> = Dcf<'a, 'b, DcState3>;

impl<'a, 'b> Dcf3<'a, 'b> {
    /// Returns a frame the applies `transform` before the rest of this frame's _world transform_.
    ///
    /// See [`Dcf3::apply`] for details.
    pub fn tfed<'c>(&'c mut self, transform: glam::Affine3A) -> Dcf3<'c, 'b> {
        self.apply(|s| s.world_transform *= transform)
    }
}

/// A `Dcf` for 2D contexts.
pub type Dcf2<'a, 'b> = Dcf<'a, 'b, DcState2>;

impl<'a, 'b> Dcf2<'a, 'b> {
    /// Returns a frame the applies `transform` before the rest of this frame's transform.
    ///
    /// See [`Dcf2::apply`] for details.
    pub fn tfed<'c>(&'c mut self, transform: glam::Affine2) -> Dcf2<'c, 'b> {
        self.apply(|s| s.transform *= transform)
    }
}

/// Something that can be rendered.
///
/// For custom types, implement [`Drawable2`] or [`Drawable3`] as appropriate.
pub trait Drawable<T: DcState> {
    /// Draws this object using the provided draw context frame.
    ///
    /// This is a wrapper for [`Drawable3::draw`] or [`Drawable2::draw`] as appropriate.
    fn draw_generic(&mut self, dcf: &mut Dcf<T>);
}

/// Something that can be rendered in a 3D context.
pub trait Drawable3 {
    /// Draws this object using the provided draw context frame.
    fn draw(&mut self, dcf: &mut Dcf3);
}

impl<T> Drawable<DcState3> for T
where
    T: Drawable3,
{
    fn draw_generic(&mut self, dcf: &mut Dcf3) {
        self.draw(dcf);
    }
}

/// Something that can be rendered in a 2D context.
pub trait Drawable2 {
    /// Draws this object using the provided draw context frame.
    fn draw(&mut self, dcf: &mut Dcf2);
}

impl<T> Drawable<DcState2> for T
where
    T: Drawable2,
{
    fn draw_generic(&mut self, dcf: &mut Dcf2) {
        self.draw(dcf);
    }
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

/// A vertex of a 3D [`Primitive3`].
#[derive(Copy, Clone)]
pub struct Vertex3 {
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

/// A vertex of a 2D [`Primitive2`].
#[derive(Copy, Clone)]
pub struct Vertex2 {
    /// The position (XYZ) of this vertex in its model's frame of reference.
    pub position: [Float; 2],

    /// The multiplicative color filter associated with this vertex.
    ///
    /// This is an RGBA vector with values in range `[0; 1]` for each component.
    ///
    /// If a texture is active, the color vector extracted from the texture is multiplied
    /// component-wise with this vector. If no texture is bound, this color is used without
    /// modification instead. The filter is interpolated linearly between vertices.
    pub color_multiplier: [Float; 4],

    /// The coordinates in texture space associated with this vertex (the UV-mapping of the vertex).
    ///
    /// This value is ignored when no texture is used.
    pub texture_coords: [Float; 2],
}

/// The simplest 3D object that can be drawn to the screen directly.
///
/// A Primitive3 is a collection of vertices, connected into triangles according to an vertex index
/// list, that has a set of textures associated with it.
pub struct Primitive3(backend::Primitive3);

impl Drawable3 for Primitive3 {
    fn draw(&mut self, dcf: &mut Dcf3) {
        self.0.draw(dcf);
    }
}

/// The simplest 2D object that can be drawn to the screen directly.
///
/// A Primitive2 is a collection of vertices, connected into triangles according to an vertex index
/// list, that has a set textures associated with it.
pub struct Primitive2(backend::Primitive2);

impl Drawable2 for Primitive2 {
    fn draw(&mut self, dcf: &mut Dcf2) {
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
    /// Creates a new [3D graphics primitive](Primitive3) from raw components.
    pub fn make_primitive3(
        &mut self,
        vertices: &[Vertex3],
        indices: &[Index],
        texture: Rc<Texture>,
    ) -> Result<Primitive3, PrimitiveError> {
        self.backend.make_primitive3(vertices, indices, texture)
    }

    /// Creates a new [2D graphics primitive](Primitive2) from raw components.
    pub fn make_primitive2(
        &mut self,
        vertices: &[Vertex2],
        indices: &[Index],
        texture: Rc<Texture>,
    ) -> Result<Primitive2, PrimitiveError> {
        self.backend.make_primitive2(vertices, indices, texture)
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
