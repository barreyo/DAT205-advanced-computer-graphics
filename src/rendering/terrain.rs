
use gfx;

pub use gfx::format::{DepthStencil};

use genmesh::{Vertices, Triangulate};
use genmesh::generators::{Plane, SharedVertex, IndexedPolygon};
use noise::{Seed, perlin2};
use rand::Rng;

pub type ColorFormat = gfx::format::Srgba8;
type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 3] = "a_Pos",
        color: [f32; 3] = "a_Color",
    }

    constant Locals {
        model: [[f32; 4]; 4] = "u_Model",
        view: [[f32; 4]; 4] = "u_View",
        proj: [[f32; 4]; 4] = "u_Proj",
    }

    pipeline pipe {
        vbuf:      gfx::VertexBuffer<Vertex> = (),
        locals:    gfx::ConstantBuffer<Locals> = "Locals",
        model:     gfx::Global<[[f32; 4]; 4]> = "u_Model",
        view:      gfx::Global<[[f32; 4]; 4]> = "u_View",
        proj:      gfx::Global<[[f32; 4]; 4]> = "u_Proj",
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
        mat4 u_View;
        mat4 u_Proj;
    };

    void main() {
        v_Color = a_Color;
        gl_Position = u_Proj * u_View * u_Model * vec4(a_Pos, 1.0);
        gl_ClipDistance[0] = 1.0;
    }
";

fn get_terrain_color(height: f32) -> [f32; 3] {
    if height > 0.5 {
        [0.5, 0.2, 0.9]
    } else {
        [0.8, 0.7, 0.2]
    }
}

struct Terrain<R: gfx::Resources> {
    pso: gfx::PipelineState<R, pipe::Meta>,
    data: pipe::Data<R>,
    slice: gfx::Slice<R>,
}

impl<R: gfx::Resources> Terrain<R> {
    fn new<F: gfx::Factory<R>>(factory: &mut F,
                               backend: gfx_app::shade::Backend,
                               window_targets: gfx_app::WindowTargets<R>)
                               -> Self {
        use gfx::traits::FactoryExt;

        let pso = factory.create_pipeline_simple(VERTEX_SHADER, FRAGMENT_SHADER, pipe::new())
            .unwrap();

        let rand_seed = rand::thread_rng().gen();
        let seed = Seed::new(rand_seed);
        let plane = Plane::subdivide(256, 256);
        let vertex_data: Vec<Vertex> = plane.shared_vertex_iter()
            .map(|(x, y)| {
                let h = perlin2(&seed, &[x, y]) * 32.0;
                Vertex {
                    pos: [25.0 * x, 25.0 * y, h],
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
            data: pipe::Data {
                vbuf: vbuf,
                locals: factory.create_constant_buffer(1),
                model: Matrix4::identity().into(),
                view: Matrix4::identity().into(),
                out_color: window_targets.color,
                out_depth: window_targets.depth,
            },
            slice: slice,
        }
    }

    fn render<C: gfx::CommandBuffer<R>>(&mut self, encoder: &mut gfx::Encoder<R, C>) {

        let locals = Locals {
            model: self.data.model,
            view: self.data.view,
            proj: self.data.proj,
        };

        encoder.update_buffer(&self.data.locals, &[locals], 0).unwrap();
        encoder.draw(&self.slice, &self.pso, &self.data);
    }
}
