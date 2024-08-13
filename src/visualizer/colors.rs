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

color_constant!(BACKGROUND_COLOR, 0xf2f2f8ffu32);
color_constant!(FOREGROUND_COLOR, 0x0f0142ffu32);
color_constant!(HIGHLIGHT_COLOR, 0xac9eeb60u32);
color_constant!(IMPORTANT_FOREGROUND_COLOR, 0x8e00faffu32);

pub trait ChangeAlpha {
    fn set_a(&self, a: f32) -> Self;
    fn modify_a(&self, f: impl FnOnce(f32) -> f32) -> Self;
}

impl ChangeAlpha for Color {
    fn set_a(&self, a: f32) -> Self {
        Color { a, ..*self }
    }
    fn modify_a(&self, f: impl FnOnce(f32) -> f32) -> Self {
        Color { a: f(self.a), ..*self }
    }
}
