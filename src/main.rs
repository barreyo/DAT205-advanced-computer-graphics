#[cfg(all(feature="winit", feature="glium"))]
#[macro_use]
extern crate conrod;
#[macro_use]
extern crate glium;
extern crate specs;

mod ui;
mod support;

fn main() {
    game::main();
}

#[cfg(all(feature="winit", feature="glium"))]
mod game {
    use conrod;
    use conrod::{widget, Colorable, Positionable, Widget};
    use conrod::widget::{Rectangle, TextBox};
    use conrod::Sizeable;
    use conrod::Labelable;
    use conrod::backend::glium::glium;

    use glium::Surface;
    use glium::index::PrimitiveType;
    use glium::DisplayBuild;

    use support;
    use ui;

    pub fn main() {
        const WIDTH: u32 = 1200;
        const HEIGHT: u32 = 1000;

        // Build the window.
        let display = glium::glutin::WindowBuilder::new()
            .with_vsync()
            .with_dimensions(WIDTH, HEIGHT)
            .with_title("Advanced Graphics")
            .build_glium()
            .unwrap();

        // construct our `Ui`.
        let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

        // Generate the widget identifiers.
        widget_ids!(struct Ids { canvas, bg, text, float, list, text_input});
        let ids = Ids::new(ui.widget_id_generator());

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

        // building the vertex buffer, which contains all the vertices that we will draw
        let vertex_buffer = {
            #[derive(Copy, Clone)]
            struct Vertex {
                position: [f32; 2],
                color: [f32; 3],
            }

            implement_vertex!(Vertex, position, color);

            glium::VertexBuffer::new(&display,
                &[
                    Vertex { position: [-0.5, -0.5], color: [0.0, 1.0, 0.0] },
                    Vertex { position: [ 0.0,  0.5], color: [0.0, 0.0, 1.0] },
                    Vertex { position: [ 0.5, -0.5], color: [1.0, 0.0, 0.0] },
                ]
            ).unwrap()
        };

        // compiling shaders and linking them together
        let program = program!(&display,
            140 => {
                vertex: "
                    #version 140
                    uniform mat4 matrix;
                    in vec2 position;
                    in vec3 color;
                    out vec3 vColor;
                    void main() {
                        gl_Position = vec4(position, 0.0, 1.0) * matrix;
                        vColor = color;
                    }
                ",

                fragment: "
                    #version 140
                    in vec3 vColor;
                    out vec4 f_color;
                    void main() {
                        f_color = vec4(vColor, 1.0);
                    }
                "
            },

            110 => {
                vertex: "
                    #version 110
                    uniform mat4 matrix;
                    attribute vec2 position;
                    attribute vec3 color;
                    varying vec3 vColor;
                    void main() {
                        gl_Position = vec4(position, 0.0, 1.0) * matrix;
                        vColor = color;
                    }
                ",

                fragment: "
                    #version 110
                    varying vec3 vColor;
                    void main() {
                        gl_FragColor = vec4(vColor, 1.0);
                    }
                ",
            },

            100 => {
                vertex: "
                    #version 100
                    uniform lowp mat4 matrix;
                    attribute lowp vec2 position;
                    attribute lowp vec3 color;
                    varying lowp vec3 vColor;
                    void main() {
                        gl_Position = vec4(position, 0.0, 1.0) * matrix;
                        vColor = color;
                    }
                ",

                fragment: "
                    #version 100
                    varying lowp vec3 vColor;
                    void main() {
                        gl_FragColor = vec4(vColor, 1.0);
                    }
                ",
            },
        ).unwrap();

        // building the index buffer
        let index_buffer = glium::IndexBuffer::new(&display,
                                                PrimitiveType::TrianglesList,
                                                &[0u16, 1, 2])
                                                .unwrap();

        // let ref mut field_text = "Edit".to_owned();

        let mut console = ui::console::Console::new();

        // Poll events from the window.
        let mut frame_time = support::frame_clock::FrameClock::new();
        'main: loop {

            frame_time.tick();

            // Collect all pending events.
            let events: Vec<_> = display.poll_events().collect();

            // Handle all events.
            for event in events {

                // Use the `winit` backend feature to convert the winit event to a conrod one.
                if let Some(event) = conrod::backend::winit::convert(event.clone(), &display) {
                    ui.handle_event(event);
                }

                match event {
                    // Break from the loop upon `Escape`.
                    glium::glutin::Event::KeyboardInput(_, _, Some(glium::glutin::VirtualKeyCode::Escape)) |
                    glium::glutin::Event::Closed =>
                        break 'main,
                    glium::glutin::Event::KeyboardInput(_, _, Some(glium::glutin::VirtualKeyCode::E)) =>
                        console.add_entry("Hello this is an ERROR. OMFG.".to_string(), ui::console::ConsoleLogLevel::ERROR),
                    glium::glutin::Event::KeyboardInput(_, _, Some(glium::glutin::VirtualKeyCode::W)) =>
                        console.add_entry("Hello this is a warning!".to_string(), ui::console::ConsoleLogLevel::WARNING),
                    glium::glutin::Event::KeyboardInput(_, _, Some(glium::glutin::VirtualKeyCode::C)) =>
                        console.toggle_visible(),
                    _ => {},
                }
            }

            // Instantiate all widgets in the GUI.
            {
                let ui = &mut ui.set_widgets();

                Rectangle::fill_with([90.0, 20.0], conrod::Color::Rgba(0.0, 0.0, 0.0, 0.8))
                    .top_left_of(ui.window)
                    .set(ids.bg, ui);

                // "Hello World!" in the middle of the screen.
                widget::Text::new(format!("{} fps ({} ms)",
                        frame_time.get_fps(),
                        frame_time.get_last_frame_duration()).as_str())
                    .middle_of(ids.bg)
                    .color(conrod::color::WHITE)
                    .font_size(12)
                    .set(ids.text, ui);

                // TODO: Move this to a UIRenderable component and use ECS
                console.update(ui, &console_ids);
            }

            // Render the `Ui` and then display it on the screen.
            let primitives = ui.draw();
            let uniforms = uniform! {
                matrix: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0f32]
                ]
            };

            renderer.fill(&display, primitives, &image_map);
            let mut target = display.draw();
            target.clear_color(0.5, 0.3, 0.7, 1.0);
            target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &Default::default()).unwrap();
            renderer.draw(&display, &mut target, &image_map).unwrap();
            target.finish().unwrap();
        }
    }
}
