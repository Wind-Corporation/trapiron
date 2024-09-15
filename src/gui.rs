//! Interface to the GUI backend.

pub mod backend_glium;

// To change the active backend, edit this line.
pub use backend_glium as backend;

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
pub struct Gui(backend::Gui);

impl Gui {}

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

pub type Float = f32;
pub type Index = u16;

#[derive(Copy, Clone)]
pub struct Vertex3 {
    pub position: [Float; 3],
    pub color_multiplier: [Float; 3],
    pub uv: [Float; 2],
}

#[derive(Copy, Clone)]
pub struct Vertex2 {
    pub position: [Float; 2],
    pub color_multiplier: [Float; 4],
    pub uv: [Float; 2],
}

/// The simplest object that can be drawn to the screen directly.
///
/// A Primitive is a collection of vertices, connected into triangles according to an vertex index
/// list, that has a texture and a render program associated with it.
pub struct Primitive3(backend::Primitive3);

impl Drawable for Primitive3 {
    fn draw(&mut self, ctxt: &mut DrawContext) {
        self.0.draw(ctxt);
    }
}

impl Gui {
    pub fn make_primitive(&mut self, vertices: &[Vertex3], indices: &[Index]) -> Primitive3 {
        self.0.make_primitive(vertices, indices)
    }
}
