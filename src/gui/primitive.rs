//! Drawing primitives and related data types.

use super::{Index, OpaqueColor, Vec2, Vec3};
use std::rc::Rc;

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

/// A group of vertices that form a triangle mesh.
///
/// Used to create [`Primitive`s](Primitive) via [`MeshWithTexture`].
#[derive(Clone)]
pub struct Mesh {
    /// The unique vertices of this mesh.
    vertices: Vec<Vertex>,

    /// The order in which vertices from [`Mesh::vertices`] should be used to assemble triangles.
    ///
    /// Each element is a valid index into the `vertices` array.
    indices: Vec<Index>,
}

/// A group of vertices that form a triangle mesh, with a texture applied to the entire geometry.
///
/// Note that only a single texture may be bound with a `MeshWithTexture`. Use multiple to assemble
/// a [`Primitive`] with more than one texture.
///
/// Used to create [`Primitive`s](Primitive). See [`Mesh`].
#[derive(Clone)]
pub struct MeshWithTexture {
    /// The vertex data and order for the mesh of triangles.
    pub geometry: Mesh,

    /// A reference to the texture used to draw the geometry.
    pub texture: Rc<super::Texture>,
}

/// An error that might occur when creating a [`Mesh`].
#[derive(Debug)]
pub enum MeshError {
    /// The `indices` array contains an index that is out of bounds for the `vertices` array.
    IndexOutOfBounds {
        /// The offset into the `indices` array at which some invalid element was found.
        index_of_index: usize,
    },

    /// The `vertices` array is too large.
    TooManyVertices {
        /// The maximum allowed size of the `vertices` array.
        max_vertices: usize,
    },
}

impl Mesh {
    /// Creates a [`Mesh`] out of an existing vertex and index arrays.
    ///
    /// `vertices` is the data for unique vertices in arbitrary order. The elements of `indices` are
    /// used in enumeration order to construct triangles, three at a time, intepreted as offsets into
    /// `vertices`.
    pub fn new(vertices: Vec<Vertex>, indices: Vec<Index>) -> Result<Self, MeshError> {
        for (index_of_index, index) in indices.iter().enumerate() {
            if *index >= vertices.len() as Index {
                return Err(MeshError::IndexOutOfBounds { index_of_index });
            }
        }

        let max_vertices = Index::MAX as usize;
        if vertices.len() > max_vertices {
            return Err(MeshError::TooManyVertices { max_vertices });
        }

        Ok(Self { vertices, indices })
    }

    /// Returns the unique vertices of this mesh.
    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    /// Returns the order in which vertices from [`Mesh::vertices`] should be used to assemble
    /// triangles.
    ///
    /// Each element is a valid index into the `vertices` array.
    pub fn indices(&self) -> &[Index] {
        &self.indices
    }

    /// Pairs a texture reference to this geometry.
    pub fn bind(self, texture: Rc<super::Texture>) -> MeshWithTexture {
        MeshWithTexture {
            geometry: self,
            texture,
        }
    }
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
