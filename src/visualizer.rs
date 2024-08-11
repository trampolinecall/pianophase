use num_traits::ToPrimitive;
use sfml::{
    graphics::{Color, RectangleShape, RenderTarget, RenderWindow, Shape, Transformable},
    window::{Event, Style},
};

use crate::{
    music::{Part, PianoPhase},
    timing::Timing,
};

pub struct Visualizer {
    window: RenderWindow,
}

impl Visualizer {
    pub fn new() -> Visualizer {
        let mut window = RenderWindow::new((1280, 720), "Piano Phase", Style::CLOSE, &Default::default());
        window.set_framerate_limit(60);

        Visualizer { window }
    }

    pub fn update(&mut self, timing: &Timing, music: &PianoPhase) -> bool {
        while let Some(event) = self.window.poll_event() {
            if event == Event::Closed {
                self.window.close();
                return false;
            }
        }
        self.window.set_active(true);

        let current_time = timing.current_musical_time(music);

        self.window.clear(Color::BLACK);

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
            let offset_in_pattern_rounded_interp = (offset_in_pattern * segment.pattern.0.len() as f32).fract();

            let current_dynamic = segment.dynamic.interpolate(offset_in_segment);

            let mut spinner = RectangleShape::new();
            spinner.set_size((100.0, 10.0));
            spinner.set_origin((5.0, 5.0));
            spinner.set_position((center_x, center_y));
            spinner.set_rotation(
                360.0 * (offset_in_pattern_rounded + offset_in_pattern_rounded_interp.powf(3.0) / segment.pattern.0.len() as f32) - 90.0,
            );

            spinner.set_fill_color(Color::rgba(255, 255, 255, (current_dynamic * 255.0) as u8));

            let mut edge_square = RectangleShape::new();
            edge_square.set_size((10.0, 10.0));
            edge_square.set_origin((-100.0, 5.0));
            edge_square.set_position((center_x, center_y));
            edge_square.set_rotation(360.0 * offset_in_pattern - 90.0);

            edge_square.set_fill_color(Color::rgba(255, 255, 255, (current_dynamic * 255.0) as u8));

            // TODO: draw circular staff

            self.window.draw(&spinner);
            self.window.draw(&edge_square);
        };

        draw_wheel(1280.0 * 0.25, 720.0 / 2.0, &music.part1, part1_segment_index);
        draw_wheel(1280.0 * 0.75, 720.0 / 2.0, &music.part2, part2_segment_index);

        self.window.display();

        true
    }
}
