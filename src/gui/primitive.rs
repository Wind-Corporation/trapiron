//! Drawing primitives and related data types.

use super::{Index, OpaqueColor, Vec2, Vec3};
use std::rc::Rc;

/// A vertex of a [`Primitive`].
#[derive(Copy, Clone)]
pub struct Vertex {
    /// The position (XYZ) of this vertex in its model's frame of reference.
    pub position: Vec3,

    /// A unit length vector (XYZ) perpendicular to the triangles defined by this vertex in its
    /// model's frame of reference.
    pub normal: Vec3,

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

////////////////////////////////////////////////////////////////////////////////////////////////////
// Mesh utility methods
//

impl Mesh {
    /// Creates a [`Mesh`] with a single parallelogram.
    ///
    /// The parallelogram spans from _origin_ to _origin + width + height_ and has sides parallel to
    /// _width_ and _height_.
    ///
    /// Texture coordinates are set up to stretch the texture over the entire parallelogram.
    /// Color multiplier is set to white.
    pub fn parallelogram_at(origin: Vec3, width: Vec3, height: Vec3) -> Self {
        let normal = width.cross(height).normalize_or(Vec3::X);

        Self {
            vertices: vec![
                Vertex {
                    position: origin + height,
                    normal,
                    color_multiplier: OpaqueColor::WHITE,
                    texture_coords: Vec2::new(0.0, 1.0),
                },
                Vertex {
                    position: origin,
                    normal,
                    color_multiplier: OpaqueColor::WHITE,
                    texture_coords: Vec2::new(0.0, 0.0),
                },
                Vertex {
                    position: origin + width + height,
                    normal,
                    color_multiplier: OpaqueColor::WHITE,
                    texture_coords: Vec2::new(1.0, 1.0),
                },
                Vertex {
                    position: origin + width,
                    normal,
                    color_multiplier: OpaqueColor::WHITE,
                    texture_coords: Vec2::new(1.0, 0.0),
                },
            ],
            indices: vec![0, 1, 2, 3, 2, 1],
        }
    }

    /// Creates a [`Mesh`] with a single parallelogram.
    ///
    /// The parallelogram spans from _(0; 0; 0)_ to _width + height_ and has sides parallel to
    /// _width_ and _height_.
    ///
    /// Texture coordinates are set up to stretch the texture over the entire parallelogram.
    /// Color multiplier is set to white.
    pub fn parallelogram(width: Vec3, height: Vec3) -> Self {
        Self::parallelogram_at(Vec3::ZERO, width, height)
    }

    /// Creates a [`Mesh`] with a single rectangle in the XY plane.
    ///
    /// The rectangle spans from _origin_ to _origin + size_ and has sides parallel to X and Y axis.
    ///
    /// Texture coordinates are set up to stretch the texture over the entire rectangle.
    /// Color multiplier is set to white.
    pub fn rectangle_at(origin: Vec3, size: Vec2) -> Self {
        Self::parallelogram_at(origin.into(), Vec3::X * size.x, Vec3::Y * size.y)
    }

    /// Creates a [`Mesh`] with a single rectangle in the XY plane.
    ///
    /// The rectangle spans from _(0; 0; 0)_ to _size_ and has sides parallel to X and Y axis.
    ///
    /// Texture coordinates are set up to stretch the texture over the entire rectangle.
    /// Color multiplier is set to white.
    pub fn rectangle(size: Vec2) -> Self {
        Self::rectangle_at(Vec3::ZERO, size)
    }
}
