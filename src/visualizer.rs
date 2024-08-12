use macroquad::{
    color::{self, Color},
    shapes::{draw_arc, draw_circle, draw_line},
    text::{draw_text_ex, TextParams},
    window::clear_background,
};
use num_traits::{FloatConst, ToPrimitive};
use smufl::{Coord, StaffSpaces};

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
    pub async fn new() -> Result<Visualizer, Box<dyn std::error::Error>> {
        Ok(Visualizer { font: Font::load_bravura().await? })
    }

    pub fn update(&mut self, timing: &Timing, music: &PianoPhase) -> bool {
        clear_background(color::BLACK);

        let current_time = timing.current_musical_time(music);

        let part1_segment_index = music.part1.find_current_segment(current_time);
        let part2_segment_index = music.part2.find_current_segment(current_time);

        let draw_wheel = |center_x: f32, center_y: f32, part: &Part, segment_index: Option<usize>| {
            let Some(segment_index) = segment_index else { return };
            let segment = &part.segments[segment_index];

            let offset_in_segment = (current_time.to_f32().unwrap() - segment.start_time.to_f32().unwrap())
                / (segment.end_time.to_f32().unwrap() - segment.start_time.to_f32().unwrap());
            let offset_in_pattern =
                ((current_time.to_f32().unwrap() - segment.start_time.to_f32().unwrap()) / segment.single_pattern_duration().to_f32().unwrap()).fract();
            let offset_in_pattern_rounded = (offset_in_pattern * segment.pattern.0.len() as f32).floor() / segment.pattern.0.len() as f32;

            let current_dynamic = segment.dynamic.interpolate(offset_in_segment);

            let color = Color::from_rgba(255, 255, 255, (255.0 * current_dynamic) as u8);

            const SPINNER_LENGTH: f32 = 100.0;
            const ARC_RADIUS: f32 = 150.0;

            let spinner_end_x = center_x + (offset_in_pattern * f32::TAU() - f32::PI() / 2.0).cos() * SPINNER_LENGTH;
            let spinner_end_y = center_y + (offset_in_pattern * f32::TAU() - f32::PI() / 2.0).sin() * SPINNER_LENGTH;
            draw_line(center_x, center_y, spinner_end_x, spinner_end_y, 10.0, color);

            let dot_x = center_x + (offset_in_pattern_rounded * f32::TAU() - f32::PI() / 2.0).cos() * ((SPINNER_LENGTH + ARC_RADIUS) / 2.0);
            let dot_y = center_y + (offset_in_pattern_rounded * f32::TAU() - f32::PI() / 2.0).sin() * ((SPINNER_LENGTH + ARC_RADIUS) / 2.0);
            draw_circle(dot_x, dot_y, 7.0, color);

            draw_arc(center_x, center_y, 56, ARC_RADIUS, -90.0, 10.0, 360.0 * offset_in_pattern, color);
        };

        // TODO: adjust to window size
        draw_wheel(1280.0 * 0.25, 720.0 / 2.0, &music.part1, part1_segment_index);
        draw_wheel(1280.0 * 0.75, 720.0 / 2.0, &music.part2, part2_segment_index);

        {
            // drawing staff

            const STAFF_SPACE: u32 = 10;
            const STAFF_HEIGHT: u32 = STAFF_SPACE * 4;

            // TODO: eventually calculate these instead of hardcoding them
            const STAFF_LEFT: f32 = 200.0;
            const STAFF_RIGHT: f32 = 800.0;
            const STAFF_TOP_Y: f32 = 600.0;

            const NOTE_HORIZ_SPACE: f32 = 50.0;
            const ACCIDENTAL_SHIFT: f32 = 0.3;

            let staff_line_thickness =
                self.font.metadata.engraving_defaults.staff_line_thickness.unwrap_or(StaffSpaces(1.0 / 8.0)).0 as f32 * STAFF_SPACE as f32;
            let stem_thickness =
                self.font.metadata.engraving_defaults.stem_thickness.unwrap_or(StaffSpaces(3.0 / 25.0)).0 as f32 * STAFF_SPACE as f32;

            // staff lines
            for i in 0..5 {
                draw_line(
                    STAFF_LEFT,
                    STAFF_TOP_Y + i as f32 * STAFF_SPACE as f32,
                    STAFF_RIGHT,
                    STAFF_TOP_Y + i as f32 * STAFF_SPACE as f32,
                    staff_line_thickness,
                    color::WHITE,
                );
            }

            enum Accidental {
                Natural,
                Sharp,
                #[allow(dead_code)]
                Flat,
            }
            let pitch_to_y_position = |note| match note {
                64 => (4.0, Accidental::Natural),
                69 => (2.5, Accidental::Natural),
                66 => (3.5, Accidental::Sharp),
                71 => (2.0, Accidental::Natural),
                73 => (1.5, Accidental::Sharp),
                74 => (1.0, Accidental::Natural),
                76 => (0.5, Accidental::Natural),
                _ => unimplemented!("{} not implemented", note),
            };

            // TODO: make a helper module to deal with these
            let smufl_coord_to_tuple = |coord: Coord| (coord.x().0 as f32 * STAFF_SPACE as f32, -coord.y().0 as f32 * STAFF_SPACE as f32);

            let draw_note = |x_position: f32, pitch, stem_up, color| {
                let (y_position, accidental) = pitch_to_y_position(pitch);
                let notehead_origin = self
                    .font
                    .metadata
                    .anchors
                    .get(smufl::Glyph::NoteheadBlack)
                    .and_then(|anchors| anchors.notehead_origin)
                    .map(smufl_coord_to_tuple)
                    .unwrap_or((0.0, 0.0));

                draw_text_ex(
                    &smufl::Glyph::NoteheadBlack.codepoint().to_string(),
                    x_position - notehead_origin.0,
                    STAFF_TOP_Y + y_position * STAFF_SPACE as f32 - notehead_origin.1,
                    TextParams {
                        font: Some(&self.font.font),
                        font_size: STAFF_HEIGHT as u16,
                        font_scale: 1.0,
                        font_scale_aspect: 1.0,
                        rotation: 0.0,
                        color,
                    },
                );

                if let Accidental::Sharp | Accidental::Flat = accidental {
                    draw_text_ex(
                        &match accidental {
                            Accidental::Natural => unreachable!(),
                            Accidental::Sharp => smufl::Glyph::AccidentalSharp,
                            Accidental::Flat => smufl::Glyph::AccidentalFlat,
                        }
                        .codepoint()
                        .to_string(),
                        x_position - NOTE_HORIZ_SPACE * ACCIDENTAL_SHIFT,
                        STAFF_TOP_Y + y_position * STAFF_SPACE as f32,
                        TextParams {
                            font: Some(&self.font.font),
                            font_size: STAFF_HEIGHT as u16,
                            font_scale: 1.0,
                            font_scale_aspect: 1.0,
                            rotation: 0.0,
                            color,
                        },
                    );
                }

                let stem_origin = if stem_up {
                    self.font
                        .metadata
                        .anchors
                        .get(smufl::Glyph::NoteheadBlack)
                        .and_then(|anchors| anchors.stem_up_se)
                        .map(smufl_coord_to_tuple)
                        .unwrap_or((0.0, 0.0))
                } else {
                    self.font
                        .metadata
                        .anchors
                        .get(smufl::Glyph::NoteheadBlack)
                        .and_then(|anchors| anchors.stem_down_nw)
                        .map(smufl_coord_to_tuple)
                        .unwrap_or((0.0, 0.0))
                };

                let stem_end_y =
                    if stem_up { STAFF_TOP_Y - 3.0 * STAFF_SPACE as f32 } else { STAFF_TOP_Y + STAFF_HEIGHT as f32 + 3.0 * STAFF_SPACE as f32 };

                draw_line(
                    x_position - notehead_origin.0 + stem_origin.0,
                    STAFF_TOP_Y + y_position * STAFF_SPACE as f32 - notehead_origin.1 + stem_origin.1,
                    x_position - notehead_origin.0 + stem_origin.0,
                    stem_end_y,
                    stem_thickness,
                    color,
                );
            };

            // notes
            if let Some(part1_segment_index) = part1_segment_index {
                let segment = &music.part1.segments[part1_segment_index];
                let current_dynamic = segment.dynamic.interpolate(
                    (current_time.to_f32().unwrap() - segment.start_time.to_f32().unwrap())
                        / (segment.end_time.to_f32().unwrap() - segment.start_time.to_f32().unwrap()),
                );
                for (i, note) in segment.pattern.0.iter().enumerate() {
                    let notehead_x = STAFF_LEFT + i as f32 * NOTE_HORIZ_SPACE;
                    draw_note(notehead_x, *note, true, Color::from_rgba(255, 255, 255, (255.0 * current_dynamic) as u8));
                }
            }

            if let Some(part2_segment_index) = part2_segment_index {
                let segment = &music.part2.segments[part2_segment_index];
                let current_dynamic = segment.dynamic.interpolate(
                    (current_time.to_f32().unwrap() - segment.start_time.to_f32().unwrap())
                        / (segment.end_time.to_f32().unwrap() - segment.start_time.to_f32().unwrap()),
                );
                for (i, note) in segment.pattern.0.iter().enumerate() {
                    let notehead_x = STAFF_LEFT + i as f32 * NOTE_HORIZ_SPACE;
                    draw_note(notehead_x, *note, false, Color::from_rgba(255, 255, 255, (255.0 * current_dynamic) as u8));
                }
            }
        }

        true
    }
}
