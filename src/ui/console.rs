
extern crate conrod;

use std::collections::VecDeque;

widget_ids!{
    pub struct ConsoleIds {
        container,
        bg,
        log,
        input
    }
}

#[derive(Debug)]
pub enum ConsoleLogLevel {
    INFO,
    WARNING,
    ERROR,
}

impl ConsoleLogLevel {
    fn value(&self) -> conrod::Color {
        match *self {
            ConsoleLogLevel::INFO => conrod::Color::Rgba(0.925, 0.941, 0.943, 1.0),
            ConsoleLogLevel::WARNING => conrod::Color::Rgba(0.943, 0.768, 0.059,1.0),
            ConsoleLogLevel::ERROR => conrod::Color::Rgba(0.905, 0.298, 0.235,1.0),
        }
    }
}

#[derive(Debug)]
pub struct ConsoleEntry {
    text: String,
    level: ConsoleLogLevel,
}

#[derive(Debug)]
pub struct Console {
    buffer: VecDeque<ConsoleEntry>,
    text_field_buffer: String,
    window_w: f64,
    window_h: f64,
    window_x: f64,
    window_y: f64,
    visible: bool,
}

impl Console {
    pub fn new() -> Console {
        Console {
            // TODO: Replace this with logger, use same buffer lol
            buffer: VecDeque::with_capacity(100),
            text_field_buffer: "Input".to_string(),
            window_w: 600.0,
            window_h: 400.0,
            window_x: 100.0,
            window_y: 100.0,
            visible: true
        }
    }

    pub fn toggle_visible(&mut self) {
        self.visible = !self.visible;
    }

    pub fn add_entry(&mut self, entry: String, level: ConsoleLogLevel) {
        if self.buffer.len() >= self.buffer.capacity() {
            self.buffer.pop_back();
        }
        let new_entry = ConsoleEntry{text: entry, level: level};
        self.buffer.push_front(new_entry);
    }

    pub fn update(&mut self, ui: &mut conrod::UiCell, ids: &ConsoleIds) {

        use conrod;
        use conrod::{widget, Colorable, Positionable, Widget};
        use conrod::widget::Rectangle;
        use conrod::widget::TextBox;
        use conrod::Labelable;
        use conrod::Sizeable;

        // Do not draw anything if not shown
        if !self.visible {
            return
        }

        let floating = widget::Canvas::new()
            .floating(true)
            .w_h(self.window_w, self.window_h)
            .label_color(conrod::color::WHITE);
        floating
            .middle_of(ui.window)
            .title_bar("Console")
            .color(conrod::color::CHARCOAL)
            .set(ids.container, ui);

        // Create background of the console window
        Rectangle::fill_with([300.0, 200.0], conrod::Color::Rgba(0.0, 0.0, 0.0, 0.8))
            .w_h(self.window_w, self.window_h - 26.0)
            .mid_bottom_of(ids.container)
            .set(ids.bg, ui);

        // Create the list of entries in the console log.
        let (mut items, scrollbar) = widget::List::new(self.buffer.len(), 20.0)
            .scrollbar_on_top()
            .middle_of(ids.bg)
            .w_h(self.window_w - 10.0, self.window_h - 30.0)
            .set(ids.log, ui);

        while let Some(item) = items.next(ui) {
            let i = item.i;
            if let Some(ev) = self.buffer.get(i as usize) {
                let label = format!("{}", ev.text);
                let e_string = widget::Text::new(label.as_str())
                                    .color(ev.level.value());
                item.set(e_string, ui);
            }
        }
        if let Some(s) = scrollbar {
            s.set(ui)
        }

        let title = self.text_field_buffer.clone();

        // Update and draw the input windows
        for edit in TextBox::new(title.as_str())
            .w_h(self.window_w, 30.0)
            .down_from(ids.container, 1.0)
            .set(ids.input, ui)
        {
            match edit {
                widget::text_box::Event::Enter => {
                    let current_str = self.text_field_buffer.clone().to_owned();
                    self.add_entry(current_str, ConsoleLogLevel::INFO);
                    self.text_field_buffer = "".to_string();
                },
                widget::text_box::Event::Update(string) => {
                    let s = string.clone().to_owned();
                    self.text_field_buffer = s;
                },
            }
        }
    }
}
