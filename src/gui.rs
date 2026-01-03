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

    /// Returns the start time of the application; more precisely, the instant this Gui was created.
    pub fn start_time(&mut self) -> std::time::Instant {
        self.start_time
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Drawing basics
//

/// The floating-point type used for graphics computations.
pub type Float = f32;

/// The integer data type used to index into vertex arrays.
///
/// The current choice of `u16` limits the vertex arrays to a length of 65535.
pub type Index = u16;

/// A Float 2D vector for graphics computations.
pub type Vec2 = glam::f32::Vec2;

/// A Float 3D vector for graphics computations.
pub type Vec3 = glam::f32::Vec3;

/// A Float 4D vector for graphics computations.
pub type Vec4 = glam::f32::Vec4;

/// A Float 3x3 matrix vector for graphics computations.
pub type Mat3 = glam::f32::Mat3;

/// A Float 4x4 matrix vector for graphics computations.
pub type Mat4 = glam::f32::Mat4;

/// A Float 3x4 matrix vector (equivalent to mat4x3 in GLSL) for graphics computations.
pub type Affine3 = glam::f32::Affine3A;

pub mod draw;
pub use draw::{Dcf, Drawable};

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
pub trait Application: draw::Drawable {}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Primitives
//

pub mod primitive;
pub use primitive::{Mesh, MeshError, MeshWithTexture, Primitive, Vertex};

impl Gui {
    /// Creates a new [3D graphics primitive](Primitive) from raw components.
    pub fn make_primitive(&mut self, meshes: Vec<MeshWithTexture>) -> Primitive {
        self.backend.make_primitive(meshes)
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
pub struct OpaqueColor(Vec3);

impl OpaqueColor {
    /// Creates a new color from an RGB triplet.
    ///
    /// Expected channel values are `[0; 1]`, but this is not a strict requirement.
    pub const fn rgb(rgb: Vec3) -> Self {
        Self(rgb)
    }

    const WHITE: OpaqueColor = OpaqueColor::rgb(Vec3::ONE);
    const BLACK: OpaqueColor = OpaqueColor::rgb(Vec3::ZERO);
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Debugging
//

pub mod debug;
