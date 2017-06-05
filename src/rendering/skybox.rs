
use gfx;
use gfx_core;
use gfx::{Bundle, texture};
use gfx::format::Rgba8;
use rendering::deferred::GFormat;

use image;

use std::io::Cursor;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
    }

    constant Locals {
        inv_proj: [[f32; 4]; 4] = "u_InvProj",
        view: [[f32; 4]; 4] = "u_WorldToCamera",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        cubemap: gfx::TextureSampler<[f32; 4]> = "t_Cubemap",
        locals: gfx::ConstantBuffer<Locals> = "Locals",
        out: gfx::RenderTarget<GFormat> = "Target0",
    }
}

const FRAGMENT_SHADER: &'static [u8] = b"
    #version 150 core

    uniform samplerCube t_Cubemap;

    in vec3 v_Uv;

    out vec4 Target0;

    void main() {
        Target0 = vec4(texture(t_Cubemap, v_Uv));
    }
";

const VERTEX_SHADER: &'static [u8] = b"
    #version 150 core

    uniform Locals {
        mat4 u_InvProj;
        mat4 u_WorldToCamera;
    };

    in vec2 a_Pos;

    out vec3 v_Uv;

    void main() {
        mat3 invModelView = transpose(mat3(u_WorldToCamera));
        vec3 unProjected = (u_InvProj * vec4(a_Pos, 0.0, 1.0)).xyz;
        v_Uv = invModelView * unProjected;

        gl_Position = vec4(a_Pos, 0.0, 1.0);
    }
";

pub type ColorFormat = gfx::format::Srgba8;

struct CubemapData<'a> {
    up: &'a [u8],
    down: &'a [u8],
    front: &'a [u8],
    back: &'a [u8],
    right: &'a [u8],
    left: &'a [u8],
}

impl<'a> CubemapData<'a> {
    fn as_array(self) -> [&'a [u8]; 6] {
        [self.right, self.left, self.up, self.down, self.front, self.back]
    }
}

fn load_cubemap<R, F>(factory: &mut F,
                      data: CubemapData)
                      -> Result<gfx::handle::ShaderResourceView<R, [f32; 4]>, String>
    where R: gfx::Resources,
          F: gfx::Factory<R>
{
    info!(target: "DAT205", "Loading cubemap...");
    let images = data.as_array()
        .iter()
        .map(|data| image::load(Cursor::new(data), image::JPEG).unwrap().to_rgba())
        .collect::<Vec<_>>();

    let data: [&[u8]; 6] = [&images[0], &images[1], &images[2], &images[3], &images[4], &images[5]];
    let kind = texture::Kind::Cube(images[0].dimensions().0 as u16);

    match factory.create_texture_immutable_u8::<Rgba8>(kind, &data) {
        Ok((_, view)) => {
            info!(target: "DAT205", "Successfully loaded cubemap");
            Ok(view)
        }
        Err(_) => {
            error!(target: "DAT205", "Unable to create an immutable cubemap texture");
            Err("Unable to create an immutable cubemap texture".to_owned())
        }
    }
}

impl Vertex {
    fn new(p: [f32; 2]) -> Vertex {
        Vertex { pos: p }
    }
}

pub struct Skybox<R: gfx::Resources> {
    res: Bundle<R, pipe::Data<R>>,
}

impl<R: gfx::Resources> Skybox<R> {
    pub fn new<F: gfx::Factory<R>>(factory: &mut F,
                                   main_color: gfx_core::handle::RenderTargetView<R, [f32; 4]>)
                                   -> Self {
        use gfx::traits::FactoryExt;

        info!(target: "DAT205", "Loading skybox...");

        let cubemap = load_cubemap(factory,
                                   CubemapData {
                                       up: &include_bytes!("../../assets/images/ss_up.jpg")[..],
                                       down: &include_bytes!("../../assets/images/ss_dn.jpg")[..],
                                       front: &include_bytes!("../../assets/images/ss_bk.jpg")[..],
                                       back: &include_bytes!("../../assets/images/ss_ft.jpg")[..],
                                       right: &include_bytes!("../../assets/images/ss_rt.jpg")[..],
                                       left: &include_bytes!("../../assets/images/ss_lf.jpg")[..],
                                   })
            .unwrap();

        let sampler = factory.create_sampler_linear();

        let skybox = {
            let vertex_data =
                [Vertex::new([-1.0, -1.0]), Vertex::new([3.0, -1.0]), Vertex::new([-1.0, 3.0])];
            let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&vertex_data, ());

            let pso = factory.create_pipeline_simple(VERTEX_SHADER, FRAGMENT_SHADER, pipe::new())
                .unwrap();

            let data = pipe::Data {
                vbuf: vbuf,
                cubemap: (cubemap, sampler),
                locals: factory.create_constant_buffer(1),
                out: main_color.clone(),
            };

            Bundle::new(slice, pso, data)
        };

        info!(target: "DAT205", "Done!");

        Skybox { res: skybox }
    }

    pub fn render<C: gfx::CommandBuffer<R>>(&mut self,
                                            encoder: &mut gfx::Encoder<R, C>,
                                            inv_proj: [[f32; 4]; 4],
                                            view: [[f32; 4]; 4]) {
        let locals = Locals {
            inv_proj: inv_proj,
            view: view,
        };
        encoder.update_constant_buffer(&self.res.data.locals, &locals);
        self.res.encode(encoder);
    }
}
