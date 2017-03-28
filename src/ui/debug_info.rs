
#[macro_use()]
extern crate conrod;

use na::Point3;

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

    pub fn update(&self, ui: &mut conrod::UiCell, ids: &DebugIds,
                  fps: u64, ms: u64, cam_pos: Point3<f32>) {

        use conrod;
        use conrod::widget;
        use conrod::Positionable;
        use conrod::Colorable;
        use conrod::Widget;
        use conrod::widget::Rectangle;

        if !self.visible {
            return
        }

        Rectangle::fill_with([100.0, 40.0], conrod::Color::Rgba(0.0, 0.0, 0.0, 0.8))
            .top_left_of(ui.window)
            .set(ids.bg, ui);

        widget::Text::new(format!("{} fps ({} ms)\nx: {:.2} y: {:.2} z: {:.2}", fps, ms,
                                  cam_pos.x, cam_pos.y, cam_pos.z).as_str())
            .middle_of(ids.bg)
            .color(conrod::color::WHITE)
            .font_size(12)
            .set(ids.fps_text, ui);
    }
}

