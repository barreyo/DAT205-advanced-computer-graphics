
use gfx;
use gfx::{Bundle, texture};
pub use gfx::format::Depth;

gfx_defines!{
    vertex FXAAVertex {
        pos_tex: [i8; 4] = "a_PosTexCoord",
    }

    constant FXAALocals {
        model: [[f32; 4]; 4] = "u_Model",
        view_proj: [[f32; 4]; 4] = "u_ViewProj",
    }

    pipeline fxaa {
        vbuf: gfx::VertexBuffer<FXAAVertex> = (),
        tex: gfx::TextureSampler<[f32; 4]> = "t_Texture",
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

const FRAGMENT_SHADER: &'static [u8] = b"
    #version 150 core

    in vec2 v_TexCoord;
    
    uniform sampler2D t_Texture;
    uniform vec3 u_InverseTextureSize;

    out vec4 Target0;

    void main() {
        vec4 tex = texture(t_Texture, v_TexCoord);
        Target0 = tex;
    }
";

const VERTEX_SHADER: &'static [u8] = b"
    #version 150 core

    in ivec4 a_PosTexCoord;

    out vec2 v_TexCoord;

    void main() {
        v_TexCoord = a_PosTexCoord.zw;
        gl_Position = vec4(a_PosTexCoord.xy, 0.0, 1.0);
    }
";

pub type ColorFormat = gfx::format::Srgba8;

pub struct FXAA<R: gfx::Resources> {
    enabled: bool,
    fxaa: Bundle<R, fxaa::Data<R>>,
}

impl<R: gfx::Resources> FXAA<R> {
    pub fn new<F: gfx::Factory<R>>(factory: &mut F,
                                   target_width: u16,
                                   target_height: u16,
                                   main_color: gfx::handle::RenderTargetView<R, ColorFormat>)
                                   -> Self {

        let sampler = factory.create_sampler(texture::SamplerInfo::new(texture::FilterMethod::Scale,
                                                      texture::WrapMode::Clamp));

        let fxaa = {
            let vertex_data = [FXAAVertex { pos_tex: [-3, -1, -1, 0] },
                               FXAAVertex { pos_tex: [1, -1, 1, 0] },
                               FXAAVertex { pos_tex: [1, 3, 1, 2] }];

            let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&vertex_data, ());

            let pso = factory.create_pipeline_simple(VERTEX_SHADER, FRAGMENT_SHADER, fxaa::new())
                .unwrap();

            let data = fxaa::Data {
                vbuf: vbuf,
                tex: (gpos.resource.clone(), sampler.clone()),
                out: main_color.clone(),
            };

            Bundle::new(slice, pso, data)
        };

        FXAA {
            enabled: true,
            fxaa: fxaa,
        }
    }
}
