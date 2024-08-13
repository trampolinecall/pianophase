
use macroquad::color::Color;

macro_rules! color_constant {
    ($name:ident, $hex_value: literal) => {
        pub const $name: Color = Color::new(
            ($hex_value >> 24) as f32 / 255.0,
            ($hex_value >> 16 & 0xff) as f32 / 255.0,
            ($hex_value >> 8 & 0xff) as f32 / 255.0,
            ($hex_value & 0xff) as f32 / 255.0,
        );
    };
}

color_constant!(BACKGROUND_COLOR, 0xe7e4e5ffu32);
color_constant!(FOREGROUND_COLOR, 0x2b121cffu32);
color_constant!(HIGHLIGHT_COLOR, 0xc86aa190u32);
color_constant!(IMPORTANT_FOREGROUND_COLOR, 0xff0000ffu32);

pub trait ColorExt {
    fn set_r(&self, r: f32) -> Self;
    fn set_g(&self, g: f32) -> Self;
    fn set_b(&self, b: f32) -> Self;
    fn set_a(&self, a: f32) -> Self;

    fn modify_r(&self, f: impl FnOnce(f32) -> f32) -> Self;
    fn modify_g(&self, f: impl FnOnce(f32) -> f32) -> Self;
    fn modify_b(&self, f: impl FnOnce(f32) -> f32) -> Self;
    fn modify_a(&self, f: impl FnOnce(f32) -> f32) -> Self;
}

impl ColorExt for Color {
    fn set_r(&self, r: f32) -> Self {
        Color { r, ..*self }
    }
    fn set_g(&self, g: f32) -> Self {
        Color { g, ..*self }
    }
    fn set_b(&self, b: f32) -> Self {
        Color { b, ..*self }
    }
    fn set_a(&self, a: f32) -> Self {
        Color { a, ..*self }
    }

    fn modify_r(&self, f: impl FnOnce(f32) -> f32) -> Self {
        Color { r: f(self.r), ..*self }
    }
    fn modify_g(&self, f: impl FnOnce(f32) -> f32) -> Self {
        Color { g: f(self.g), ..*self }
    }
    fn modify_b(&self, f: impl FnOnce(f32) -> f32) -> Self {
        Color { b: f(self.b), ..*self }
    }
    fn modify_a(&self, f: impl FnOnce(f32) -> f32) -> Self {
        Color { a: f(self.a), ..*self }
    }
}
