
use gfx;

pub use gfx::format::DepthStencil;

use genmesh::{Vertices, Triangulate};
use genmesh::generators::{Plane, SharedVertex, IndexedPolygon};
use noise::perlin2;
use noise;
use rand::Rng;
use rand;
use rand::SeedableRng;

use na::{Vector3, Matrix4};
use na;

use rendering::colors;

gfx_defines!{
    vertex TerrainVertex {
        pos: [f32; 3] = "a_Pos",
        color: [f32; 3] = "a_Color",
        normal: [f32; 3] = "a_Norm",
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
    in vec3 v_Normal;
    
    out vec4 Target0;

    void main() {
        Target0 = vec4(v_Color, 1.0);
    }
";

const VERTEX_SHADER: &'static [u8] = b"
    #version 150 core

    in vec3 a_Pos;
    in vec3 a_Color;
    in vec3 a_Norm;

    out vec3 v_Color;
    out vec3 v_Normal;
    out vec3 v_FragPos;

    uniform Locals {
        mat4 u_Model;
        mat4 u_ViewProj;
    };

    void main() {
        v_Color = a_Color;
        v_Normal = mat3(u_Model) * a_Norm;
        v_FragPos = (u_Model * vec4(a_Pos, 1.0)).xyz;
        gl_Position = u_ViewProj * u_Model * vec4(a_Pos, 1.0);
        gl_ClipDistance[0] = 1.0;
    }
";

pub type ColorFormat = gfx::format::Srgba8;
type DepthFormat = gfx::format::DepthStencil;

fn get_terrain_color(height: f32) -> [f32; 3] {
    if height > 80.0 {
        colors::WHITE.into() // Snow
    } else if height > 70.0 {
        colors::BROWN.into() // Ground
    } else if height > -5.0 {
        colors::LIGHT_GREEN.into() // Grass
    } else {
        colors::LIGHT_BLUE.into() // Water
    }
}

fn calculate_normal(seed: &noise::PermutationTable, x: f32, y: f32) -> [f32; 3] {

    let sample_distance = 0.001;
    let s_x0 = x - sample_distance;
    let s_x1 = x + sample_distance;
    let s_y0 = y - sample_distance;
    let s_y1 = y + sample_distance;

    let dzdx = (perlin2(seed, &[s_x1, y]) - perlin2(seed, &[s_x0, y])) / (s_x1 - s_x0);
    let dzdy = (perlin2(seed, &[x, s_y1]) - perlin2(seed, &[x, s_y0])) / (s_y1 - s_y0);

    // cross gradient vectors to get normal
    let v1 = Vector3::new(1.0, 0.0, dzdx);
    let v2 = Vector3::new(0.0, 1.0, dzdy);
    let normal = v1.cross(&v2).normalize();

    return normal.into();
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
                let h = perlin2(&rand_seed, &[x, y]) * 140.0;
                TerrainVertex {
                    pos: [500.0 * x, h, 500.0 * y],
                    color: get_terrain_color(h),
                    normal: calculate_normal(&rand_seed, x, y),
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
