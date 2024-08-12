use macroquad::color::Color;

pub const BACKGROUND_COLOR: Color = Color::new(0.921, 0.918, 0.918, 1.0);
pub const FOREGROUND_COLOR: Color = Color::new(0.102, 0.031, 0.0, 1.0);

// pub const BACKGROUND_COLOR: Color = Color::new(0.0, 0.0, 0.0, 1.0);
// pub const FOREGROUND_COLOR: Color = Color::new(1.0, 1.0, 1.0, 1.0);

pub trait ColorExt {
    fn set_r(&self, r: f32) -> Self;
    fn set_g(&self, g: f32) -> Self;
    fn set_b(&self, b: f32) -> Self;
    fn set_a(&self, a: f32) -> Self;
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
}
