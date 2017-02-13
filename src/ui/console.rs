
extern crate conrod;
extern crate log_buffer;

// Generate a unique `WidgetId` for each widget.
widget_ids! {
    pub struct Ids {

        // Scrollbar
        canvas_scrollbar,



    }
}

#[derive(Debug)]
struct Console {
    buffer: log_buffer::LogBuffer,
    input:
}


pub fn update(&mut self, ui: &mut conrod::UiCell, ids: &Ids) {

}


