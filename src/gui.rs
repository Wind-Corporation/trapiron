//! Interface to the GUI backend.

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
}

impl Gui {
    /// Wraps the provided backend implementation of Gui with the public-facing type.
    fn from(backend: backend::Gui) -> Self {
        Self { backend }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Drawing basics
//

/// An active render pass.
///
/// A single instance of this object exists while a frame is being rendered.
pub struct DrawContext<'a>(backend::DrawContext<'a>);

/// Something that can be rendered.
pub trait Drawable {
    /// Draws this object using the provided DrawContext.
    fn draw(&mut self, ctxt: &mut DrawContext);
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
pub trait Application: Drawable {}

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
    pub uv: [Float; 2],
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
    pub uv: [Float; 2],
}

/// The simplest 3D object that can be drawn to the screen directly.
///
/// A Primitive3 is a collection of vertices, connected into triangles according to an vertex index
/// list, that has a texture associated with it.
pub struct Primitive3(backend::Primitive3);

impl Drawable for Primitive3 {
    fn draw(&mut self, ctxt: &mut DrawContext) {
        self.0.draw(ctxt);
    }
}

/// The simplest 2D object that can be drawn to the screen directly.
///
/// A Primitive2 is a collection of vertices, connected into triangles according to an vertex index
/// list, that has a texture associated with it.
pub struct Primitive2(backend::Primitive2);

impl Drawable for Primitive2 {
    fn draw(&mut self, ctxt: &mut DrawContext) {
        self.0.draw(ctxt);
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
    ) -> Result<Primitive3, PrimitiveError> {
        self.backend.make_primitive3(vertices, indices)
    }

    /// Creates a new [2D graphics primitive](Primitive2) from raw components.
    pub fn make_primitive2(
        &mut self,
        vertices: &[Vertex2],
        indices: &[Index],
    ) -> Result<Primitive2, PrimitiveError> {
        self.backend.make_primitive2(vertices, indices)
    }
}
