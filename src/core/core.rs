
mod core {

    use conrod;
    use conrod::backend::glium::glium;

    use glium::Surface;
    use glium::index::PrimitiveType;
    use glium::DisplayBuild;

    use alewife;
    use find_folder::Search;

    use core::event;
    use support;
    use ui;

    const DEFAULT_WINDOW_WIDTH:  u32 = 1200;
    const DEFAULT_WINDOW_HEIGHT: u32 = 1000;

    pub fn init() {

        // Setup the message bus for core systems
        let mut bus = alewife::Publisher::<event::EventID, event::Event>::new();

        let console_sub = bus.add_subscriber(&[event::EventID::UIEvent,
                                               event::EventID::RenderEvent,
                                               event::EventID::WindowEvent,
                                               event::EventID::EntityEvent]);
        let renderer_sub = bus.add_subscriber(&[event::EventID::RenderEvent]);

        // TODO: Create a REDO module, sub to some events and save them in buffer
        //       WHen invoked perform events in reverse. Events need to send state.
        //       CMD-z -> sends message to module to perform step.

        // Once we have built the message bus we can clone it to all
        // modules that wanna publish to it.
        let publisher = bus.build();

        let logger = support::logging::LogBuilder::new()
            .publisher(publisher.clone())
            .init();

        // Build the window.
        let display = glium::glutin::WindowBuilder::new()
            .with_vsync()
            .with_dimensions(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
            .with_title("TDA361 Advanced Graphics".to_owned())
            .build_glium()
            .unwrap();

        // Construct our `Ui`.
        let mut ui = conrod::UiBuilder::new(
            [DEFAULT_WINDOW_WIDTH  as f64,
             DEFAULT_WINDOW_HEIGHT as f64]).build();

        // Generate the widget identifiers.
        let debug_ids = ui::debug_info::DebugIds::new(ui.widget_id_generator());
        let console_ids = ui::console::ConsoleIds::new(ui.widget_id_generator());

        // Add a `Font` to the `Ui`'s `font::Map` from file.
        const FONT_PATH: &'static str =
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts/noto_sans_regular.ttf");
        ui.fonts.insert_from_file(FONT_PATH).unwrap();

        // A type used for converting `conrod::render::Primitives` into `Command`s that can be used
        // for drawing to the glium `Surface`.
        let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

        // The image map describing each of our widget->image mappings (in our case, none).
        let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

        let mut console = ui::console::Console::new(publisher.clone(), console_sub);
        let debug_info  = ui::debug_info::DebugInfo::new();
    }
}
