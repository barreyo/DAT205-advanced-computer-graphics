
use gfx;

pub use gfx::format::DepthStencil;

use genmesh::{Vertices, Triangulate};
use genmesh::generators::{Plane, SharedVertex, IndexedPolygon};
use noise::perlin2;
use rand::Rng;
use rand;

use na::Matrix4;

gfx_defines!{
    vertex TerrainVertex {
        pos: [f32; 3] = "a_Pos",
        color: [f32; 3] = "a_Color",
    }

    constant TerrainLocals {
        model: [[f32; 4]; 4] = "u_Model",
        view_proj: [[f32; 4]; 4] = "u_ViewProj",
    }

    pipeline terrain {
        vbuf:      gfx::VertexBuffer<TerrainVertex> = (),
        locals:    gfx::ConstantBuffer<TerrainLocals> = "Locals",
        model:     gfx::Global<[[f32; 4]; 4]> = "u_Model",
        view_proj: gfx::Global<[[f32; 4]; 4]> = "u_ViewProj",
        out_color: gfx::RenderTarget<ColorFormat> = "Target0",
        out_depth: gfx::DepthTarget<DepthFormat> =
            gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

const FRAGMENT_SHADER: &'static [u8] = b"
    #version 150 core

    in vec3 v_Color;
    out vec4 Target0;

    void main() {
        Target0 = vec4(v_Color, 1.0);
    }
";

const VERTEX_SHADER: &'static [u8] = b"
    #version 150 core

    in vec3 a_Pos;
    in vec3 a_Color;
    out vec3 v_Color;

    uniform Locals {
        mat4 u_Model;
        mat4 u_ViewProj;
    };

    void main() {
        v_Color = a_Color;
        gl_Position = u_ViewProj * u_Model * vec4(a_Pos, 1.0);
        gl_ClipDistance[0] = 1.0;
    }
";

pub type ColorFormat = gfx::format::Srgba8;
type DepthFormat = gfx::format::DepthStencil;

fn get_terrain_color(height: f32) -> [f32; 3] {
    if height > 8.0 {
        [0.9, 0.9, 0.9] // white
    } else if height > 0.0 {
        [0.7, 0.7, 0.7] // gray
    } else if height > -5.0 {
        [0.2, 0.7, 0.2] // green
    } else {
        [0.2, 0.2, 0.7] // blue
    }
}

pub struct Terrain<R: gfx::Resources> {
    pso: gfx::PipelineState<R, terrain::Meta>,
    data: terrain::Data<R>,
    slice: gfx::Slice<R>,
}

impl<R: gfx::Resources> Terrain<R> {
    pub fn new<F: gfx::Factory<R>>(size: usize,
                                   factory: &mut F,
                                   main_color: gfx::handle::RenderTargetView<R, ColorFormat>,
                                   main_depth: gfx::handle::DepthStencilView<R, DepthFormat>)
                                   -> Self {
        use gfx::traits::FactoryExt;

        let pso = factory.create_pipeline_simple(VERTEX_SHADER, FRAGMENT_SHADER, terrain::new())
            .unwrap();

        let rand_seed = rand::thread_rng().gen();
        let plane = Plane::subdivide(size, size);

        let vertex_data: Vec<TerrainVertex> = plane.shared_vertex_iter()
            .map(|(x, y)| {
                let h = perlin2(&rand_seed, &[x, y]) * 8.0;
                TerrainVertex {
                    pos: [100.0 * x, 100.0 * y, h],
                    color: get_terrain_color(h),
                }
            })
            .collect();

        let index_data: Vec<u32> = plane.indexed_polygon_iter()
            .triangulate()
            .vertices()
            .map(|i| i as u32)
            .collect();

        let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&vertex_data, &index_data[..]);

        Terrain {
            pso: pso,
            data: terrain::Data {
                vbuf: vbuf,
                locals: factory.create_constant_buffer(1),
                model: Matrix4::identity().into(),
                view_proj: Matrix4::identity().into(),
                out_color: main_color.clone(),
                out_depth: main_depth.clone(),
            },
            slice: slice,
        }
    }

    pub fn render<C: gfx::CommandBuffer<R>>(&mut self,
                                            encoder: &mut gfx::Encoder<R, C>,
                                            view_proj: [[f32; 4]; 4]) {

        let locals = TerrainLocals {
            model: self.data.model,
            view_proj: view_proj,
        };

        encoder.update_buffer(&self.data.locals, &[locals], 0).unwrap();
        encoder.draw(&self.slice, &self.pso, &self.data);
    }
}
