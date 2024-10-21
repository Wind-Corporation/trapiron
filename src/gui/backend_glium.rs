//! GUI backend based on Glium for Linux (X11, Wayland), Windows and MacOS.
//!
//! Do not use path `gui::backend_glium` unless writing code that speicifically requires this
//! backend. Use `gui::*` wrappers, or use `gui::backend` when implementing these wrappers.

use super::{Index, Vertex};
use crate::crash;
use glium::winit;
use glium::Surface; // OpenGL interface
use std::rc::Rc;

mod winit_lifecycle;

// Shorthand
type WindowDisplay = glium::Display<glium::glutin::surface::WindowSurface>;

/// The super::Gui trait implementation for the Glium backend.
///
/// Only one object of this type should normally be instantiated, as it owns most of Glium
/// resources.
///
/// All interactions with Gui objects must happen in main application thread.
pub struct Gui {
    /// OpenGL program for 3D visuals with lighting support.
    program: glium::Program,

    /// The [`glium::Display`] instance of the main window that may be used for OpenGL operations.
    display: WindowDisplay,

    /// The main window.
    ///
    /// Implementation note: this must be the last field to prevent deadlocks on drop.
    window: winit::window::Window,
}

pub use winit_lifecycle::run;

impl Gui {
    /// Performs initialization of the basic GUI resources required to implement super::Gui methods.
    ///
    /// An OS window and an OpenGL context are created.
    ///
    /// Returned values include the constructed Gui instance and an winit event loop object.
    /// The latter must be forwarded to Gui::run_main_loop as a requirement of Glium library.
    fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> super::Gui {
        let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
            .with_title("Trapiron")
            .build(event_loop);

        let program = glium::Program::from_source(
            &display,
            include_str!("backend_glium/shader/vertex.glsl"),
            include_str!("backend_glium/shader/fragment.glsl"),
            None,
        )
        .expect("Could not create GLSL shared program");

        super::Gui::from(Self {
            program,
            display,
            window,
        })
    }
}

/// Processes a single Glium event.
///
/// Method arguments, other than `app`, correspond to the callback interface of
/// winit::event_loop::run.
fn handle_event(
    gui: &mut super::Gui,
    app: &mut impl super::Application,
    event: &winit::event::WindowEvent,
    event_loop: &winit::event_loop::ActiveEventLoop,
) {
    use winit::event::WindowEvent::*;

    match &event {
        CloseRequested => event_loop.exit(),

        Resized(window_size) => {
            gui.backend.display.resize((*window_size).into());
        }

        RedrawRequested => process_frame(gui, app),

        _ => (),
    };
}

/// Processes a single OpenGL frame.
///
/// This method, among other responsibilities, issues all OpenGL drawing commands via the
/// application object. However, no input events are issued.
fn process_frame(gui: &mut super::Gui, app: &mut impl super::Application) {
    gui.last_started_frame += 1;

    let frame_number = gui.last_started_frame;
    crash::with_context(("Current frame", || frame_number), || {
        let ctxt = DrawContext {
            target: gui.backend.display.draw(),
            _phantom: std::marker::PhantomData,
        };

        let mut ctxt = super::draw::Context {
            gui,
            backend: ctxt,
            time: std::time::Instant::now(),
        };

        ctxt.backend.target.clear_color(0.0, 0.0, 0.0, 1.0);
        app.draw(&mut ctxt);
        ctxt.backend
            .target
            .finish()
            .expect("OpenGL drawing sequence failed");
    });
}

/// The super::DrawContext implementation for the Glium backend.
pub struct DrawContext<'a> {
    target: glium::Frame,
    _phantom: std::marker::PhantomData<&'a ()>,
}

glium::implement_vertex!(Vertex, position, color_multiplier, texture_coords);

pub struct Primitive {
    vertices: glium::VertexBuffer<Vertex>,
    indices: glium::IndexBuffer<Index>,
    texture: Rc<super::Texture>,
}

impl super::Drawable for Primitive {
    fn draw(&mut self, dcf: &mut super::Dcf) {
        let sampler = self
            .texture
            .0
            .sampled()
            .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
            .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest);

        let uniforms = glium::uniform! {
            world_transform: dcf.state().world_transform.to_cols_array_2d(),
            color_multiplier_global: dcf.state().color_multiplier.0.to_array(),
            tex: sampler,
        };

        dcf.ctxt
            .backend
            .target
            .draw(
                &self.vertices,
                &self.indices,
                &dcf.ctxt.gui.backend.program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
    }
}

impl Gui {
    pub fn make_primitive(
        &mut self,
        vertices: &[Vertex],
        indices: &[Index],
        texture: Rc<super::Texture>,
    ) -> Result<super::Primitive, super::PrimitiveError> {
        // TODO Check validity of indices and length of vertices

        let vertices = glium::VertexBuffer::new(&self.display, vertices)
            .expect("Could not create a vertex buffer");

        let indices = glium::IndexBuffer::new(
            &self.display,
            glium::index::PrimitiveType::TrianglesList,
            indices,
        )
        .expect("Could not create an index buffer");

        Ok(super::Primitive(Primitive {
            vertices,
            indices,
            texture,
        }))
    }
}

pub type Texture = glium::texture::Texture2d;

impl Gui {
    pub fn make_texture(
        &mut self,
        image: image::DynamicImage,
        _id: &super::TextureId,
    ) -> super::Texture {
        use glium::texture::{MipmapsOption, RawImage2d, Texture2d};

        let image = image.to_rgba8();
        let image_dimensions = image.dimensions();
        let image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        let texture = Texture2d::with_mipmaps(&self.display, image, MipmapsOption::NoMipmap)
            .expect("Could not upload texture to GPU");
        super::Texture(texture)
    }
}
