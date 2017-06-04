
use gfx;
use gfx::{Bundle, texture};
pub use gfx::format::Depth;
use na::{Point3, Vector2, Vector3, Matrix4, Isometry3, Perspective3, Translation3};
use na;
use noise::perlin2;
use noise;

use alewife;
use core::event;
use rendering;
use rendering::colors;

use genmesh::generators::SphereUV;
use genmesh::{Vertices, Triangulate};
use genmesh::generators::{Plane, SharedVertex, IndexedPolygon};

gfx_defines!{
    vertex BlitVertex {
        pos_tex: [i8; 4] = "a_PosTexCoord",
    }

    vertex FXAAVertex {
        pos_tex: [i8; 4] = "a_PosTexCoord",
    }

    vertex TerrainVertex {
        pos: [f32; 3] = "a_Pos",
        normal: [f32; 3] = "a_Normal",
        color: [f32; 3] = "a_Color",
    }

    vertex CubeVertex {
        pos: [i8; 4] = "a_Pos",
    }

    constant LightLocals {
        cam_pos_and_radius: [f32; 4] = "u_CamPosAndRadius",
    }

    constant CubeLocals {
        transform: [[f32; 4]; 4] = "u_Transform",
        radius: f32 = "u_Radius",
    }
    
    constant TerrainLocals {
        model: [[f32; 4]; 4] = "u_Model",
        viewProj: [[f32; 4]; 4] = "u_ViewProj",
    }

    constant LightInfo {
        pos: [f32; 4] = "pos",
    }
/*
    constant BlitLocals {
        inverse_tex_size: [f32; 3] = "u_InverseTextureSize",
    }
*/
    pipeline terrain {
        vbuf: gfx::VertexBuffer<TerrainVertex> = (),
        locals: gfx::ConstantBuffer<TerrainLocals> = "TerrainLocals",
        out_position: gfx::RenderTarget<GFormat> = "Target0",
        out_normal: gfx::RenderTarget<GFormat> = "Target1",
        out_color: gfx::RenderTarget<GFormat> = "Target2",
        out_depth: gfx::DepthTarget<Depth> =
            gfx::preset::depth::LESS_EQUAL_WRITE,
    }

    pipeline emitter {
        vbuf: gfx::VertexBuffer<CubeVertex> = (),
        locals: gfx::ConstantBuffer<CubeLocals> = "CubeLocals",
        light_pos_buf: gfx::ConstantBuffer<LightInfo> = "LightPosBlock",
        out_color: gfx::BlendTarget<GFormat> =
            ("Target0", gfx::state::MASK_ALL, gfx::preset::blend::ADD),
        out_depth: gfx::DepthTarget<Depth> =
            gfx::preset::depth::LESS_EQUAL_TEST,
    }
    
    pipeline light {
        vbuf: gfx::VertexBuffer<CubeVertex> = (),
        locals_vs: gfx::ConstantBuffer<CubeLocals> = "CubeLocals",
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
     //   locals: gfx::ConstantBuffer<BlitLocals> = "BlitLocals",
        tex: gfx::TextureSampler<[f32; 4]> = "t_BlitTex",
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }

    pipeline fxaa {
        vbuf: gfx::VertexBuffer<FXAAVertex> = (),
     //   locals: gfx::ConstantBuffer<BlitLocals> = "BlitLocals",
        tex: gfx::TextureSampler<[f32; 4]> = "t_FXAATex",
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

const FXAA_FRAGMENT_SHADER: &'static [u8] = b"
    #version 150 core

    uniform sampler2D t_FXAATex;
    // uniform vec3 u_InverseTextureSize;

    in vec2 v_TexCoord;

    out vec4 Target0;

    void main() {

        float MAX_SPAN = 8.0;
        float REDUCE_MIN = 1.0 / 128.0;
        float REDUCE_MUL = 1.0 / 8.0;

        vec3 luma = vec3(0.299, 0.587, 0.114);
        vec2 offset = vec2(1.0 / 1200.0, 1.0 / 1000.0);
        float lumaTL = dot(luma, texture(t_FXAATex, v_TexCoord.xy + vec2(-1.0, -1.0) * offset).rgb);
        float lumaTR = dot(luma, texture(t_FXAATex, v_TexCoord.xy + vec2(1.0, -1.0) * offset).rgb);
        float lumaBL = dot(luma, texture(t_FXAATex, v_TexCoord.xy + vec2(-1.0, 1.0) * offset).rgb);
        float lumaBR = dot(luma, texture(t_FXAATex, v_TexCoord.xy + vec2(1.0, 1.0) * offset).rgb);
        float lumaM  = dot(luma, texture(t_FXAATex, v_TexCoord.xy).rgb);

        vec2 blur_dir;
        blur_dir.x = -((lumaTL + lumaTR) - (lumaBL + lumaBR));
        blur_dir.y = ((lumaTL + lumaBL) - (lumaTR + lumaBR));

        float dirReduce = max((lumaTL + lumaTR + lumaBL + lumaBR) * REDUCE_MUL * 0.25, REDUCE_MIN);
        float resV = 1.0 / (min(abs(blur_dir.x), abs(blur_dir.y)) + dirReduce);

        blur_dir = min(vec2(MAX_SPAN, MAX_SPAN), 
                       max(vec2(-MAX_SPAN, -MAX_SPAN), blur_dir * resV)) * offset;

        vec3 res1 = (1.0 / 2.0) * 
            (texture(t_FXAATex, v_TexCoord.xy + (blur_dir * vec2(1.0 / 3.0 - 0.5))).rgb +
             texture(t_FXAATex, v_TexCoord.xy + (blur_dir * vec2(2.0 / 3.0 - 0.5))).rgb);

        vec3 res2 = res1 * (1.0 / 2.0) + (1.0 / 4.0) * 
            (texture(t_FXAATex, v_TexCoord.xy + (blur_dir * vec2(0.0 / 3.0 - 0.5))).rgb +
             texture(t_FXAATex, v_TexCoord.xy + (blur_dir * vec2(3.0 / 3.0 - 0.5))).rgb);

        float lumaMin = min(lumaM, min(min(lumaTL, lumaTR), min(lumaBL, lumaBR)));
        float lumaMax = max(lumaM, max(max(lumaTL, lumaTR), max(lumaBL, lumaBR)));
        float lumaRes2 = dot(luma, res2);

        if (lumaRes2 < lumaMin || lumaRes2 > lumaMax) {
            Target0 = vec4(res1, 1.0);
        } else {
            Target0 = vec4(res2, 1.0);
        }
    }
";

const FXAA_VERTEX_SHADER: &'static [u8] = b"
    #version 150 core

    in ivec4 a_PosTexCoord;
    
    out vec2 v_TexCoord;

    void main() {
        v_TexCoord = a_PosTexCoord.zw;
        gl_Position = vec4(a_PosTexCoord.xy, 0.0, 1.0);
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
    uniform CubeLocals {
        mat4 u_Transform;
        float u_Radius;
    };

    struct LightInfo {
        vec4 pos;
    };

    const int NUM_LIGHTS = 250;
    
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
    uniform CubeLocals {
        mat4 u_Transform;
        float u_Radius;
    };

    struct LightInfo {
        vec4 pos;
    };

    const int NUM_LIGHTS = 250;
    
    layout(std140)
    uniform LightPosBlock {
        LightInfo u_Lights[NUM_LIGHTS];
    };

    void main() {
        gl_Position = u_Transform * vec4(u_Radius * a_Pos + u_Lights[gl_InstanceID].pos.xyz, 1.0);
    }
";

const TERRAIN_FRAGMENT_SHADER: &'static [u8] = b"
    #version 150 core

    in vec3 v_FragPos;
    in vec3 v_Normal;
    in vec3 v_Color;
    
    out vec4 Target0;
    out vec4 Target1;
    out vec4 Target2;

    void main() {
        vec3 n = normalize(v_Normal);

        Target0 = vec4(v_FragPos, 0.0);
        Target1 = vec4(n, 0.0);
        Target2 = vec4(v_Color, 1.0);
    }
";

const TERRAIN_VERTEX_SHADER: &'static [u8] = b"
    #version 150 core

    layout(std140)
    uniform TerrainLocals {
        mat4 u_Model;
        mat4 u_ViewProj;
    };
    
    in vec3 a_Pos;
    in vec3 a_Normal;
    in vec3 a_Color;

    out vec3 v_FragPos;
    out vec3 v_Normal;
    out vec3 v_Color;

    void main() {
        v_FragPos = (u_Model * vec4(a_Pos, 1.0)).xyz;
        v_Normal = mat3(u_Model) * a_Normal;
        v_Color = a_Color;
        gl_Position = u_ViewProj * u_Model * vec4(a_Pos, 1.0);
    }
";

pub type ColorFormat = gfx::format::Srgba8;

const NUMBER_OF_LIGHTS: u32 = 250;
const LIGHT_RADIUS: f32 = 10.0;
const EMITTER_RADIUS: f32 = 0.5;
const TERRAIN_SCALE: [f32; 3] = [100.0, 100.0, 100.0];

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

fn calculate_normal(seed: &noise::PermutationTable, x: f32, y: f32) -> [f32; 3] {

    let sample_distance = 0.001;
    let s_x0 = x - sample_distance;
    let s_x1 = x + sample_distance;
    let s_y0 = y - sample_distance;
    let s_y1 = y + sample_distance;

    let dzdx = (perlin2(seed, &[s_x1, y]) - perlin2(seed, &[s_x0, y])) / (s_x1 - s_x0);
    let dzdy = (perlin2(seed, &[x, s_y1]) - perlin2(seed, &[x, s_y0])) / (s_y1 - s_y0);

    // Cross gradient vectors to get normal
    let v1 = Vector3::new(1.0, 0.0, dzdx);
    let v2 = Vector3::new(0.0, 1.0, dzdy);
    let normal = v1.cross(&v2).normalize();

    return normal.into();
}

fn calculate_color(height: f32) -> [f32; 3] {
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

pub struct DeferredLightSystem<R: gfx::Resources> {
    event_queue: alewife::Subscriber<event::EventID, event::Event>,
    fxaa_enabled: bool,
    terrain: Bundle<R, terrain::Data<R>>,
    blit: Bundle<R, blit::Data<R>>,
    fxaa: Bundle<R, fxaa::Data<R>>,
    light: Bundle<R, light::Data<R>>,
    emitter: Bundle<R, emitter::Data<R>>,
    intermediate: ViewPair<R, GFormat>,
    light_pos: Vec<LightInfo>,
    depth_resource: gfx::handle::ShaderResourceView<R, [f32; 4]>,
    debug_buf: Option<gfx::handle::ShaderResourceView<R, [f32; 4]>>,
    inverse_tex_size: [f32; 3],
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
    pub fn new<F: gfx::Factory<R>>(e_que: alewife::Subscriber<event::EventID, event::Event>,
                                   factory: &mut F,
                                   target_width: u16,
                                   target_height: u16,
                                   seed: &noise::PermutationTable,
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

        let terrain = {
            let plane = Plane::subdivide(256, 256);
            let vertex_data: Vec<TerrainVertex> = plane.shared_vertex_iter()
                .map(|(x, y)| {
                    let h = TERRAIN_SCALE[2] * perlin2(seed, &[x, y]);
                    TerrainVertex {
                        pos: [TERRAIN_SCALE[0] * x, TERRAIN_SCALE[1] * y, h],
                        normal: calculate_normal(seed, x, y),
                        color: calculate_color(h),
                    }
                })
                .collect();

            let index_data: Vec<u32> = plane.indexed_polygon_iter()
                .triangulate()
                .vertices()
                .map(|i| i as u32)
                .collect();

            let (vbuf, slice) =
                factory.create_vertex_buffer_with_slice(&vertex_data, &index_data[..]);

            let pso = factory.create_pipeline_simple(TERRAIN_VERTEX_SHADER,
                                        TERRAIN_FRAGMENT_SHADER,
                                        terrain::new())
                .unwrap();

            let data = terrain::Data {
                vbuf: vbuf,
                locals: factory.create_constant_buffer(1),
                out_position: gpos.target.clone(),
                out_normal: gnormal.target.clone(),
                out_color: gdiffuse.target.clone(),
                out_depth: depth_target.clone(),
            };

            Bundle::new(slice, pso, data)
        };

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
                //     locals: factory.create_constant_buffer(1),
                tex: (gpos.resource.clone(), sampler.clone()),
                out: main_color.clone(),
            };

            Bundle::new(slice, pso, data)
        };


        let fxaa = {
            let vertex_data = [FXAAVertex { pos_tex: [-3, -1, -1, 0] },
                               FXAAVertex { pos_tex: [1, -1, 1, 0] },
                               FXAAVertex { pos_tex: [1, 3, 1, 2] }];

            let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&vertex_data, ());

            let pso = factory.create_pipeline_simple(FXAA_VERTEX_SHADER, FXAA_FRAGMENT_SHADER,
                                        fxaa::new())
                .unwrap();

            let data = fxaa::Data {
                vbuf: vbuf,
                //     locals: factory.create_constant_buffer(1),
                tex: (gpos.resource.clone(), sampler.clone()),
                out: main_color.clone(),
            };

            Bundle::new(slice, pso, data)
        };

        let light_pos_buffer = factory.create_constant_buffer(NUMBER_OF_LIGHTS as usize);

        let (light_vbuf, mut light_slice) = {
            let vertex_data = [// top (0, 0, 1)
                               CubeVertex { pos: [-1, -1, 1, 1] },
                               CubeVertex { pos: [1, -1, 1, 1] },
                               CubeVertex { pos: [1, 1, 1, 1] },
                               CubeVertex { pos: [-1, 1, 1, 1] },
                               // bottom (0, 0, -1)
                               CubeVertex { pos: [-1, 1, -1, 1] },
                               CubeVertex { pos: [1, 1, -1, 1] },
                               CubeVertex { pos: [1, -1, -1, 1] },
                               CubeVertex { pos: [-1, -1, -1, 1] },
                               // right (1, 0, 0)
                               CubeVertex { pos: [1, -1, -1, 1] },
                               CubeVertex { pos: [1, 1, -1, 1] },
                               CubeVertex { pos: [1, 1, 1, 1] },
                               CubeVertex { pos: [1, -1, 1, 1] },
                               // left (-1, 0, 0)
                               CubeVertex { pos: [-1, -1, 1, 1] },
                               CubeVertex { pos: [-1, 1, 1, 1] },
                               CubeVertex { pos: [-1, 1, -1, 1] },
                               CubeVertex { pos: [-1, -1, -1, 1] },
                               // front (0, 1, 0)
                               CubeVertex { pos: [1, 1, -1, 1] },
                               CubeVertex { pos: [-1, 1, -1, 1] },
                               CubeVertex { pos: [-1, 1, 1, 1] },
                               CubeVertex { pos: [1, 1, 1, 1] },
                               // back (0, -1, 0)
                               CubeVertex { pos: [1, -1, 1, 1] },
                               CubeVertex { pos: [-1, -1, 1, 1] },
                               CubeVertex { pos: [-1, -1, -1, 1] },
                               CubeVertex { pos: [1, -1, -1, 1] }];

            let index_data: &[u16] = &[
                 0,  1,  2,  2,  3,  0, // top
                 4,  5,  6,  6,  7,  4, // bottom
                 8,  9, 10, 10, 11,  8, // right
                12, 13, 14, 14, 15, 12, // left
                16, 17, 18, 18, 19, 16, // front
                20, 21, 22, 22, 23, 20, // back
            ];

            factory.create_vertex_buffer_with_slice(&vertex_data, index_data)
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
            event_queue: e_que,
            fxaa_enabled: true,
            terrain: terrain,
            blit: blit,
            fxaa: fxaa,
            debug_buf: None,
            light: light,
            emitter: emitter,
            intermediate: res,
            light_pos: (0..NUMBER_OF_LIGHTS)
                .map(|_| LightInfo { pos: [0.0, 0.0, 0.0, 0.0] })
                .collect(),
            depth_resource: depth_resource,
            inverse_tex_size: [1.0 / target_width as f32, 1.0 / target_height as f32, 0.0],
        }
    }

    pub fn render<C: gfx::CommandBuffer<R>>(&mut self,
                                            time: f32,
                                            seed: &noise::PermutationTable,
                                            cam_pos: Point3<f32>,
                                            encoder: &mut gfx::Encoder<R, C>,
                                            view_proj: [[f32; 4]; 4]) {

        let events: Vec<_> = self.event_queue.fetch();

        for event in events {
            match event {
                (_, event::Event::ToggleFXAA) => {
                    self.fxaa_enabled = !self.fxaa_enabled;
                    info!(target: "DAT205", "FXAA state changed to {}", self.fxaa_enabled);
                }
                _ => {}
            }
        }

        let terrain_locals = TerrainLocals {
            model: Matrix4::identity().into(),
            viewProj: view_proj.clone(),
        };
        encoder.update_constant_buffer(&self.terrain.data.locals, &terrain_locals);

        let light_locals = LightLocals {
            cam_pos_and_radius: [cam_pos.x,
                                 cam_pos.y,
                                 cam_pos.z,
                                 1.0 / (LIGHT_RADIUS * LIGHT_RADIUS)],
        };
        encoder.update_buffer(&self.light.data.locals_ps, &[light_locals], 0).unwrap();

        let mut cube_locals = CubeLocals {
            transform: view_proj.clone(),
            radius: LIGHT_RADIUS,
        };
        encoder.update_constant_buffer(&self.light.data.locals_vs, &cube_locals);
        cube_locals.radius = EMITTER_RADIUS;
        encoder.update_constant_buffer(&self.emitter.data.locals, &cube_locals);

        // Update light positions
        for (i, d) in self.light_pos.iter_mut().enumerate() {
            let (x, y) = {
                let fi = i as f32;
                // Distribute lights nicely
                let r = 1.0 - (fi * fi) / ((NUMBER_OF_LIGHTS * NUMBER_OF_LIGHTS) as f32);
                (r * (0.2 + i as f32).cos(), r * (0.2 + i as f32).sin())
            };
            let h = perlin2(seed, &[x, y]);

            d.pos[0] = TERRAIN_SCALE[0] * x;
            d.pos[1] = TERRAIN_SCALE[1] * y;
            d.pos[2] = TERRAIN_SCALE[2] * h + 5.0 * time.cos();
        }
        encoder.update_buffer(&self.light.data.light_pos_buf, &self.light_pos, 0).unwrap();

        encoder.clear_depth(&self.terrain.data.out_depth, 1.0);
        encoder.clear(&self.terrain.data.out_position, [0.0, 0.0, 0.0, 1.0]);
        encoder.clear(&self.terrain.data.out_normal, [0.0, 0.0, 0.0, 1.0]);
        encoder.clear(&self.terrain.data.out_color, [0.0, 0.0, 0.0, 1.0]);

        self.terrain.encode(encoder);

        if self.fxaa_enabled {
            let fxaa_tex = match self.debug_buf {
                Some(ref tex) => tex,   // Show one of the immediate buffers
                None => {
                    encoder.clear(&self.intermediate.target, [0.0, 0.0, 0.0, 1.0]);
                    // Apply lights
                    self.light.encode(encoder);
                    // Draw light emitters
                    self.emitter.encode(encoder);
                    &self.intermediate.resource
                }
            };

            self.fxaa.data.tex.0 = fxaa_tex.clone();
            self.fxaa.encode(encoder);
        } else {
            let blit_tex = match self.debug_buf {
                Some(ref tex) => tex,   // Show one of the immediate buffers
                None => {
                    encoder.clear(&self.intermediate.target, [0.0, 0.0, 0.0, 1.0]);
                    // Apply lights
                    self.light.encode(encoder);
                    // Draw light emitters
                    self.emitter.encode(encoder);
                    &self.intermediate.resource
                }
            };

            self.blit.data.tex.0 = blit_tex.clone();
            self.blit.encode(encoder);
        }
    }
}
