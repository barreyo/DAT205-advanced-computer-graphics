#[macro_use]
extern crate conrod;
extern crate glutin;
#[macro_use]
extern crate gfx;
extern crate gfx_core;
extern crate gfx_window_glutin;

extern crate genmesh;
extern crate noise;
extern crate rand;

extern crate obj;

#[macro_use]
extern crate lazy_static;
extern crate find_folder;

#[macro_use]
extern crate log;

extern crate approx; // For the macro relative_eq!
extern crate nalgebra as na;

extern crate alewife;

mod core;
// mod input;
mod rendering;
mod support;
mod ui;

fn main() {

    use core::core::core::init;

    init();
}
/*
#[cfg(all(feature="winit", feature="glium"))]
mod game {
    use conrod;
    use conrod::backend::glium::glium;

    use glium::Surface;
    use glium::index::PrimitiveType;
    use glium::DisplayBuild;

    use alewife;

    use core::event;
    use support;
    use ui;
    use rendering;

    use na::{Matrix4, Point3};
    use std::ops::Mul;

    pub fn main() {
        const WIDTH: u32 = 1200;
        const HEIGHT: u32 = 1000;

        // Setup the message bus for core systems
        let mut bus = alewife::Publisher::<event::EventID, event::Event>::new();

        let console_sub = bus.add_subscriber(&[event::EventID::UIEvent,
                                               event::EventID::RenderEvent,
                                               event::EventID::WindowEvent,
                                               event::EventID::EntityEvent]);

        let cam_sub = bus.add_subscriber(&[event::EventID::EntityEvent]);

        // Once we have built the message bus we can clone it to all
        // modules that wanna publish to it.
        let publisher = bus.build();

        let mut cam = rendering::camera::Camera::new(1.7,
                                                     HEIGHT as f32 / WIDTH as f32,
                                                     Point3::new(0.0, -3.0, 0.0),
                                                     cam_sub);
        cam.look_at(Point3::new(1.0, 1.0, 1.0), Point3::new(0.0, 0.0, 0.0));

        let logger = support::logging::LogBuilder::new()
            .with_publisher(publisher.clone())
            .init();

        // Build the window.
        let display = glium::glutin::WindowBuilder::new()
            .with_vsync()
            .with_dimensions(WIDTH, HEIGHT)
            .with_title("TDA361 Advanced Graphics")
            .build_glium()
            .unwrap();

        // Construct our `Ui`.
        let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

        // Generate the widget identifiers.
        let debug_ids = ui::debug_info::DebugIds::new(ui.widget_id_generator());
        let console_ids = ui::console::ConsoleIds::new(ui.widget_id_generator());

        // Add a `Font` to the `Ui`'s `font::Map` from file.
        const FONT_PATH: &'static str = concat!(env!("CARGO_MANIFEST_DIR"),
                                                "/assets/fonts/noto_sans_regular.ttf");
        ui.fonts.insert_from_file(FONT_PATH).unwrap();

        // A type used for converting `conrod::render::Primitives` into `Command`s that can be used
        // for drawing to the glium `Surface`.
        let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

        // The image map describing each of our widget->image mappings (in our case, none).
        let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

        // Building the vertex buffer, which contains all the vertices that we will draw
        let vertex_buffer = {
            #[derive(Copy, Clone)]
            struct Vertex {
                position: [f32; 2],
                color: [f32; 3],
            }

            implement_vertex!(Vertex, position, color);

            glium::VertexBuffer::new(&display,
                                     &[Vertex {
                                           position: [-0.5, -0.5],
                                           color: [0.0, 1.0, 0.0],
                                       },
                                       Vertex {
                                           position: [0.0, 0.5],
                                           color: [0.0, 0.0, 1.0],
                                       },
                                       Vertex {
                                           position: [0.5, -0.5],
                                           color: [1.0, 0.0, 0.0],
                                       }])
                .unwrap()
        };

        // compiling shaders and linking them together
        let program = program!(&display,
            140 => {
                vertex: "
                    #version 140
                    in vec2 position;
                    in vec3 color;
                    out vec3 vColor;
                    uniform mat4 modelViewProjMatrix;
                    void main() {
                        gl_Position = modelViewProjMatrix * vec4(position, 0.0, 1.0);
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
                    uniform mat4 modelViewProjMatrix;
                    attribute vec2 position;
                    attribute vec3 color;
                    varying vec3 vColor;
                    void main() {
                        gl_Position = vec4(position, 0.0, 1.0) * modelViewProjMatrix;
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
        )
            .unwrap();

        // building the index buffer
        let index_buffer =
            glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList, &[0u16, 1, 2]).unwrap();

        let mut console = ui::console::Console::new(publisher.clone(), console_sub);
        let debug_info = ui::debug_info::DebugInfo::new();

        // Poll events from the window.
        let mut frame_time = support::frame_clock::FrameClock::new();
        'main: loop {

            frame_time.tick();

            // Collect all pending events.
            let ref events: Vec<_> = display.poll_events().collect();

            // Handle all events.
            for event in events {

                // Use the `winit` backend feature to convert the winit event to a conrod one.
                if let Some(event) = conrod::backend::winit::convert(event.clone(), &display) {
                    ui.handle_event(event);
                }

                match *event {
                    // Break from the loop upon `Escape`.
                    glium::glutin::Event::KeyboardInput(_, _, Some(glium::glutin::VirtualKeyCode::Escape)) | glium::glutin::Event::Closed =>
                        break 'main,
                    glium::glutin::Event::KeyboardInput(glium::glutin::ElementState::Released, _, Some(glium::glutin::VirtualKeyCode::Key0)) => {
                        glium::glutin::WindowBuilder::new().with_fullscreen(glium::glutin::get_primary_monitor())
                                                    .rebuild_glium(&display).unwrap();
                    }
                    glium::glutin::Event::KeyboardInput(glium::glutin::ElementState::Released, _, Some(glium::glutin::VirtualKeyCode::Key2)) => {
                        warn!("Warning logged!");
                    }
                    glium::glutin::Event::KeyboardInput(glium::glutin::ElementState::Released, _, Some(glium::glutin::VirtualKeyCode::Comma)) => {
                        publisher.publish(event::EventID::UIEvent, event::Event::ToggleConsole);
                    }
                    _ => {}
                }
            }

            // Instantiate all widgets in the GUI.
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

            cam.update(&events);
            let model_matrix = Matrix4::new(1.0,
                                            0.0,
                                            0.0,
                                            0.0,
                                            0.0,
                                            1.0,
                                            0.0,
                                            0.0,
                                            0.0,
                                            0.0,
                                            1.0,
                                            0.0,
                                            0.0,
                                            0.0,
                                            0.0,
                                            1.0);

            // Render the `Ui` and then display it on the screen.
            let primitives = ui.draw();

            let mvp: [[f32; 4]; 4] = (cam.get_view_proj() * model_matrix).into();
            let uniforms = uniform! {
                modelViewProjMatrix: mvp
            };

            renderer.fill(&display, primitives, &image_map);
            let mut target = display.draw();
            target.clear_color(0.5, 0.3, 0.7, 1.0);
            target.draw(&vertex_buffer,
                      &index_buffer,
                      &program,
                      &uniforms,
                      &Default::default())
                .unwrap();
            renderer.draw(&display, &mut target, &image_map).unwrap();
            target.finish().unwrap();
        }
    }
}*/
