
use conrod;

#[derive(Debug, Clone, Copy)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

pub const LIGHT_BLUE: Color = Color {
    r: 0.203,
    g: 0.59375,
    b: 0.85547,
    a: 1.0,
};
pub const DARK_BLUE: Color = Color {
    r: 0.1601,
    g: 0.5,
    b: 0.72265,
    a: 1.0,
};

pub const LIGHT_GREEN: Color = Color {
    r: 0.17968,
    g: 0.7968,
    b: 0.4414,
    a: 1.0,
};
// const DARK_GREEN:   Color = {};

// const BLUE_GRAY:    Color = {};
// const MIDNIGHT:     Color = {};

// const TURQUOISE:    Color = {};
// const GREENSEA:     Color = {};

// const LIGHT_PURPLE: Color = {};
// const DARK_PURPLE:  Color = {};

pub const LIGHT_RED: Color = Color {
    r: 0.905,
    g: 0.298,
    b: 0.235,
    a: 1.0,
};
// const DARK_RED:     Color = {};

// const LIGHT_ORANGE: Color = {};
// const DARK_ORANGE:  Color = {};

pub const LIGHT_YELLOW: Color = Color {
    r: 0.943,
    g: 0.768,
    b: 0.059,
    a: 1.0,
};
// const DARK_YELLOW:  Color = {};

pub const WHITE: Color = Color {
    r: 0.925,
    g: 0.941,
    b: 0.943,
    a: 1.0,
};
// const SILVER:       Color = {};

// const LIGHT_GRAY:   Color = {};
// const DARK_GRAY:    Color = {};

pub const BROWN: Color = Color {
    r: 0.2421,
    g: 0.1406,
    b: 0.1406,
    a: 1.0,
};

impl Color {
    pub fn to_conrod_color(self) -> conrod::Color {
        conrod::Color::Rgba(self.r, self.g, self.b, self.a)
    }

    pub fn into(self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }

    pub fn into_with_a(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}
