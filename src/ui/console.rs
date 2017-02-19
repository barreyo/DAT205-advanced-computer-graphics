
use std::collections::VecDeque;
use conrod;
use alewife;

use core::event;

widget_ids!{
    pub struct ConsoleIds {
        container,
        bg,
        log,
        input
    }
}

#[derive(Debug, Clone)]
pub enum ConsoleLogLevel {
    INFO,
    WARNING,
    ERROR,
}

impl ConsoleLogLevel {
    fn value(&self) -> conrod::Color {
        match *self {
            ConsoleLogLevel::INFO    => conrod::Color::Rgba(0.925, 0.941, 0.943, 1.0),
            ConsoleLogLevel::WARNING => conrod::Color::Rgba(0.943, 0.768, 0.059, 1.0),
            ConsoleLogLevel::ERROR   => conrod::Color::Rgba(0.905, 0.298, 0.235, 1.0),
        }
    }
}

#[derive(Debug)]
pub struct ConsoleEntry {
    text: String,
    level: ConsoleLogLevel,
}

pub struct Console {
    buffer: VecDeque<ConsoleEntry>,
    text_field_buffer: String,
    publisher: alewife::Publisher<event::EventID, event::Event>,
    event_queue: alewife::Subscriber<event::EventID, event::Event>,
    window_w: f64,
    window_h: f64,
    window_x: f64,
    window_y: f64,
    font_size: u32,
    visible: bool,
}

impl Console {
    pub fn new(publisher: alewife::Publisher<event::EventID, event::Event>,
               e_que: alewife::Subscriber<event::EventID, event::Event>) -> Console {
        Console {
            // TODO: Replace this with logger, use same buffer lol
            buffer: VecDeque::with_capacity(100),
            text_field_buffer: "".to_string(),
            publisher: publisher,
            event_queue: e_que,
            window_w: 600.0,
            window_h: 400.0,
            window_x: 100.0,
            window_y: 100.0,
            font_size: 11,
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

        use core::event;

        let events: Vec<_> = self.event_queue.fetch();

        // No need to do this shit, since logger redirects to console.
        // Only first match is required.
        for event in events {
            match event {
                (_, event::Event::ConsoleMessage(msg, level)) => self.add_entry(msg, level),
                (_, event::Event::ReloadShaders)              => self.add_entry("Reloading shaders...".to_owned(), ConsoleLogLevel::INFO),
                (_, event::Event::ToggleWireframe)            => self.add_entry("Toggled wireframe mode...".to_owned(), ConsoleLogLevel::INFO),
                (_, event::Event::SetWindowSize(w, h))        => self.add_entry(format!("Setting window size to w: {} h: {}", w, h), ConsoleLogLevel::INFO),
                (_, event::Event::ToggleFullscreen)           => self.add_entry("Toggle Fullscreen".to_owned(), ConsoleLogLevel::INFO),
                (_, event::Event::ToggleVSync)                => self.add_entry("Toggle Vertical Sync".to_owned(), ConsoleLogLevel::INFO),
                (_, event::Event::MoveCamera(x, y))           => self.add_entry(format!("Moved Camera to x: {} y: {}", x, y), ConsoleLogLevel::INFO),
                (_, event::Event::ToggleConsole)              => {
                    self.add_entry("INFO: Toggle console visibility".to_owned(), ConsoleLogLevel::INFO);
                    self.toggle_visible();
                },
            }
        }

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
        let (mut items, scrollbar) = widget::List::new(self.buffer.len(), self.font_size * 1.5)
            .scrollbar_on_top()
            .middle_of(ids.bg)
            .w_h(self.window_w - 10.0, self.window_h - 30.0)
            .set(ids.log, ui);

        while let Some(item) = items.next(ui) {
            let i = item.i;
            if let Some(ev) = self.buffer.get(i as usize) {
                let label = format!("{}", ev.text);
                let e_string = widget::Text::new(label.as_str())
                                    .font_size(self.font_size)
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
