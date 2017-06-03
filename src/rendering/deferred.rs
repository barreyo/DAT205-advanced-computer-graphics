
use gfx;
use gfx::{Bundle, texture};
pub use gfx::format::Depth;
use na::Vector3;
use na::Point3;
use noise::perlin2;
use noise;

use genmesh::{Vertices, Triangulate};
use genmesh::generators::SharedVertex;
use genmesh::generators::IndexedPolygon;
use genmesh::generators::SphereUV;

gfx_defines!{
    constant LightInfo {
        pos: [f32; 4] = "pos",
    }

    vertex BlitVertex {
        pos_tex: [i8; 4] = "a_PosTexCoord",
    }

    constant LightLocals {
        cam_pos_and_radius: [f32; 4] = "u_CamPosAndRadius",
    }

    vertex SphereVertex {
        pos: [i8; 4] = "a_Pos",
    }

    constant SphereLocals {
        transform: [[f32; 4]; 4] = "u_Transform",
        radius: f32 = "u_Radius",
    }

    pipeline emitter {
        vbuf: gfx::VertexBuffer<SphereVertex> = (),
        locals: gfx::ConstantBuffer<SphereLocals> = "SphereLocals",
        light_pos_buf: gfx::ConstantBuffer<LightInfo> = "LightPosBlock",
        out_color: gfx::BlendTarget<GFormat> =
            ("Target0", gfx::state::MASK_ALL, gfx::preset::blend::ADD),
        out_depth: gfx::DepthTarget<Depth> =
            gfx::preset::depth::LESS_EQUAL_TEST,
    }
    
    pipeline light {
        vbuf: gfx::VertexBuffer<SphereVertex> = (),
        locals_vs: gfx::ConstantBuffer<SphereLocals> = "SphereLocals",
        locals_ps: gfx::ConstantBuffer<LightLocals> = "LightLocals",
        light_pos_buf: gfx::ConstantBuffer<LightInfo> = "LightPosBlock",
        tex_pos: gfx::TextureSampler<[f32; 4]> = "t_Position",
        tex_normal: gfx::TextureSampler<[f32; 4]> = "t_Normal",
        tex_diffuse: gfx::TextureSampler<[f32; 4]> = "t_Diffuse",
        out_color: gfx::BlendTarget<GFormat> =
            ("Target0", gfx::state::MASK_ALL, gfx::preset::blend::ADD),
        out_depth: gfx::DepthTarget<Depth> =
            gfx::preset::depth::LESS_EQUAL_TEST,
    }

    pipeline blit {
        vbuf: gfx::VertexBuffer<BlitVertex> = (),
        tex: gfx::TextureSampler<[f32; 4]> = "t_BlitTex",
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

const BLIT_FRAGMENT_SHADER: &'static [u8] = b"
    #version 150 core

    uniform sampler2D t_BlitTex;

    in vec2 v_TexCoord;
    
    out vec4 Target0;

    void main() {
        vec4 tex = texture(t_BlitTex, v_TexCoord);
        Target0 = tex;
    }
";

const BLIT_VERTEX_SHADER: &'static [u8] = b"
    #version 150 core

    in ivec4 a_PosTexCoord;
    
    out vec2 v_TexCoord;

    void main() {
        v_TexCoord = a_PosTexCoord.zw;
        gl_Position = vec4(a_PosTexCoord.xy, 0.0, 1.0);
    }
";

const LIGHT_FRAGMENT_SHADER: &'static [u8] = b"
    #version 150 core

    layout(std140)
    
    uniform LightLocals {
        vec4 u_CamPosAndRadius;
    };

    uniform sampler2D t_Position;
    uniform sampler2D t_Normal;
    uniform sampler2D t_Diffuse;
    
    in vec3 v_LightPos;
    
    out vec4 Target0;

    void main() {
        ivec2 itc = ivec2(gl_FragCoord.xy);
        vec3 pos     = texelFetch(t_Position, itc, 0).xyz;
        vec3 normal  = texelFetch(t_Normal,   itc, 0).xyz;
        vec3 diffuse = texelFetch(t_Diffuse,  itc, 0).xyz;

        vec3 light    = v_LightPos;
        vec3 to_light = normalize(light - pos);
        vec3 to_cam   = normalize(u_CamPosAndRadius.xyz - pos);

        vec3 n = normalize(normal);
        float s = pow(max(0.0, dot(to_cam, reflect(-to_light, n))), 20.0);
        float d = max(0.0, dot(n, to_light));

        float dist_sq = dot(light - pos, light - pos);
        float scale = max(0.0, 1.0 - dist_sq * u_CamPosAndRadius.w);

        vec3 res_color = d * diffuse + vec3(s);

        Target0 = vec4(scale * res_color, 1.0);
    }
";

// TODO: NUM_LIGHTS as uniform
const LIGHT_VERTEX_SHADER: &'static [u8] = b"
    #version 150 core

    in ivec3 a_Pos;

    out vec3 v_LightPos;

    layout(std140)
    uniform SphereLocals {
        mat4 u_Transform;
        float u_Radius;
    };

    struct LightInfo {
        vec4 pos;
    };

    const int NUM_LIGHTS = 100;
    
    layout(std140)
    uniform LightPosBlock {
        LightInfo u_Lights[NUM_LIGHTS];
    };

    void main() {
        v_LightPos = u_Lights[gl_InstanceID].pos.xyz;
        gl_Position = u_Transform * vec4(u_Radius * a_Pos + v_LightPos, 1.0);
    }
";

const EMITTER_FRAGMENT_SHADER: &'static [u8] = b"
    #version 150 core

    out vec4 Target0;

    void main() {
        Target0 = vec4(1.0, 1.0, 1.0, 1.0);
    }
";

const EMITTER_VERTEX_SHADER: &'static [u8] = b"
    #version 150 core

    in ivec3 a_Pos;

    layout(std140)
    uniform SphereLocals {
        mat4 u_Transform;
        float u_Radius;
    };

    struct LightInfo {
        vec4 pos;
    };

    const int NUM_LIGHTS = 100;
    
    layout(std140)
    uniform LightPosBlock {
        LightInfo u_Lights[NUM_LIGHTS];
    };

    void main() {
        gl_Position = u_Transform * vec4(u_Radius * a_Pos + u_Lights[gl_InstanceID].pos.xyz, 1.0);
    }
";

pub type ColorFormat = gfx::format::Srgba8;

const NUMBER_OF_LIGHTS: u32 = 100;
const LIGHT_RADIUS: f32 = 4.0;
const EMITTER_RADIUS: f32 = 0.5;

pub type GFormat = [f32; 4];

pub struct ViewPair<R: gfx::Resources, T: gfx::format::Formatted> {
    resource: gfx::handle::ShaderResourceView<R, T::View>,
    target: gfx::handle::RenderTargetView<R, T>,
}

pub struct DepthFormat;

impl gfx::format::Formatted for DepthFormat {
    type Surface = gfx::format::D24;
    type Channel = gfx::format::Unorm;
    type View = [f32; 4];

    fn get_format() -> gfx::format::Format {
        use gfx::format as f;
        f::Format(f::SurfaceType::D24, f::ChannelType::Unorm)
    }
}

pub struct DeferredLightSystem<R: gfx::Resources> {
    light_pos: Vec<LightInfo>,
    light: Bundle<R, light::Data<R>>,
    blit: Bundle<R, blit::Data<R>>,
    emitter: Bundle<R, emitter::Data<R>>,
    intermediate: ViewPair<R, GFormat>,
    depth_rv: gfx::handle::ShaderResourceView<R, [f32; 4]>,
}

fn create_g_buffer<R: gfx::Resources, F: gfx::Factory<R>>(target_width: texture::Size,
                                                          target_height: texture::Size,
                                                          factory: &mut F)
                                                          -> (ViewPair<R, GFormat>,
                                                              ViewPair<R, GFormat>,
                                                              ViewPair<R, GFormat>,
                                                              gfx::handle::ShaderResourceView<R,
                                                                                              [f32;
                                                                                               4]>,
                                                              gfx::handle::DepthStencilView<R,
                                                                                            Depth>) {
    use gfx::format::ChannelSource;

    let pos = {
        let (_, srv, rtv) = factory.create_render_target(target_width, target_height)
            .unwrap();
        ViewPair {
            resource: srv,
            target: rtv,
        }
    };
    let normal = {
        let (_, srv, rtv) = factory.create_render_target(target_width, target_height)
            .unwrap();
        ViewPair {
            resource: srv,
            target: rtv,
        }
    };
    let diffuse = {
        let (_, srv, rtv) = factory.create_render_target(target_width, target_height)
            .unwrap();
        ViewPair {
            resource: srv,
            target: rtv,
        }
    };
    let (tex, _srv, depth_rtv) = factory.create_depth_stencil(target_width, target_height)
        .unwrap();
    let swizzle = gfx::format::Swizzle(ChannelSource::X,
                                       ChannelSource::X,
                                       ChannelSource::X,
                                       ChannelSource::X);
    let depth_srv = factory.view_texture_as_shader_resource::<DepthFormat>(&tex, (0, 0), swizzle)
        .unwrap();

    (pos, normal, diffuse, depth_srv, depth_rtv)
}

impl<R: gfx::Resources> DeferredLightSystem<R> {
    pub fn new<F: gfx::Factory<R>>(factory: &mut F,
                                   target_width: u16,
                                   target_height: u16,
                                   main_color: gfx::handle::RenderTargetView<R, ColorFormat>)
                                   -> Self {
        use gfx::traits::FactoryExt;

        let (gpos, gnormal, gdiffuse, depth_resource, depth_target) =
            create_g_buffer(target_width, target_height, factory);

        let res = {
            let (_, srv, rtv) = factory.create_render_target(target_width, target_height).unwrap();
            ViewPair {
                resource: srv,
                target: rtv,
            }
        };

        let sampler = factory.create_sampler(texture::SamplerInfo::new(texture::FilterMethod::Scale,
                                                      texture::WrapMode::Clamp));

        let blit = {
            let vertex_data = [BlitVertex { pos_tex: [-3, -1, -1, 0] },
                               BlitVertex { pos_tex: [1, -1, 1, 0] },
                               BlitVertex { pos_tex: [1, 3, 1, 2] }];

            let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&vertex_data, ());

            let pso = factory.create_pipeline_simple(BLIT_VERTEX_SHADER, BLIT_FRAGMENT_SHADER,
                                        blit::new())
                .unwrap();

            let data = blit::Data {
                vbuf: vbuf,
                tex: (gpos.resource.clone(), sampler.clone()),
                out: main_color.clone(),
            };

            Bundle::new(slice, pso, data)
        };

        let light_pos_buffer = factory.create_constant_buffer(NUMBER_OF_LIGHTS as usize);

        let (light_vbuf, mut light_slice) = {
            let s = SphereUV::new(10, 10);

            let v_data: Vec<SphereVertex> = s.shared_vertex_iter()
                .map(|(x, y, z)| SphereVertex { pos: [x as i8, y as i8, z as i8, 1] })
                .collect();

            let idx_data: Vec<u32> = s.indexed_polygon_iter()
                .triangulate()
                .vertices()
                .map(|i| i as u32)
                .collect();

            factory.create_vertex_buffer_with_slice(&v_data, &idx_data[..])
        };

        light_slice.instances = Some((NUMBER_OF_LIGHTS as gfx::InstanceCount, 0));

        let light = {
            let pso =
                factory.create_pipeline_simple(LIGHT_VERTEX_SHADER,
                                            LIGHT_FRAGMENT_SHADER,
                                            light::new())
                    .unwrap();

            let data = light::Data {
                vbuf: light_vbuf.clone(),
                locals_vs: factory.create_constant_buffer(1),
                locals_ps: factory.create_constant_buffer(1),
                light_pos_buf: light_pos_buffer.clone(),
                tex_pos: (gpos.resource.clone(), sampler.clone()),
                tex_normal: (gnormal.resource.clone(), sampler.clone()),
                tex_diffuse: (gdiffuse.resource.clone(), sampler.clone()),
                out_color: res.target.clone(),
                out_depth: depth_target.clone(),
            };

            Bundle::new(light_slice.clone(), pso, data)
        };

        let emitter = {
            let pso = factory.create_pipeline_simple(EMITTER_VERTEX_SHADER,
                                        EMITTER_FRAGMENT_SHADER,
                                        emitter::new())
                .unwrap();

            let data = emitter::Data {
                vbuf: light_vbuf.clone(),
                locals: factory.create_constant_buffer(1),
                light_pos_buf: light_pos_buffer.clone(),
                out_color: res.target.clone(),
                out_depth: depth_target.clone(),
            };

            Bundle::new(light_slice, pso, data)
        };

        DeferredLightSystem {
            light_pos: (0..NUMBER_OF_LIGHTS)
                .map(|_| LightInfo { pos: [0.0, 0.0, 0.0, 0.0] })
                .collect(),
            light: light,
            emitter: emitter,
            blit: blit,
            intermediate: res,
            depth_rv: depth_resource,
        }
    }

    pub fn render<C: gfx::CommandBuffer<R>>(&mut self,
                                            seed: &noise::PermutationTable,
                                            cam_pos: Point3<f32>,
                                            encoder: &mut gfx::Encoder<R, C>,
                                            view_proj: [[f32; 4]; 4]) {
        let light_locals = LightLocals {
            cam_pos_and_radius: [cam_pos.x,
                                 cam_pos.y,
                                 cam_pos.z,
                                 1.0 / (LIGHT_RADIUS * LIGHT_RADIUS)],
        };
        encoder.update_buffer(&self.light.data.locals_ps, &[light_locals], 0).unwrap();

        let mut sphere_locals = SphereLocals {
            transform: view_proj,
            radius: LIGHT_RADIUS,
        };
        encoder.update_constant_buffer(&self.light.data.locals_vs, &sphere_locals);
        sphere_locals.radius = EMITTER_RADIUS;

        for (i, d) in self.light_pos.iter_mut().enumerate() {
            let (x, z) = {
                let ix = i as f32;
                let scale_factor = 1.0 - (ix * ix) / ((NUMBER_OF_LIGHTS * NUMBER_OF_LIGHTS) as f32);
                (scale_factor, scale_factor)
            };
            let y = perlin2(seed, &[x, z]) * 140.0 + 10.0;

            d.pos[0] = x * 500.0;
            d.pos[1] = y;
            d.pos[2] = z * 500.0;
        }

        encoder.update_buffer(&self.light.data.light_pos_buf, &self.light_pos, 0).unwrap();

        let blit_tex = {
            encoder.clear(&self.intermediate.target, [0.0, 0.0, 0.0, 1.0]);
            self.light.encode(encoder);
            self.emitter.encode(encoder);
            &self.intermediate.resource
        };
        self.blit.data.tex.0 = blit_tex.clone();
        self.blit.encode(encoder);
    }
}
