//! GUI backend based on Glium for Linux (X11, Wayland), Windows and MacOS.
//!
//! Do not use path `gui::backend_glium` unless writing code that speicifically requires this
//! backend. Use `gui::*` wrappers, or use `gui::backend` when implementing these wrappers.

use crate::crash;

use glium::winit;
use glium::Surface; // OpenGL interface

// Shorthand
type WindowDisplay = glium::Display<glium::glutin::surface::WindowSurface>;

/// The super::Gui trait implementation for the Glium backend.
///
/// Only one object of this type should normally be instantiated, as it owns most of Glium
/// resources.
///
/// All interactions with Gui objects msut happen in main application thread.
pub struct Gui {
    program_3d: glium::Program,
    display: WindowDisplay,
    window: winit::window::Window,
    last_started_frame: u64,
    start_time: std::time::Instant,
}

/// Initializes the GUI, runs GUI main loop and shuts it down when exiting.
///
/// Due to the requirements of various underlying OS libraries, this function must be called in the
/// application main thread. It only returns after an exit is requested, and after GUI has shut
/// down.
///
/// In order to respond to user input, a super::Application object must be provided via
/// `initializer`. This object should directly or indirectly own all GUI-related resources.
///
/// A single super::Gui instance is created and later dropped by this function. It is only available
/// while this function runs, and it is only available in the thread of this function.
///
/// The exact order of events is as follows:
///   1. GUI initialization happens: window and OpenGL context are created.
///   2. `initializer` is executed. User logic may instantiate necessary resources, but blocking
///      operations should be deferred until the main loop to prevent UI freezes.
///   3. Main loop executes until an exit is requested. The object returned by `initializer`
///      receives events.
///   4. The object returned by `initializer` is dropped. User logic may release necessary
///      resources. Blocking operations should happen before GUI exits to prevent UI freezes.
///   5. GUI shuts down.
///   6. This function returns.
pub fn run<I, A>(initializer: I)
where
    I: FnOnce(&mut super::Gui) -> A,
    A: super::Application,
{
    // Perform GUI initialization
    let (mut gui, event_loop) = Gui::new();

    // Perform user logic initialization
    let mut application = initializer(&mut gui);

    // Hand control over to the GUI. This call exits only when the game is shutting down
    gui.0.run_main_loop(event_loop, &mut application);

    // drop(application): Perform user logic shutdown
    // drop(gui): Perform GUI shutdown
}

impl Gui {
    /// Performs initialization of the basic GUI resources required to implement super::Gui methods.
    ///
    /// An OS window and an OpenGL context are created.
    ///
    /// Returned values include the constructed Gui instance and an winit event loop object.
    /// The latter must be forwarded to Gui::run_main_loop as a requirement of Glium library.
    fn new() -> (super::Gui, winit::event_loop::EventLoop<()>) {
        let event_loop = winit::event_loop::EventLoopBuilder::new()
            .build()
            .expect("Could not create winit event loop");

        let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
            .with_title("Trapiron")
            .build(&event_loop);

        let program_3d = glium::Program::from_source(
            &display,
            include_str!("vertex_3d.glsl"),
            include_str!("fragment_3d.glsl"),
            None,
        )
        .expect("Could not create GLSL shared program");

        (
            super::Gui(Self {
                program_3d,
                display,
                window,
                last_started_frame: 0,
                start_time: std::time::Instant::now(),
            }),
            event_loop,
        )
    }

    /// Executes the main loop of GUI until an exit is requested.
    ///
    /// `event_loop` must be the object returned by Gui::new.
    fn run_main_loop(
        &mut self,
        event_loop: winit::event_loop::EventLoop<()>,
        app: &mut impl super::Application,
    ) {
        let _ = event_loop.run(move |event, ael| {
            crash::with_context(("Current winit (GUI) event", || &event), || {
                self.handle_event(app, &event, &ael);
            });
        });
    }

    /// Processes a single Glium event.
    ///
    /// Method arguments, other than `app`, correspond to the callback interface of
    /// winit::event_loop::run.
    fn handle_event(
        &mut self,
        app: &mut impl super::Application,
        event: &winit::event::Event<()>,
        ael: &winit::event_loop::ActiveEventLoop,
    ) {
        match &event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => ael.exit(),

                winit::event::WindowEvent::Resized(window_size) => {
                    self.display.resize((*window_size).into());
                }

                winit::event::WindowEvent::RedrawRequested => self.process_frame(app),

                _ => (),
            },

            winit::event::Event::AboutToWait => self.window.request_redraw(),

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

use super::Vertex3;
glium::implement_vertex!(Vertex3, position, color_multiplier);

pub struct Primitive3 {
    vertices: glium::VertexBuffer<super::Vertex3>,
    indices: glium::IndexBuffer<super::Index>,
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

impl Gui {
    pub fn make_primitive(
        &mut self,
        vertices: &[super::Vertex3],
        indices: &[super::Index],
    ) -> super::Primitive3 {
        let vertices = glium::VertexBuffer::new(&self.display, vertices)
            .expect("Could not create a vertex buffer");

        let indices = glium::IndexBuffer::new(
            &self.display,
            glium::index::PrimitiveType::TrianglesList,
            indices,
        )
        .expect("Could not create an index buffer");

        super::Primitive3(Primitive3 { vertices, indices })
    }
}
