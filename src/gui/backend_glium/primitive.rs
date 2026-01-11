use crate::gui::{Dcf, Float, Index, MeshWithTexture};
use glium::Surface;
use std::{ops::Deref, rc::Rc};

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [Float; 3],
    normal: [Float; 3],
    color_multiplier: [Float; 3],
    texture_coords: [Float; 2],
}

glium::implement_vertex!(Vertex, position, normal, color_multiplier, texture_coords);

pub struct Primitive {
    vertices: glium::VertexBuffer<Vertex>,
    indices: glium::IndexBuffer<Index>,
    parts: Vec<Part>,
}

struct Part {
    start: usize,
    end: usize,
    texture: Rc<crate::gui::Texture>,
}

impl Primitive {
    pub fn draw(&self, dcf: &mut Dcf) {
        let screen_transform = dcf.settings().screen_transform.to_cols_array_2d();
        let view_transform = dcf.settings().view_transform.to_cols_array_2d();
        let world_transform = dcf.state().world_transform.to_cols_array_2d();
        let color_multiplier_global = dcf.state().color_multiplier.0.to_array();
        let ambient_color = dcf.settings().lighting.ambient_color.0.to_array();
        let diffuse_color = dcf.settings().lighting.diffuse_color.0.to_array();
        let diffuse_direction = dcf.settings().lighting.diffuse_direction.to_array();

        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            ..Default::default()
        };

        let target = &mut dcf.ctxt.backend.target;
        let program = &dcf.ctxt.gui.backend.program;

        for part in &self.parts {
            let sampler = part
                .texture
                .0
                .atlas
                .sampled()
                .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest);

            let uniforms = glium::uniform! {
                screen_transform: screen_transform,
                view_transform: view_transform,
                world_transform: world_transform,
                ambient_color: ambient_color,
                diffuse_color: diffuse_color,
                diffuse_direction: diffuse_direction,
                color_multiplier_global: color_multiplier_global,
                tex: sampler,
            };

            target
                .draw(
                    &self.vertices,
                    self.indices.slice(part.start..part.end).unwrap(),
                    program,
                    &uniforms,
                    &params,
                )
                .unwrap();
        }
    }
}

/// The raw components for a [`Primitive`] that do not require interaction with the GPU.
pub struct PrimitiveOnCpu {
    vertices: Vec<Vertex>,
    indices: Vec<Index>,
    parts: Vec<Part>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Assembly
//

/// Assembles the collection of meshes into a [half-finished `Primitive`](PrimitiveOnCpu).
///
/// Does not interact with the GPU, so this function does not need to run in the graphics thread.
/// (In practice it still does because `MeshWithTexture` and `PrimitiveOnCpu` is not `Send`)
pub fn assemble(meshes: Vec<MeshWithTexture>) -> PrimitiveOnCpu {
    // TODO reduce copying of Rc

    let meshes = SortedMeshes::new(meshes);

    let vertex_count = meshes.iter().map(|m| m.geometry.vertices().len()).sum();
    let index_count = meshes.iter().map(|m| m.geometry.indices().len()).sum();
    let part_count = meshes.groups().count();

    let mut result = PrimitiveOnCpu {
        vertices: Vec::with_capacity(vertex_count),
        indices: Vec::with_capacity(index_count),
        parts: Vec::with_capacity(part_count),
    };

    for (texture, group) in meshes.groups() {
        let start = result.indices.len();

        for mesh in group {
            append_mesh(&mut result.vertices, &mut result.indices, mesh);
        }

        result.parts.push(Part {
            start,
            end: result.indices.len(),
            texture,
        });
    }

    result
}

/// An immutable collection of [`MeshWithTexture`s](MeshWithTexture) sorted by texture identity.
struct SortedMeshes(Vec<MeshWithTexture>);

struct Groups<'a> {
    meshes: &'a SortedMeshes,
    pos: usize,
}

impl Deref for SortedMeshes {
    type Target = Vec<MeshWithTexture>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SortedMeshes {
    /// Sorts `meshes` by texture identity and wraps it in a `SortedMeshes`.
    pub fn new(mut meshes: Vec<MeshWithTexture>) -> Self {
        // Optimization: sort Mwt's into an order that allows merging meshes with the same textures
        // and reduces atlas changes
        meshes.sort_unstable_by_key(|mesh| {
            (
                &raw const *mesh.texture.0.atlas, // atlas identity
                mesh.texture.0.identity(),
            )
        });

        Self(meshes)
    }

    /// Returns an iterator that visits each groups of [`MeshWithTexture`s](MeshWithTexture) that
    /// share the same texture exactly once.
    pub fn groups(&self) -> Groups<'_> {
        Groups {
            meshes: &self,
            pos: 0,
        }
    }
}

impl<'a> Iterator for Groups<'a> {
    type Item = (Rc<crate::gui::Texture>, &'a [MeshWithTexture]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.meshes.len() {
            return None;
        }

        let tid_of = |index: usize| self.meshes[index].texture.0.identity();

        let start = self.pos;
        while self.pos < self.meshes.len() && tid_of(start) == tid_of(self.pos) {
            self.pos += 1;
        }

        Some((
            self.meshes[start].texture.clone(),
            &self.meshes[start..self.pos],
        ))
    }
}

/// Appends the vertex and index data from `data` into `vertices` and `indices`.
///
/// Performs the conversion from [`gui::Vertex`](crate::gui::Vertex) to [`Vertex`] and adjusts
/// indices appropriately.
fn append_mesh(vertices: &mut Vec<Vertex>, indices: &mut Vec<Index>, data: &MeshWithTexture) {
    let mesh = &data.geometry;
    let index_offset = vertices.len() as Index;

    // TODO check that (vertices.len() + geometry.vertices < Index::MAX)
    vertices.extend(
        mesh.vertices()
            .iter()
            .map(|v| convert_vertex(v, &data.texture.0)),
    );
    indices.extend(mesh.indices().iter().map(|i| i + index_offset));
}

/// Converts a [`gui::Vertex`](crate::gui::Vertex) to a [`Vertex`].
///
/// `texture` is required to bake the texture coordinates properly.
fn convert_vertex(input: &crate::gui::Vertex, texture: &super::Texture) -> Vertex {
    let texture_coords = [
        input.texture_coords.x * texture.size.x + texture.origin.x,
        input.texture_coords.y * texture.size.y + texture.origin.y,
    ];

    Vertex {
        position: input.position.to_array(),
        normal: input.normal.to_array(),
        color_multiplier: input.color_multiplier.0.to_array(),
        texture_coords,
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Upload
//

impl PrimitiveOnCpu {
    /// Uploads this primitive to the GPU.
    fn upload(self, gui: &crate::gui::backend_glium::Gui) -> Primitive {
        let vertices = glium::VertexBuffer::immutable(&gui.display, &self.vertices)
            .expect("Could not create a vertex buffer");

        let indices = glium::IndexBuffer::new(
            &gui.display,
            glium::index::PrimitiveType::TrianglesList,
            &self.indices,
        )
        .expect("Could not create an index buffer");

        Primitive {
            vertices,
            indices,
            parts: self.parts,
        }
    }
}

pub fn make_primitive(
    gui: &crate::gui::backend_glium::Gui,
    meshes: Vec<MeshWithTexture>,
) -> crate::gui::Primitive {
    crate::gui::Primitive(assemble(meshes).upload(gui))
}
