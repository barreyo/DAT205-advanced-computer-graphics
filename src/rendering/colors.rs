
use conrod;

#[derive(Debug)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

// const LIGHT_BLUE:   Color = {};
// const DARK_BLUE:    Color = {};

const LIGHT_GREEN:  Color = Color {r: 0.925, g: 0.941, b: 0.943, a: 1.0};
// const DARK_GREEN:   Color = {};

// const BLUE_GRAY:    Color = {};
// const MIDNIGHT:     Color = {};

// const TURQUOISE:    Color = {};
// const GREENSEA:     Color = {};

// const LIGHT_PURPLE: Color = {};
// const DARK_PURPLE:  Color = {};

const LIGHT_RED:    Color = Color {r: 0.905, g: 0.298, b: 0.235, a: 1.0};
// const DARK_RED:     Color = {};

// const LIGHT_ORANGE: Color = {};
// const DARK_ORANGE:  Color = {};

const LIGHT_YELLOW: Color = Color {r: 0.943, g: 0.768, b: 0.059, a: 1.0};
// const DARK_YELLOW:  Color = {};

// const WHITE:        Color = {};
// const SILVER:       Color = {};

// const LIGHT_GRAY:   Color = {};
// const DARK_GRAY:    Color = {};

impl Color {
    pub fn toConrodColor(self) -> conrod::Color {
        conrod::Color::Rgba(self.r, self.g, self.b, self.a)
    }
}
