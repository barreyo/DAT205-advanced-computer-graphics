
use conrod;
use conrod::Color;
use conrod::render::Text;
use conrod::text::rt;
use conrod::text;
use gfx;

use gfx::texture;
use gfx::traits::FactoryExt;

pub type ColorFormat = gfx::format::Srgba8;
type SurfaceFormat = gfx::format::R8_G8_B8_A8;
type FullFormat = (SurfaceFormat, gfx::format::Unorm);

const FRAGMENT_SHADER: &'static [u8] = b"
        #version 140
        uniform sampler2D t_Color;
        in vec2 v_Uv;
        in vec4 v_Color;
        out vec4 f_Color;
        void main() {
            vec4 tex = texture(t_Color, v_Uv);
            f_Color = v_Color * tex;
        }
    ";

const VERTEX_SHADER: &'static [u8] = b"
        #version 140
        in vec2 a_Pos;
        in vec2 a_Uv;
        in vec4 a_Color;
        out vec2 v_Uv;
        out vec4 v_Color;
        void main() {
            v_Uv = a_Uv;
            v_Color = a_Color;
            gl_Position = vec4(a_Pos, 0.0, 1.0);
        }
    ";

// Vertex and pipeline declarations
gfx_defines! {
    vertex TextVertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
        color: [f32; 4] = "a_Color",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<TextVertex> = (),
        color: gfx::TextureSampler<[f32; 4]> = "t_Color",
        out: gfx::BlendTarget<ColorFormat> = ("f_Color", ::gfx::state::MASK_ALL, ::gfx::preset::blend::ALPHA),
    }
}

// Convenience constructor
impl TextVertex {
    fn new(pos: [f32; 2], uv: [f32; 2], color: [f32; 4]) -> TextVertex {
        TextVertex {
            pos: pos,
            uv: uv,
            color: color,
        }
    }
}

// Creates a gfx texture with the given data
fn create_texture<F, R>
    (factory: &mut F,
     width: u32,
     height: u32,
     data: &[u8])
     -> (gfx::handle::Texture<R, SurfaceFormat>, gfx::handle::ShaderResourceView<R, [f32; 4]>)
    where R: gfx::Resources,
          F: gfx::Factory<R>
{
    // Modified `Factory::create_texture_immutable_u8` for dynamic texture.
    fn create_texture<T, F, R>(factory: &mut F,
                               kind: gfx::texture::Kind,
                               data: &[&[u8]])
                               -> Result<(gfx::handle::Texture<R, T::Surface>,
                                          gfx::handle::ShaderResourceView<R, T::View>),
                                         gfx::CombinedError>
        where F: gfx::Factory<R>,
              R: gfx::Resources,
              T: gfx::format::TextureFormat
    {
        use gfx::{format, texture};
        use gfx::memory::{Usage, SHADER_RESOURCE};
        use gfx_core::memory::Typed;

        let surface = <T::Surface as format::SurfaceTyped>::get_surface_type();
        let num_slices = kind.get_num_slices().unwrap_or(1) as usize;
        let num_faces = if kind.is_cube() { 6 } else { 1 };
        let desc = texture::Info {
            kind: kind,
            levels: (data.len() / (num_slices * num_faces)) as texture::Level,
            format: surface,
            bind: SHADER_RESOURCE,
            usage: Usage::Dynamic,
        };
        let cty = <T::Channel as format::ChannelTyped>::get_channel_type();
        let raw = try!(factory.create_texture_raw(desc, Some(cty), Some(data)));
        let levels = (0, raw.get_info().levels - 1);
        let tex = Typed::new(raw);
        let view = try!(factory.view_texture_as_shader_resource::<T>(
                &tex, levels, format::Swizzle::new()
            ));
        Ok((tex, view))
    }

    let kind = texture::Kind::D2(width as texture::Size,
                                 height as texture::Size,
                                 texture::AaMode::Single);
    create_texture::<ColorFormat, F, R>(factory, kind, &[data]).unwrap()
}

// Updates a texture with the given data (used for updating the GlyphCache texture)
fn update_texture<R, C>(encoder: &mut gfx::Encoder<R, C>,
                        texture: &gfx::handle::Texture<R, SurfaceFormat>,
                        offset: [u16; 2],
                        size: [u16; 2],
                        data: &[[u8; 4]])
    where R: gfx::Resources,
          C: gfx::CommandBuffer<R>
{
    let info = texture::ImageInfoCommon {
        xoffset: offset[0],
        yoffset: offset[1],
        zoffset: 0,
        width: size[0],
        height: size[1],
        depth: 0,
        format: (),
        mipmap: 0,
    };

    encoder.update_texture::<SurfaceFormat, FullFormat>(texture, None, info, data).unwrap();
}

pub struct TextRenderer<R: gfx::Resources> {
    pso: gfx::PipelineState<R, pipe::Meta>,
    data: pipe::Data<R>,
    glyph_cache: conrod::text::GlyphCache,
    texture: gfx::handle::Texture<R, SurfaceFormat>,
    texture_view: gfx::handle::ShaderResourceView<R, [f32; 4]>,
    vertices: Vec<TextVertex>,
    dpi: f32,
    screen_width: f32,
    screen_height: f32,
}

impl<R: gfx::Resources> TextRenderer<R> {
    pub fn new<F: gfx::Factory<R>>(window_width: f32,
                                   window_height: f32,
                                   dpi: f32,
                                   main_color: gfx::handle::RenderTargetView<R, ColorFormat>,
                                   factory: &mut F)
                                   -> Self {

        // Create texture sampler
        let sampler_info = texture::SamplerInfo::new(texture::FilterMethod::Bilinear,
                                                     texture::WrapMode::Clamp);
        let sampler = factory.create_sampler(sampler_info);

        // Dummy values for initialization
        let vbuf = factory.create_vertex_buffer(&[]);
        let (_, fake_texture) = create_texture(factory, 2, 2, &[0; 4]);

        let data = pipe::Data {
            vbuf: vbuf,
            color: (fake_texture.clone(), sampler),
            out: main_color.clone(),
        };

        // Compile GL program
        let pso = factory.create_pipeline_simple(VERTEX_SHADER, FRAGMENT_SHADER, pipe::new())
            .unwrap();

        // Create glyph cache and its texture
        let (glyph_cache, cache_tex, cache_tex_view) = {
            let width = (window_width * dpi) as u32;
            let height = (window_height * dpi) as u32;

            const SCALE_TOLERANCE: f32 = 0.1;
            const POSITION_TOLERANCE: f32 = 0.1;

            let cache =
                conrod::text::GlyphCache::new(width, height, SCALE_TOLERANCE, POSITION_TOLERANCE);

            let data = vec![0; (width * height * 4) as usize];

            let (texture, texture_view) = create_texture(factory, width, height, &data);

            (cache, texture, texture_view)
        };

        TextRenderer {
            pso: pso,
            data: data,
            glyph_cache: glyph_cache,
            texture: cache_tex,
            texture_view: cache_tex_view,
            vertices: Vec::new(),
            dpi: dpi,
            screen_width: window_width,
            screen_height: window_height,
        }
    }

    pub fn prepare_frame(&mut self, dpi: f32, screen_width: f32, screen_height: f32) {
        self.vertices = Vec::new();
        self.dpi = dpi;
        self.screen_height = screen_width * dpi;
        self.screen_width = screen_height * dpi;
    }

    pub fn add_text<C: gfx::CommandBuffer<R>>(&mut self,
                                              color: Color,
                                              text: Text,
                                              font_id: text::font::Id,
                                              encoder: &mut gfx::Encoder<R, C>) {

        let positioned_glyphs = text.positioned_glyphs(self.dpi);

        // Queue the glyphs to be cached
        for glyph in positioned_glyphs {
            self.glyph_cache.queue_glyph(font_id.index(), glyph.clone());
        }

        let texture = &self.texture;
        self.glyph_cache
            .cache_queued(|rect, data| {
                let offset = [rect.min.x as u16, rect.min.y as u16];
                let size = [rect.width() as u16, rect.height() as u16];

                let new_data = data.iter().map(|x| [0, 0, 0, *x]).collect::<Vec<_>>();

                update_texture(encoder, texture, offset, size, &new_data);
            })
            .unwrap();

        let color = color.to_fsa();
        let cache_id = font_id.index();
        let origin = rt::point(0.0, 0.0);

        let sw = self.screen_width;
        let sh = self.screen_height;

        // A closure to convert RustType rects to GL rects
        let to_gl_rect = |screen_rect: rt::Rect<i32>| {
            rt::Rect {
                min: origin +
                     (rt::vector(screen_rect.min.x as f32 / sw - 0.5,
                                 1.0 - screen_rect.min.y as f32 / sh - 0.5)) *
                     2.0,
                max: origin +
                     (rt::vector(screen_rect.max.x as f32 / sw - 0.5,
                                 1.0 - screen_rect.max.y as f32 / sh - 0.5)) *
                     2.0,
            }
        };

        let ref gc = self.glyph_cache;
        // Create new vertices
        let extension = positioned_glyphs.into_iter()
            .filter_map(|g| gc.rect_for(cache_id, g).ok().unwrap_or(None))
            .flat_map(|(uv_rect, screen_rect)| {
                use std::iter::once;

                let gl_rect = to_gl_rect(screen_rect);
                let v = |pos, uv| once(TextVertex::new(pos, uv, color));

                v([gl_rect.min.x, gl_rect.max.y],
                  [uv_rect.min.x, uv_rect.max.y])
                    .chain(v([gl_rect.min.x, gl_rect.min.y],
                             [uv_rect.min.x, uv_rect.min.y]))
                    .chain(v([gl_rect.max.x, gl_rect.min.y],
                             [uv_rect.max.x, uv_rect.min.y]))
                    .chain(v([gl_rect.max.x, gl_rect.min.y],
                             [uv_rect.max.x, uv_rect.min.y]))
                    .chain(v([gl_rect.max.x, gl_rect.max.y],
                             [uv_rect.max.x, uv_rect.max.y]))
                    .chain(v([gl_rect.min.x, gl_rect.max.y],
                             [uv_rect.min.x, uv_rect.max.y]))
            });

        self.vertices.extend(extension);
    }

    pub fn render<C: gfx::CommandBuffer<R>, F: gfx::Factory<R>>(&mut self,
                                                                encoder: &mut gfx::Encoder<R, C>,
                                                                factory: &mut F) {
        self.data.color.0 = self.texture_view.clone();
        let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&self.vertices, ());
        self.data.vbuf = vbuf;
        encoder.draw(&slice, &self.pso, &self.data);
    }
}
