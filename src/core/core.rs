
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
    use na::Point3;

    use rand::Rng;
    use rand;

    use alewife;
    use find_folder;

    use core::event;
    use support;
    use ui;
    use rendering;
    use rendering::colors;

    const DEFAULT_WINDOW_WIDTH: u32 = 1200;
    const DEFAULT_WINDOW_HEIGHT: u32 = 1000;

    pub type ColorFormat = gfx::format::Srgba8;
    type DepthFormat = gfx::format::DepthStencil;

    pub fn init() {

        // Setup the message bus for core systems
        let mut bus = alewife::Publisher::<event::EventID, event::Event>::new();

        let console_sub = bus.add_subscriber(&[event::EventID::UIEvent,
                                               event::EventID::RenderEvent,
                                               event::EventID::WindowEvent,
                                               event::EventID::EntityEvent]);
        let renderer_sub = bus.add_subscriber(&[event::EventID::RenderEvent]);
        let cam_sub = bus.add_subscriber(&[event::EventID::EntityEvent]);

        // TODO: Create a REDO module, sub to some events and save them in buffer
        //       When invoked perform events in reverse. Events need to send state.
        //       CMD-z -> sends message to module to perform step.

        // Once we have built the message bus we can clone it to all
        // modules that wanna publish to it.
        let publisher = bus.build();

        let mut cam = rendering::camera::Camera::new(1.7,
                                                     DEFAULT_WINDOW_HEIGHT as f32 /
                                                     DEFAULT_WINDOW_WIDTH as f32,
                                                     Point3::new(0.0, 0.0, 0.0),
                                                     cam_sub);
        cam.look_at(Point3::new(0.0, 0.0, 0.0), Point3::new(32.0, 32.0, 16.0));

        let logger = support::logging::LogBuilder::new()
            .with_publisher(publisher.clone())
            .init();

        // Builder for window
        let builder = glutin::WindowBuilder::new()
            .with_title("Advanced Computer Graphics")
            .with_dimensions(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT);

        // Initialize gfx things
        let (window, mut device, mut factory, main_color, mut main_depth) =
            gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);
        let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

        // Create Ui and Ids of widgets to instantiate
        let mut ui = conrod::UiBuilder::new([DEFAULT_WINDOW_WIDTH as f64,
                                             DEFAULT_WINDOW_HEIGHT as f64])
            .build();

        // Generate the widget identifiers.
        let debug_ids = ui::debug_info::DebugIds::new(ui.widget_id_generator());
        let console_ids = ui::console::ConsoleIds::new(ui.widget_id_generator());

        // Load font from file
        let assets = find_folder::Search::KidsThenParents(2, 4).for_folder("assets").unwrap();
        let font_path = assets.join("fonts/noto_sans_regular.ttf");
        ui.fonts.insert_from_file(font_path).unwrap();

        let mut console = ui::console::Console::new(publisher.clone(), console_sub);
        let debug_info = ui::debug_info::DebugInfo::new();

        // Create seed for terrain generation.
        let rand_seed = rand::thread_rng().gen();

        let dpi = window.hidpi_factor();
        let mut text_render = ui::text::TextRenderer::new(DEFAULT_WINDOW_WIDTH as f32,
                                                          DEFAULT_WINDOW_HEIGHT as f32,
                                                          dpi,
                                                          main_color.clone(),
                                                          &mut factory);

        //let teapot_input = BufReader::new(File::open("/Users/barre/Desktop/DAT205-advanced-computer-graphics/assets/models/teapot.obj").unwrap());
        //let teapot: Obj = load_obj(teapot_input).unwrap();

        /*let mut terrain = rendering::terrain::Terrain::new(512 as usize,
                                                           &mut factory,
                                                           main_color.clone(),
                                                           main_depth.clone());
*/
        let mut deferred_light_sys =
            rendering::deferred::DeferredLightSystem::new(renderer_sub,
                                                          &mut factory,
                                                          DEFAULT_WINDOW_WIDTH as u16,
                                                          DEFAULT_WINDOW_HEIGHT as u16,
                                                          &rand_seed,
                                                          main_color.clone());

        let mut frame_time = support::frame_clock::FrameClock::new();

        // Event loop
        let mut events = window.poll_events();

        'main: loop {

            // Update FPS timer
            frame_time.tick();

            // If the window is closed, this will be None for one tick, so to avoid panicking with
            // unwrap, instead break the loop
            let (win_w, win_h) = match window.get_inner_size() {
                Some(s) => s,
                None => break 'main,
            };

            if let Some(event) = events.next() {
                // Convert winit event to conrod event, requires conrod to be built with the `winit` feature
                if let Some(event) = conrod::backend::winit::convert(event.clone(),
                                                                     window.as_winit_window()) {
                    ui.handle_event(event);
                }
                cam.process_input(&event);

                // Close window if the escape key or the exit button is pressed
                match event {
                    glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                    glutin::Event::Closed => break 'main,
                    glutin::Event::Resized(_width, _height) => {
                        // gfx_window_glutin::update_views(&window, &mut data.out, &mut main_depth);
                    }

                    _ => {}
                }
            }

            cam.update();

            // Closure to update UI elements
            {
                let ui = &mut ui.set_widgets();

                debug_info.update(ui,
                                  &debug_ids,
                                  frame_time.get_fps(),
                                  frame_time.get_last_frame_duration(),
                                  cam.get_eye());
                // TODO: Move this to a UIRenderable component and use ECS
                console.update(ui, &console_ids);
            }

            let dpi_factor = window.hidpi_factor();

            {
                let mut primitives = ui.draw();

                let (screen_width, screen_height) = (win_w as f32 * dpi_factor,
                                                     win_h as f32 * dpi_factor);

                text_render.prepare_frame(dpi_factor, win_w as f32, win_h as f32);

                // Create vertices
                while let Some(render::Primitive { id, kind, scizzor, rect }) = primitives.next() {
                    match kind {
                        render::PrimitiveKind::Rectangle { color } => {}
                        render::PrimitiveKind::Polygon { color, points } => {}
                        render::PrimitiveKind::Lines { color, cap, thickness, points } => {}
                        render::PrimitiveKind::Image { image_id, color, source_rect } => {}
                        render::PrimitiveKind::Text { color, text, font_id } => {
                            text_render.add_text(color, text, font_id, &mut encoder);
                        }
                        render::PrimitiveKind::Other(_) => {}
                    }
                }
            }

            // Clear the window
            encoder.clear_depth(&main_depth, 1.0);
            encoder.clear(&main_color, colors::DARK_BLUE.into_with_a());

            //terrain.render(&mut encoder, cam.get_view_proj().into());
            deferred_light_sys.render((frame_time.elapsed() as f32) / 1000.0,
                                      &rand_seed,
                                      cam.get_eye(),
                                      &mut encoder,
                                      cam.get_view_proj().into());
            text_render.render(&mut encoder, &mut factory);

            // Display the results
            encoder.flush(&mut device);
            window.swap_buffers().unwrap();
            device.cleanup();
        }
    }
}
