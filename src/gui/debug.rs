//! Several aids and utilities for debugging graphics.

const DEBUG_TEXTURES: super::TextureGroup = super::TextureGroup {};

/// A graphics debugging aid; a primitive that displays the X, Y and Z axes.
///
/// The X axis is red, Y axis is green, Z axis is blue, similar to Blender UI.
///
/// Inside the primitive is a magenta box. It is normally hidden, but if the coordinate space is
/// inverted, it becomes visible.
pub fn axes(gui: &mut super::Gui) -> super::Primitive {
    let texture = gui.texture(&DEBUG_TEXTURES.id("axes"));
    let mesh = super::asset::load_mesh("axes");
    gui.make_primitive(vec![mesh.bind(texture)])
}
