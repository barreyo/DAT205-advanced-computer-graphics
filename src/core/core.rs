
mod core {

    use conrod;
    use conrod::backend::glium::glium;

    use glium::Surface;
    use glium::index::PrimitiveType;
    use glium::DisplayBuild;

    use alewife;

    use core::event;
    use support;
    use ui;

    const DEFAULT_WINDOW_WIDTH:  u32 = 1200;
    const DEFAULT_WINDOW_HEIGHT: u32 = 1000;

    pub fn init() {

        // Setup the message bus for core systems
        let mut bus = alewife::Publisher::<event::EventID, event::Event>::new();

        // Build the window.
        let display = glium::glutin::WindowBuilder::new()
            .with_vsync()
            .with_dimensions(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
            .with_title("TDA361 Advanced Graphics".to_owned())
            .build_glium()
            .unwrap();

    }
}
