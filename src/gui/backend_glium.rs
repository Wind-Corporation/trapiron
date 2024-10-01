//! GUI backend based on Glium for Linux (X11, Wayland), Windows and MacOS.
//!
//! Do not use path `gui::backend_glium` unless writing code that speicifically requires this
//! backend. Use `gui::*` wrappers, or use `gui::backend` when implementing these wrappers.

use super::{Index, Vertex2, Vertex3};
use crate::crash;
use glium::winit;
use glium::Surface; // OpenGL interface

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
    /// The number of the frame that most recently started rendering.
    ///
    /// The counter is `0` before first frame, then it is incremented by one before invoking user
    /// code during each frame render.
    last_started_frame: u64,

    /// The moment this struct was constructed.
    start_time: std::time::Instant,

    /// OpenGL program for 3D visuals with lighting support.
    program_3d: glium::Program,

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

        let program_3d = glium::Program::from_source(
            &display,
            include_str!("vertex_3d.glsl"),
            include_str!("fragment_3d.glsl"),
            None,
        )
        .expect("Could not create GLSL shared program");

        super::Gui::from(Self {
            last_started_frame: 0,
            start_time: std::time::Instant::now(),
            program_3d,
            display,
            window,
        })
    }

    /// Processes a single Glium event.
    ///
    /// Method arguments, other than `app`, correspond to the callback interface of
    /// winit::event_loop::run.
    fn handle_event(
        &mut self,
        app: &mut impl super::Application,
        event: &winit::event::WindowEvent,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) {
        use winit::event::WindowEvent::*;

        match &event {
            CloseRequested => event_loop.exit(),

            Resized(window_size) => {
                self.display.resize((*window_size).into());
            }

            RedrawRequested => self.process_frame(app),

            _ => (),
        };
    }

    /// Processes a single OpenGL frame.
    ///
    /// This method, among other responsibilities, issues all OpenGL drawing commands via the
    /// application object. However, no input events are issued.
    fn process_frame(&mut self, app: &mut impl super::Application) {
        self.last_started_frame += 1;

        let frame_number = self.last_started_frame;
        crash::with_context(("Current frame", || frame_number), || {
            let mut ctxt = super::DrawContext(DrawContext {
                target: self.display.draw(),
                gui: &self,
                now: self.start_time.elapsed(),
            });

            ctxt.0.target.clear_color(0.0, 0.0, 0.0, 1.0);
            app.draw(&mut ctxt);
            ctxt.0
                .target
                .finish()
                .expect("OpenGL drawing sequence failed");
        });
    }
}

/// The super::DrawContext implementation for the Glium backend.
pub struct DrawContext<'a> {
    target: glium::Frame,
    gui: &'a Gui,
    now: std::time::Duration,
}

glium::implement_vertex!(Vertex3, position, color_multiplier, uv);

glium::implement_vertex!(Vertex2, position, color_multiplier, uv);

pub struct Primitive3 {
    vertices: glium::VertexBuffer<Vertex3>,
    indices: glium::IndexBuffer<Index>,
}

pub struct Primitive2 {
    vertices: glium::VertexBuffer<Vertex2>,
    indices: glium::IndexBuffer<Index>,
}

impl super::Drawable for Primitive3 {
    fn draw(&mut self, ctxt: &mut super::DrawContext) {
        let t = ctxt.0.now.as_secs_f32();
        let x = (t * 1.0).sin();
        let y = (t * 1.3).sin();
        let s = (t * 2.3).sin() * 0.3 + 0.7;

        ctxt.0
            .target
            .draw(
                &self.vertices,
                &self.indices,
                &ctxt.0.gui.program_3d,
                &glium::uniform! {
                    world_transform: [
                        [s, 0.0, 0.0, 0.0],
                        [0.0, s, 0.0, 0.0],
                        [0.0, 0.0, s, 0.0],
                        [x * 0.5, y * 0.5, 0.0, 1.0]
                    ],
                },
                &Default::default(),
            )
            .unwrap();
    }
}

impl super::Drawable for Primitive2 {
    fn draw(&mut self, _ctxt: &mut super::DrawContext) {
        unimplemented!();
    }
}

impl Gui {
    pub fn make_primitive3(
        &mut self,
        vertices: &[Vertex3],
        indices: &[Index],
    ) -> Result<super::Primitive3, super::PrimitiveError> {
        // TODO Check validity of indices and length of vertices

        let vertices = glium::VertexBuffer::new(&self.display, vertices)
            .expect("Could not create a vertex buffer");

        let indices = glium::IndexBuffer::new(
            &self.display,
            glium::index::PrimitiveType::TrianglesList,
            indices,
        )
        .expect("Could not create an index buffer");

        Ok(super::Primitive3(Primitive3 { vertices, indices }))
    }

    pub fn make_primitive2(
        &mut self,
        vertices: &[Vertex2],
        indices: &[Index],
    ) -> Result<super::Primitive2, super::PrimitiveError> {
        // TODO Check validity of indices and length of vertices

        let vertices = glium::VertexBuffer::new(&self.display, vertices)
            .expect("Could not create a vertex buffer");

        let indices = glium::IndexBuffer::new(
            &self.display,
            glium::index::PrimitiveType::TrianglesList,
            indices,
        )
        .expect("Could not create an index buffer");

        Ok(super::Primitive2(Primitive2 { vertices, indices }))
    }
}
