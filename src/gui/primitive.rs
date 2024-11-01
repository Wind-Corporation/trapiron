//! Drawing primitives and related data types.

use super::{OpaqueColor, Vec2, Vec3};

/// A vertex of a [`Primitive`].
#[derive(Copy, Clone)]
pub struct Vertex {
    /// The position (XYZ) of this vertex in its model's frame of reference.
    pub position: Vec3,

    /// The multiplicative color filter associated with this vertex.
    ///
    /// This is interpreted as an RGB vector with values in range `[0; 1]` for each component.
    ///
    /// If a texture is active, the color vector extracted from the texture is multiplied
    /// component-wise with this vector. If no texture is bound, this color is used without
    /// modification instead. The filter is interpolated linearly between vertices.
    pub color_multiplier: OpaqueColor,

    /// The coordinates in texture space associated with this vertex (the UV-mapping of the vertex).
    pub texture_coords: Vec2,
}

/// The simplest 3D object that can be drawn to the screen directly.
///
/// A Primitive is a collection of vertices, connected into triangles according to an vertex index
/// list, that has a set of textures associated with it.
pub struct Primitive(pub(super) super::backend::Primitive);

impl super::Drawable for Primitive {
    fn draw(&mut self, dcf: &mut super::Dcf) {
        self.0.draw(dcf);
    }
}
