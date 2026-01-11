//! Block kinds and abstractions.
//!
//! There are three main types associated with blocks: [block instances](Instance), block
//! [kinds](KindInstance) and block [views](ViewInstance):
//! - Block kinds: one per type of block, such as "stone" or "sand"; also owns GUI resources.
//!   - [`KindInstance`]\: trait implemented by every block kind
//!   - [`Kinds`]\: a registry with one of every known kind
//!   - [`KindRef`]\: a reference to a kind; essentially just `&dyn KindInstance`
//! - Block views: look-and-feel model of a specific block with its animation state, etc.
//!   - [`ViewInstance`]\: trait implemented by every block view
//!   - [`View`]\: an enum of every possible `ViewInstance` for dispatch.
//! - Block instances: the state of a specific block in the level.
//!   - [`Instance`]\: trait implemented by every block state type.
//!   - [`Block`]\: an enum of every possible `Instance` for dispatch.

mod basic;

use std::rc::Rc;

use crate::gui::{Drawable, Gui, Primitive, Texture};
use basic::*;

/// Serialized representation of a single block. Kind identifier is not included.
pub struct Serialized;

/// A single type of block, such as "stone" or "sand".
///
/// Responsible for initialization and ownership of resources used by blocks of this kind, such as
/// textures or models.
pub trait KindInstance {
    /// Initialize assets used by this kind.
    ///
    /// This is expected to be called once when application starts.
    fn new(gui: &mut Gui) -> Self;
}

/// The GUI representation of a specific state of a single block instance.
///
/// For now, this only includes its graphics; audio and other kinds of GUI feedback will be included
/// in the future.
///
/// A new view is generated each time the block state updates, so the state of the block can be
/// baked into the ViewInstance, especially since it doesn't have access to the block state from the
/// world when it is rendered. This is to facilitate their use outside of world rendering contexts,
/// such as in UI elements.
pub trait ViewInstance: Drawable {}

/// The state of a single instance of a block in a world.
///
/// This should be empty unless the block contains some modifiable properties.
pub trait Instance {
    /// The kind of this block.
    type Kind: KindInstance;

    /// The type used for the view of this block.
    type View: ViewInstance;

    /// Obtain a view for this block state.
    ///
    /// This method should execute quickly to avoid lag. Cache all expensive computation in `Kind`;
    /// for many block kinds, the entire view can be pre-initialized and shared via [`Rc`].
    fn view(&self, rsrc: &Self::Kind) -> Self::View;

    /// Deserialize `Self`.
    fn from(data: &Serialized) -> Self;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Texture group of bundled block textures.
const TEXTURES: crate::gui::TextureGroup = crate::gui::TextureGroup {};

/// A block view that renders an opaque cube.
///
/// This view is static, and so it should be pre-initialized in [`KindInstance`].
#[derive(Clone)]
pub struct FullCube(Rc<Primitive>);

impl FullCube {
    /// Create a `FullCube` view with a given texture.
    fn new(texture: &Rc<Texture>, gui: &mut Gui) -> Self {
        let mwt = crate::gui::Mesh::tmp_ppp(
            crate::gui::Vec3::splat(-0.5),
            crate::gui::Vec3::X,
            crate::gui::Vec3::Y,
            crate::gui::Vec3::Z,
            &texture,
        );

        Self(Rc::new(gui.make_primitive(mwt)))
    }
}

impl ViewInstance for FullCube {}
impl Drawable for FullCube {
    fn draw(&mut self, dcf: &mut crate::gui::Dcf) {
        self.0.draw(dcf);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Creates a registry of all known block kinds and generates boilerplate types and methods.
///
/// ## Usage
/// ```
/// // At module level
///
/// all_blocks! {
///     // snake_case_id: BlockInstanceType
///     stone: Stone,
///     sand: Sand,
/// }
/// ```
macro_rules! all_blocks {
    { $($snake_case:ident: $title_case:ident),+ $(,)? } => {
        /// A reference to some [`KindInstance`] value from the [registry](Kinds).
        pub enum KindRef<'a> {
            $(
                $title_case(&'a <$title_case as Instance>::Kind),
            )*
        }

        impl<'a> KindRef<'a> {
            /// Create a block instance with given state.
            pub fn instantiate(&self, data: &Serialized) -> Block {
                match self {
                    $(
                        KindRef::$title_case(_) => {
                            Block::$title_case(<$title_case as Instance>::from(data))
                        }
                    )*
                }
            }
        }

        /// A GUI representation of a specific state of a specific block; a [`ViewInstance`] value.
        pub enum View {
            $(
                $title_case(<$title_case as Instance>::View),
            )*
        }

        impl Drawable for View {
            fn draw(&mut self, dcf: &mut crate::gui::Dcf) {
                match self {
                    $(
                        View::$title_case(instance) => instance.draw(dcf),
                    )*
                }
            }
        }

        /// A single block instance; an [`Instance`] value.
        pub enum Block {
            $(
                $title_case($title_case),
            )*
        }

        impl Block {
            /// Obtain a view for this block state.
            ///
            /// The view will have the state of this block baked into it.
            pub fn view(&self, types: &Kinds) -> View {
                match self {
                    $(
                        Block::$title_case(instance) => {
                            View::$title_case(instance.view(&types.$snake_case))
                        }
                    )*
                }
            }
        }

        /// All resources required by blocks, such as textures and models, as well as the registry
        /// of all known block kinds.
        ///
        /// This struct should be initialized once when game first loads, as [`Self::new`] is rather
        /// expensive.
        pub struct Kinds {
            $(
                $snake_case: <$title_case as Instance>::Kind,
            )*
        }

        impl Kinds {
            /// Initializes all runtime resources content needs: loads textures, generates models,
            /// etc.
            pub fn new(gui: &mut crate::gui::Gui) -> Self {
                Self {
                    $(
                        $snake_case: <$title_case as Instance>::Kind::new(gui),
                    )*
                }
            }

            /// Find a block kind by its name.
            pub fn get<'a> (&'a self, name: &str) -> Option<KindRef<'a>> {
                match name {
                    $(
                        stringify!($snake_case) => Some(KindRef::$title_case(&self.$snake_case)),
                    )*
                    _ => None,
                }
            }
        }
    };
}

impl Default for Block {
    fn default() -> Self {
        Self::Air(Air)
    }
}

all_blocks! {
    air: Air,
    pusher: Pusher,
    sand: Sand,
    stone: Stone,
}
