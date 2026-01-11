//! Hardcoded game content that is not part of the engine. For now it is only the various block
//! kinds.

pub mod block;

/// All resources required by content, such as textures and models, as well as the registry of all
/// known content.
///
/// This struct should be initialized once when game first loads, as [`Self::new`] is rather
/// expensive.
pub struct Resources {
    pub blocks: block::Kinds,
}

impl Resources {
    /// Initializes all runtime resources content needs: loads textures, generates models, etc.
    pub fn new(gui: &mut crate::gui::Gui) -> Self {
        Self {
            blocks: block::Kinds::new(gui),
        }
    }
}
