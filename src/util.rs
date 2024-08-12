use num_traits::Float;

pub fn lerp<T: Float>(a: T, b: T, t: T) -> T {
    a + (b - a) * t
}

pub fn remap<T: Float>(v: T, a1: T, b1: T, a2: T, b2: T) -> T {
    let normalized = (v - a1) / (b1 - a1);
    lerp(a2, b2, normalized)
}
