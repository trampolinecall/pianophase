use macroquad::math::Vec2;
use num_traits::Float;

pub fn lerp<T: Float>(a: T, b: T, t: T) -> T {
    a + (b - a) * t
}

pub fn remap<T: Float>(v: T, a1: T, b1: T, a2: T, b2: T) -> T {
    let normalized = (v - a1) / (b1 - a1);
    lerp(a2, b2, normalized)
}

pub fn circle_coord(center_x: f32, center_y: f32, radius: f32, angle: f32) -> Vec2 {
    Vec2::new(center_x + angle.cos() * radius, center_y + angle.sin() * radius)
}
