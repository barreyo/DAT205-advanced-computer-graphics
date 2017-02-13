
#[cfg(feature="winit")] #[macro_use] extern crate conrod;

extern crate find_folder;

/// Macro used to generate the ids
widget_ids! {
    pub struct Ids {
        canvas,
        bg,
        fps_text
    }
}

#[derive(Debug)]
pub struct DebugInfo {
    ui: conrod::Ui,
    renderer: conrod::backend::glium::Renderer,
    ids: Ids
}

impl DebugInfo {

    use conrod;
    use conrod::{widget, Colorable, Positionable, Widget};
    use conrod::backend::glium::glium;
    use conrod::backend::glium::glium::{DisplayBuild, Surface};
    use std;

    pub fn new(sc_width : f64,
               sc_height : f64,
               display : &glium::glutin::Window) -> DebugInfo {

        let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

        // Generate the widget identifiers.
        widget_ids!(struct Ids { canvas, bg, fps_text });
        let ui_ids = ;

        // Load default font
        ui.fonts.insert_from_file(get_font_path("noto_sans_regular"))
            .unwrap();

        let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

        DebugInfo {
            ui: conrod::UiBuilder::new([sc_width, sc_height]).build(),
            renderer: conrod::backend::glium::Renderer::new(display).unwrap(),
            ids: Ids::new(ui.widget_id_generator())
        }
    }

    pub fn update(&mut self, event : &glium::glutin:Event, display : &glium::glutin::Window) {
        // Use the `winit` backend feature to convert the winit event to a conrod one.
        if let Some(event) = conrod::backend::winit::convert(event.clone(), self.display) {
            ui.handle_event(event);
        }
    }

    pub fn render() {
        // Instantiate all widgets in the GUI.
        {
            let ui = &mut ui.set_widgets();

            // "Hello World!" in the middle of the screen.
            widget::Text::new("Hello World!")
                .middle_of(ui.window)
                .color(conrod::color::WHITE)
                .font_size(32)
                .set(ids.text, ui);
        }

        // Render the `Ui` and then display it on the screen.
        if let Some(primitives) = ui.draw_if_changed() {
            renderer.fill(&display, primitives, &image_map);
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            renderer.draw(&display, &mut target, &image_map).unwrap();
            target.finish().unwrap();
        }
    }

    fn get_font_path(font: &str) -> super::std::path::PathBuf {
        use find_folder::Search::KidsThenParents;

        let fonts_dir = KidsThenParents(3, 5)
                            .for_folder("fonts")
                            .expect("`fonts/` not found!");
        let font_path = fonts_dir.join(font);

        font.path.join(".ttf")
    }
}

