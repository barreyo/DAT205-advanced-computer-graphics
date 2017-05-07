


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

        let vs = gfx_app::shade::Source {
            glsl_120: include_bytes!("shader/terrain_120.glslv"),
            glsl_150: include_bytes!("shader/terrain_150.glslv"),
            hlsl_40: include_bytes!("data/vertex.fx"),
            msl_11: include_bytes!("shader/terrain_vertex.metal"),
            vulkan: include_bytes!("data/vert.spv"),
            ..gfx_app::shade::Source::empty()
        };
        let ps = gfx_app::shade::Source {
            glsl_120: include_bytes!("shader/terrain_120.glslf"),
            glsl_150: include_bytes!("shader/terrain_150.glslf"),
            hlsl_40: include_bytes!("data/pixel.fx"),
            msl_11: include_bytes!("shader/terrain_frag.metal"),
            vulkan: include_bytes!("data/frag.spv"),
            ..gfx_app::shade::Source::empty()
        };

        let rand_seed = rand::thread_rng().gen();
        let seed = Seed::new(rand_seed);
        let plane = Plane::subdivide(256, 256);
        let vertex_data: Vec<Vertex> = plane.shared_vertex_iter()
            .map(|(x, y)| {
                let h = perlin2(&seed, &[x, y]) * 32.0;
                Vertex {
                    pos: [25.0 * x, 25.0 * y, h],
                    color: calculate_color(h),
                }
            })
            .collect();

        let index_data: Vec<u32> = plane.indexed_polygon_iter()
            .triangulate()
            .vertices()
            .map(|i| i as u32)
            .collect();

        let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&vertex_data, &index_data[..]);

        App {
            pso: factory.create_pipeline_simple(vs.select(backend).unwrap(),
                                        ps.select(backend).unwrap(),
                                        pipe::new())
                .unwrap(),
            data: pipe::Data {
                vbuf: vbuf,
                locals: factory.create_constant_buffer(1),
                model: Matrix4::identity().into(),
                view: Matrix4::identity().into(),
                proj: cgmath::perspective(Deg(60.0f32), window_targets.aspect_ratio, 0.1, 1000.0)
                    .into(),
                out_color: window_targets.color,
                out_depth: window_targets.depth,
            },
            slice: slice,
            start_time: Instant::now(),
        }
    }

    fn render<C: gfx::CommandBuffer<R>>(&mut self, encoder: &mut gfx::Encoder<R, C>) {
        let elapsed = self.start_time.elapsed();
        let time = elapsed.as_secs() as f32 + elapsed.subsec_nanos() as f32 / 1000_000_000.0;
        let x = time.sin();
        let y = time.cos();
        let view = Matrix4::look_at(Point3::new(x * 32.0, y * 32.0, 16.0),
                                    Point3::new(0.0, 0.0, 0.0),
                                    Vector3::unit_z());

        self.data.view = view.into();
        let locals = Locals {
            model: self.data.model,
            view: self.data.view,
            proj: self.data.proj,
        };

        encoder.update_buffer(&self.data.locals, &[locals], 0).unwrap();
        encoder.clear(&self.data.out_color, [0.3, 0.3, 0.3, 1.0]);
        encoder.clear_depth(&self.data.out_depth, 1.0);
        encoder.draw(&self.slice, &self.pso, &self.data);
    }

    fn on_resize(&mut self, window_targets: gfx_app::WindowTargets<R>) {
        self.data.out_color = window_targets.color;
        self.data.out_depth = window_targets.depth;
        self.data.proj =
            cgmath::perspective(Deg(60.0f32), window_targets.aspect_ratio, 0.1, 1000.0).into();
    }
}
