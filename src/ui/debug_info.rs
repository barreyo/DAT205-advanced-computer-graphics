
extern crate conrod;

/// Macro used to generate the ids
widget_ids! {
    pub struct DebugIds {
        bg,
        fps_text
    }
}

#[derive(Debug)]
pub struct DebugInfo {
    visible: bool
}

impl DebugInfo {

    pub fn new() -> DebugInfo {
        DebugInfo {
            visible: true
        }
    }

    pub fn update(&self, ui: &mut conrod::UiCell, ids: &DebugIds, fps : u64, ms : u64) {

        use conrod;
        use conrod::widget;
        use conrod::Positionable;
        use conrod::Colorable;
        use conrod::Widget;
        use conrod::widget::Rectangle;

        if !self.visible {
            return
        }

        Rectangle::fill_with([90.0, 20.0], conrod::Color::Rgba(0.0, 0.0, 0.0, 0.8))
            .top_left_of(ui.window)
            .set(ids.bg, ui);

        widget::Text::new(format!("{} fps ({} ms)", fps, ms).as_str())
            .middle_of(ids.bg)
            .color(conrod::color::WHITE)
            .font_size(12)
            .set(ids.fps_text, ui);
    }
}

