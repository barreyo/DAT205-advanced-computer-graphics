
#[cfg(feature="winit")] #[macro_use] extern crate conrod;

use conrod;
use conrod::render;
use conrod::text::rt;

pub fn init(sc_width : f64, sc_height : f64) {
    let mut ui = conrod::UiBuilder::new([sc_width, sc_height]).build();

    // Generate the widget identifiers.
    widget_ids!(struct Ids { canvas, counter });
    let ids = Ids::new(ui.widget_id_generator());

    ui.fonts.insert_from_file("../assets/fonts/noto_sans_regular.ttf")
        .unwrap();
    
    let mut renderer = conrod::backend::glutin::convert()
    
}
