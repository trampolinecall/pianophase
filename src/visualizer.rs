use macroquad::{
    color::{self, Color},
    shapes::{draw_arc, draw_circle, draw_line, draw_rectangle, draw_rectangle_ex},
    window::clear_background,
};
use num_traits::{FloatConst, ToPrimitive};

use crate::{
    music::{Part, PianoPhase},
    timing::Timing,
    visualizer::notation::Font,
};

mod notation;

pub struct Visualizer {
    font: Font,
}

impl Visualizer {
    pub fn new() -> Result<Visualizer, Box<dyn std::error::Error>> {
        Ok(Visualizer { font: Font::load_bravura()? })
    }

    pub fn update(&mut self, timing: &Timing, music: &PianoPhase) -> bool {
        let current_time = timing.current_musical_time(music);

        clear_background(color::BLACK);

        let part1_segment_index = music.part1.find_current_segment(current_time);
        let part2_segment_index = music.part2.find_current_segment(current_time);

        let mut draw_wheel = |center_x: f32, center_y: f32, part: &Part, segment_index: Option<usize>| {
            let Some(segment_index) = segment_index else { return };
            let (segment, segment_start, segment_end) = &part.0[segment_index];

            let offset_in_segment = (current_time.to_f32().unwrap() - segment_start.to_f32().unwrap())
                / (segment_end.to_f32().unwrap() - segment_start.to_f32().unwrap());
            let offset_in_pattern =
                ((current_time.to_f32().unwrap() - segment_start.to_f32().unwrap()) / segment.single_pattern_duration().to_f32().unwrap()).fract();
            let offset_in_pattern_rounded = (offset_in_pattern * segment.pattern.0.len() as f32).floor() / segment.pattern.0.len() as f32;

            let current_dynamic = segment.dynamic.interpolate(offset_in_segment);

            let color = Color::from_rgba(255, 255, 255, (255.0 * current_dynamic) as u8);

            let spinner_end_x = center_x + (offset_in_pattern * f32::TAU()).cos() * 100.0;
            let spinner_end_y = center_y + (offset_in_pattern * f32::TAU()).sin() * 100.0;
            draw_line(center_x, center_y, spinner_end_x, spinner_end_y, 10.0, color);

            let dot_x = center_x + (offset_in_pattern_rounded * f32::TAU()).cos() * 120.0;
            let dot_y = center_y + (offset_in_pattern_rounded * f32::TAU()).sin() * 120.0;
            draw_circle(dot_x, dot_y, 7.0, color);

            draw_arc(center_x, center_y, 56, 150.0, 0.0, 10.0, 360.0 * offset_in_pattern, color);
        };

        draw_wheel(1280.0 * 0.25, 720.0 / 2.0, &music.part1, part1_segment_index);
        draw_wheel(1280.0 * 0.75, 720.0 / 2.0, &music.part2, part2_segment_index);

        true
    }
}
