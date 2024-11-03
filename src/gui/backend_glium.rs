//! GUI backend based on Glium for Linux (X11, Wayland), Windows and MacOS.
//!
//! Do not use path `gui::backend_glium` unless writing code that specifically requires this
//! backend. Use `gui::*` wrappers, or use `gui::backend` when implementing these wrappers.

use super::{Float, Vec2};
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
        let size = gui.backend.window.inner_size();
        let scale = gui.backend.window.scale_factor() as Float;
        let size = Vec2::new(size.width as Float / scale, size.height as Float / scale);

        let ctxt = DrawContext {
            target: gui.backend.display.draw(),
            _phantom: std::marker::PhantomData,
        };

        let mut ctxt = super::draw::Context {
            gui,
            backend: ctxt,
            size,
            time: std::time::Instant::now(),
            settings: Default::default(),
        };

        ctxt.backend
            .target
            .clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
        app.draw(&mut super::Dcf::new(&mut ctxt));
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

////////////////////////////////////////////////////////////////////////////////////////////////////
// Primitive assembly
//

mod primitive;
pub use primitive::Primitive;

impl Gui {
    pub fn make_primitive(&self, meshes: Vec<super::MeshWithTexture>) -> super::Primitive {
        primitive::make_primitive(&self, meshes)
    }
}

/// A texture uploaded to the GPU that might be reused for multiple [`Texture`s](super::Texture).
type Atlas = glium::texture::Texture2d;

/// The [`Texture`](super::Texture) implementation for the Glium backend.
///
/// A texture is a section of an _atlas_, which is the actual OpenGL texture that is uploaded to the
/// GPU. This allows grouping textures that are often used at the same time, saving time on
/// switching textures.
///
/// A `Texture` represents a region of `atlas` from `origin` to `origin + size`. Both `origin` and
/// `origin + size` represent in-bounds points on the atlas in normalized coordinates. Both `origin`
/// `and `size` must be positive.
pub struct Texture {
    /// The GPU texture object that contains the data of this texture.
    atlas: Rc<Atlas>,

    /// The starting point of the texture in the `atlas` in normalized coordinates.
    origin: Vec2,

    /// The span of the texture in the `atlas` in normalized coordinates.
    size: Vec2,
}

impl Texture {
    /// Creates a new Glium [`Texture`] from its raw parts.
    ///
    /// Panics if either `origin` or `origin + size` are not valid normalized texture coordinates,
    /// or if `size` is a zero vector.
    fn new(atlas: Rc<Atlas>, origin: Vec2, size: Vec2) -> Self {
        let is_valid = |v: Vec2| v.cmpge(Vec2::ZERO).all() && v.cmple(Vec2::ONE).all();

        assert!(size != Vec2::ZERO, "Cannot create Texture: size is zero");

        assert!(
            is_valid(origin),
            "Cannot create Texture: origin {} is out of bounds [0; 1]",
            origin
        );

        assert!(
            is_valid(origin + size),
            "Cannot create Texture: origin + size {} is out of bounds [0; 1]",
            origin + size
        );

        Self {
            atlas,
            origin,
            size,
        }
    }

    fn identity(&self) -> *const Self {
        &raw const *self
    }
}

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
        let texture = Texture::new(Rc::new(texture), Vec2::ZERO, Vec2::ONE);
        super::Texture(texture)
    }
}
