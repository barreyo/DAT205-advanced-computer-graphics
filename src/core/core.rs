
pub mod core {

    use conrod;
    use glutin;
    use glutin::Event;
    use gfx;
    use gfx_window_glutin;

    use gfx::{Factory, Device, texture};
    use gfx::traits::FactoryExt;
    use conrod::render;
    use conrod::text::rt;

    use alewife;
    use find_folder;

    use core::event;
    use support;
    use ui;

    const DEFAULT_WINDOW_WIDTH: u32 = 1200;
    const DEFAULT_WINDOW_HEIGHT: u32 = 1000;

    const CLEAR_COLOR: [f32; 4] = [0.1, 0.2, 0.3, 1.0];

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

    // Format definitions (must be pub for  gfx_defines to use them)
    pub type ColorFormat = gfx::format::Srgba8;
    type DepthFormat = gfx::format::DepthStencil;
    type SurfaceFormat = gfx::format::R8_G8_B8_A8;
    type FullFormat = (SurfaceFormat, gfx::format::Unorm);

    // Vertex and pipeline declarations
    gfx_defines! {
        vertex Vertex {
            pos: [f32; 2] = "a_Pos",
            uv: [f32; 2] = "a_Uv",
            color: [f32; 4] = "a_Color",
        }

        pipeline pipe {
            vbuf: gfx::VertexBuffer<Vertex> = (),
            color: gfx::TextureSampler<[f32; 4]> = "t_Color",
            out: gfx::BlendTarget<ColorFormat> = ("f_Color", ::gfx::state::MASK_ALL, ::gfx::preset::blend::ALPHA),
        }
    }

    // Convenience constructor
    impl Vertex {
        fn new(pos: [f32; 2], uv: [f32; 2], color: [f32; 4]) -> Vertex {
            Vertex {
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

    pub fn init() {

        // Setup the message bus for core systems
        let mut bus = alewife::Publisher::<event::EventID, event::Event>::new();

        let console_sub = bus.add_subscriber(&[event::EventID::UIEvent,
                                               event::EventID::RenderEvent,
                                               event::EventID::WindowEvent,
                                               event::EventID::EntityEvent]);
        let renderer_sub = bus.add_subscriber(&[event::EventID::RenderEvent]);

        // TODO: Create a REDO module, sub to some events and save them in buffer
        //       When invoked perform events in reverse. Events need to send state.
        //       CMD-z -> sends message to module to perform step.

        // Once we have built the message bus we can clone it to all
        // modules that wanna publish to it.
        let publisher = bus.build();

        let logger = support::logging::LogBuilder::new()
            .with_publisher(publisher.clone())
            .init();

        // Builder for window
        let builder = glutin::WindowBuilder::new()
            .with_title("Advanced Computer Graphics")
            .with_dimensions(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT);

        // Initialize gfx things
        let (window, mut device, mut factory, main_color, _) =
            gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);
        //let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

        // Create texture sampler
        let sampler_info = texture::SamplerInfo::new(texture::FilterMethod::Bilinear,
                                                     texture::WrapMode::Clamp);
        let sampler = factory.create_sampler(sampler_info);

        // Dummy values for initialization
        let vbuf = factory.create_vertex_buffer(&[]);
        let (_, fake_texture) = create_texture(&mut factory, 2, 2, &[0; 4]);

        let mut data = pipe::Data {
            vbuf: vbuf,
            color: (fake_texture.clone(), sampler),
            out: main_color.clone(),
        };

        // Compile GL program
        let pso = factory.create_pipeline_simple(VERTEX_SHADER, FRAGMENT_SHADER, pipe::new())
            .unwrap();

        // Create Ui and Ids of widgets to instantiate
        let mut ui = conrod::UiBuilder::new([DEFAULT_WINDOW_WIDTH as f64,
                                             DEFAULT_WINDOW_HEIGHT as f64])
            .build();

        // Load font from file
        let assets = find_folder::Search::KidsThenParents(2, 4).for_folder("assets").unwrap();
        let font_path = assets.join("fonts/noto_sans_regular.ttf");
        ui.fonts.insert_from_file(font_path).unwrap();

        // Create glyph cache and its texture
        let (mut glyph_cache, cache_tex, cache_tex_view) = {
            let dpi = window.hidpi_factor();
            let width = (DEFAULT_WINDOW_WIDTH as f32 * dpi) as u32;
            let height = (DEFAULT_WINDOW_HEIGHT as f32 * dpi) as u32;

            const SCALE_TOLERANCE: f32 = 0.1;
            const POSITION_TOLERANCE: f32 = 0.1;

            let cache =
                conrod::text::GlyphCache::new(width, height, SCALE_TOLERANCE, POSITION_TOLERANCE);

            let data = vec![0; (width * height * 4) as usize];

            let (texture, texture_view) = create_texture(&mut factory, width, height, &data);

            (cache, texture, texture_view)
        };

        // Event loop
        let mut events = window.poll_events();

        'main: loop {
            // If the window is closed, this will be None for one tick, so to avoid panicking with
            // unwrap, instead break the loop
            let (win_w, win_h) = match window.get_inner_size() {
                Some(s) => s,
                None => break 'main,
            };

            let dpi_factor = window.hidpi_factor();

            if let Some(event) = events.next() {
                let (w, h) = (win_w as conrod::Scalar, win_h as conrod::Scalar);
                let dpi_factor = dpi_factor as conrod::Scalar;

                // Convert winit event to conrod event, requires conrod to be built with the `winit` feature
                if let Some(event) = conrod::backend::winit::convert(event.clone(),
                                                                     window.as_winit_window()) {
                    ui.handle_event(event);
                }

                // Close window if the escape key or the exit button is pressed
                match event {
                    glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                    glutin::Event::Closed => break 'main,

                    _ => {}
                }
            }
        }
    }
}
