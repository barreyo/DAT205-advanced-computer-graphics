
use conrod;
use conrod::Color;
use conrod::render::Text;
use conrod::text::rt;
use conrod::text;
use conrod::Point;
use gfx;

use genmesh::{Vertices, Triangulate};
use genmesh::generators::{Plane, SharedVertex, IndexedPolygon};

use gfx::texture;
use gfx::traits::FactoryExt;

pub type ColorFormat = gfx::format::Srgba8;
type SurfaceFormat = gfx::format::R8_G8_B8_A8;
type FullFormat = (SurfaceFormat, gfx::format::Unorm);

const FRAGMENT_SHADER: &'static [u8] = b"
        #version 140
        
        in vec4 v_Color;
        out vec4 f_Color;
        
        void main() {
            f_Color = v_Color;
        }
    ";

const VERTEX_SHADER: &'static [u8] = b"
        #version 140

        in vec2 a_Pos;
        in vec4 a_Color;
        
        out vec4 v_Color;
        
        void main() {
            v_Color = a_Color;
            gl_Position = vec4(a_Pos, 0.0, 1.0);
        }
    ";

// Vertex and pipeline declarations
gfx_defines! {
    vertex PrimitiveVertex {
        pos: [f32; 2] = "a_Pos",
        color: [f32; 4] = "a_Color",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<PrimitiveVertex> = (),
        out: gfx::RenderTarget<ColorFormat> = "f_Color",
    }
}

// Convenience constructor
impl PrimitiveVertex {
    fn new(pos: [f32; 2], color: [f32; 4]) -> PrimitiveVertex {
        PrimitiveVertex {
            pos: pos,
            color: color,
        }
    }
}

pub struct PrimitiveRender<R: gfx::Resources> {
    pso: gfx::PipelineState<R, pipe::Meta>,
    data: pipe::Data<R>,
}

impl<R: gfx::Resources> PrimitiveRender<R> {
    pub fn new<F: gfx::Factory<R>>(factory: &mut F,
                                   main_color: gfx::handle::RenderTargetView<R, ColorFormat>)
                                   -> Self {

        let pso = factory.create_pipeline_simple(VERTEX_SHADER, FRAGMENT_SHADER, pipe::new())
            .unwrap();

        PrimitiveRender {
            pso: pso,
            data: pipe::Data {
                vbuf: [],
                out: main_color.clone(),
            },
        }
    }

    fn render_rectangle<F: gfx::Factory<R>>(&self,
                                            factory: &mut F,
                                            color: Color,
                                            width: f32,
                                            height: f32,
                                            pos: [f32; 2]) {

        let plane = Plane::new();
        let col = color.to_fsa();

        // TODO: Support rounded corners
        let vertex_data: Vec<PrimitiveVertex> = plane.shared_vertex_iter()
            .map(|(x, y)| {
                PrimitiveVertex {
                    pos: [x, y],
                    color: col.clone(),
                }
            })
            .collect();

        let index_data: Vec<u32> = plane.indexed_polygon_iter()
            .triangulate()
            .vertices()
            .map(|i| i as u32)
            .collect();

        let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&vertex_data, &index_data[..]);
    }

    fn render_lines(&self, thickness: f32, points: [Point]) {}
}
