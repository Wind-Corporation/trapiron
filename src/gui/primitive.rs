//! Drawing primitives and related data types.

use super::{Float, Index, OpaqueColor, Vec2, Vec3};
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

    /// The `indices` array is too large.
    TooManyIndices {
        /// The maximum allowed size of the `indices` array.
        max_indices: usize,
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

        let max_indices = 6 * max_vertices;
        if indices.len() > max_indices {
            return Err(MeshError::TooManyIndices { max_indices });
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
// Simple Mesh builders
//

/// A template for a [`Mesh`] with a single parallelogram.
///
/// The parallelogram may be located anywhere and it can have any arbitrary orientation. The color
/// multiplier shared by the vertices is configurable. The texture is mapped to stretch to fill the
/// entire shape.
///
/// See [`Mesh::square`], [`Mesh::rectangle`] and [`Mesh::parallelogram`] to obtain a builder. Use
/// [`Self::build`] or [`Self::bind`] to produce a [`Mesh`] or a [`MeshWithTexture`] respectively.
#[derive(Clone)]
pub struct ParallelogramBuilder {
    origin: Vec3,
    width: Vec3,
    height: Vec3,
    color_multiplier: OpaqueColor,
}

impl ParallelogramBuilder {
    /// Bakes this parallelogram into a [`Mesh`].
    ///
    /// See also [`Self::bind`].
    pub fn build(&self) -> Mesh {
        let normal = self.width.cross(self.height).normalize_or(Vec3::X);

        Mesh {
            vertices: vec![
                Vertex {
                    position: self.origin + self.height,
                    normal,
                    color_multiplier: self.color_multiplier,
                    texture_coords: Vec2::new(0.0, 1.0),
                },
                Vertex {
                    position: self.origin,
                    normal,
                    color_multiplier: self.color_multiplier,
                    texture_coords: Vec2::new(0.0, 0.0),
                },
                Vertex {
                    position: self.origin + self.width + self.height,
                    normal,
                    color_multiplier: self.color_multiplier,
                    texture_coords: Vec2::new(1.0, 1.0),
                },
                Vertex {
                    position: self.origin + self.width,
                    normal,
                    color_multiplier: self.color_multiplier,
                    texture_coords: Vec2::new(1.0, 0.0),
                },
            ],
            indices: vec![0, 1, 2, 3, 2, 1],
        }
    }

    /// Bakes this parallelogram into a [`MeshWithTexture`], applying the provided texture.
    ///
    /// Shorthand for `builder.build().bind(texture)`.
    ///
    /// See also [`Self::build`].
    pub fn bind(&self, texture: Rc<super::Texture>) -> MeshWithTexture {
        self.build().bind(texture)
    }

    /// Moves the origin (one of the corners) to the provided location.
    pub fn at(mut self, origin: Vec3) -> Self {
        self.origin = origin;
        self
    }

    /// Centers the parallelogram at (0; 0; 0).
    pub fn centered(self) -> Self {
        let center = (self.width + self.height) / 2.0;
        self.at(-center)
    }

    /// Applies a color multiplier, compounding with previous color multiplier edits.
    ///
    /// See [`Vertex::color_multiplier`] for more details.
    pub fn apply_color_mult(mut self, color_multiplier: OpaqueColor) -> Self {
        self.color_multiplier = OpaqueColor(self.color_multiplier.0 * color_multiplier.0);
        self
    }
}

impl Mesh {
    /// Begins building a `Mesh` with a single square.
    ///
    /// The square is aligned with X and Y axis and spans (0; 0; 0) to (`size`; `size`; 0).
    /// Its color mulitplier is white.
    ///
    /// See [`ParallelogramBuilder`] for more details.
    pub fn square(size: Float) -> ParallelogramBuilder {
        ParallelogramBuilder {
            origin: Vec3::ZERO,
            width: Vec3::X * size,
            height: Vec3::Y * size,
            color_multiplier: OpaqueColor::WHITE,
        }
    }

    /// Begins building a `Mesh` with a single rectangle.
    ///
    /// The rectangle is aligned with X and Y axis and spans (0; 0; 0) to (`size`; 0). Its color
    /// mulitplier is white.
    ///
    /// See [`ParallelogramBuilder`] for more details.
    pub fn rectangle(size: Vec2) -> ParallelogramBuilder {
        ParallelogramBuilder {
            width: Vec3::X * size.x,
            height: Vec3::Y * size.y,
            ..Mesh::square(1.0)
        }
    }

    /// Begins building a `Mesh` with a single parallelogram.
    ///
    /// The parallelogram has sides (0; 0; 0) to `width` and (0; 0; 0) to `height`. Its color
    /// mulitplier is white.
    ///
    /// See [`ParallelogramBuilder`] for more details.
    pub fn parallelogram(width: Vec3, height: Vec3) -> ParallelogramBuilder {
        ParallelogramBuilder {
            width,
            height,
            ..Mesh::square(1.0)
        }
    }

    pub fn tmp_ppp(
        origin: Vec3,
        width: Vec3,
        height: Vec3,
        depth: Vec3,
        texture: &Rc<super::Texture>,
    ) -> Vec<MeshWithTexture> {
        let min_origin = origin;
        let max_origin = origin + width + height + depth;
        vec![
            Self::parallelogram(height, width)
                .at(min_origin)
                .bind(texture.clone()),
            Self::parallelogram(width, depth)
                .at(min_origin)
                .bind(texture.clone()),
            Self::parallelogram(depth, height)
                .at(min_origin)
                .bind(texture.clone()),
            Self::parallelogram(-width, -height)
                .at(max_origin)
                .bind(texture.clone()),
            Self::parallelogram(-depth, -width)
                .at(max_origin)
                .bind(texture.clone()),
            Self::parallelogram(-height, -depth)
                .at(max_origin)
                .bind(texture.clone()),
        ]
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// OBJ input
//

impl Mesh {
    /// Loads a mesh from a triangulated Wavefront OBJ file.
    ///
    /// The positions and normals of vertices are used verbatim; only first two texture coordinates
    /// are used; color multiplier is set to white.
    ///
    /// See doc/obj.md for more details.
    pub fn load_obj<T>(input: T) -> Result<Self, obj::ObjError>
    where
        T: std::io::BufRead,
    {
        let data = obj::load_obj::<obj::TexturedVertex, T, Index>(input)?;

        Ok(Self {
            vertices: data
                .vertices
                .into_iter()
                .map(|v| Vertex {
                    position: v.position.into(),
                    normal: v.normal.into(),
                    color_multiplier: OpaqueColor::WHITE,
                    texture_coords: Vec2::new(v.texture[0], v.texture[1]),
                })
                .collect(),
            indices: data.indices,
        })
    }
}
