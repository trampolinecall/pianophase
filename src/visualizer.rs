use macroquad::{
    color::{self, Color},
    shapes::{draw_arc, draw_circle, draw_line, draw_rectangle, draw_rectangle_ex},
    text::{draw_text_ex, TextParams},
    window::clear_background,
};
use num_traits::{FloatConst, ToPrimitive};
use smufl::{Coord, Metadata, StaffSpaces};

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
        let current_time = timing.current_musical_time(music);

        clear_background(color::BLACK);

        let part1_segment_index = music.part1.find_current_segment(current_time);
        let part2_segment_index = music.part2.find_current_segment(current_time);

        let draw_wheel = |center_x: f32, center_y: f32, part: &Part, segment_index: Option<usize>| {
            let Some(segment_index) = segment_index else { return };
            let (segment, segment_start, segment_end) = &part.0[segment_index];

            let offset_in_segment = (current_time.to_f32().unwrap() - segment_start.to_f32().unwrap())
                / (segment_end.to_f32().unwrap() - segment_start.to_f32().unwrap());
            let offset_in_pattern =
                ((current_time.to_f32().unwrap() - segment_start.to_f32().unwrap()) / segment.single_pattern_duration().to_f32().unwrap()).fract();
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

            const NOTE_HORIZ_SPACE: f32 = 50.0;

            let staff_line_thickness =
                self.font.metadata.engraving_defaults.staff_line_thickness.unwrap_or(StaffSpaces(1.0 / 8.0)).0 as f32 * STAFF_SPACE as f32;
            let stem_thickness =
                self.font.metadata.engraving_defaults.stem_thickness.unwrap_or(StaffSpaces(3.0 / 25.0)).0 as f32 * STAFF_SPACE as f32;

            // staff lines
            for i in 0..5 {
                draw_line(
                    200.0,
                    600.0 + i as f32 * STAFF_SPACE as f32,
                    800.0,
                    600.0 + i as f32 * STAFF_SPACE as f32,
                    staff_line_thickness,
                    color::WHITE,
                );
            }

            enum Accidental {
                Natural,
                Sharp,
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

            let smufl_coord_to_tuple = |coord: Coord| (coord.x().0 as f32 * STAFF_SPACE as f32, -coord.y().0 as f32 * STAFF_SPACE as f32);

            // notes
            if let Some(part1_segment_index) = part1_segment_index {
                for (i, note) in music.part1.0[part1_segment_index].0.pattern.0.iter().enumerate() {
                    let notehead_x = 200.0 + i as f32 * NOTE_HORIZ_SPACE;
                    let notehead_origin = self
                        .font
                        .metadata
                        .anchors
                        .get(smufl::Glyph::NoteheadBlack)
                        .and_then(|anchors| anchors.notehead_origin)
                        .map(smufl_coord_to_tuple)
                        .unwrap_or((0.0, 0.0));
                    let (y_position, accidental) = pitch_to_y_position(*note);
                    draw_text_ex(
                        &smufl::Glyph::NoteheadBlack.codepoint().to_string(),
                        notehead_x - notehead_origin.0,
                        600.0 + y_position * STAFF_SPACE as f32 - notehead_origin.1,
                        TextParams {
                            font: Some(&self.font.font),
                            font_size: STAFF_HEIGHT as u16,
                            font_scale: 1.0,
                            font_scale_aspect: 1.0,
                            rotation: 0.0,
                            color: color::WHITE,
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
                            notehead_x - NOTE_HORIZ_SPACE * 0.3,
                            600.0 + y_position * STAFF_SPACE as f32,
                            TextParams {
                                font: Some(&self.font.font),
                                font_size: STAFF_HEIGHT as u16,
                                font_scale: 1.0,
                                font_scale_aspect: 1.0,
                                rotation: 0.0,
                                color: color::WHITE,
                            },
                        );
                    }
                    let stem_origin = self
                        .font
                        .metadata
                        .anchors
                        .get(smufl::Glyph::NoteheadBlack)
                        .and_then(|anchors| anchors.stem_up_se)
                        .map(smufl_coord_to_tuple)
                        .unwrap_or((0.0, 0.0));
                    draw_line(
                        notehead_x + stem_origin.0,
                        600.0 + y_position * STAFF_SPACE as f32 + stem_origin.1,
                        notehead_x + stem_origin.0,
                        600.0 - 3.0 * STAFF_SPACE as f32,
                        stem_thickness,
                        color::WHITE,
                    );
                }
            }

            if let Some(part2_segment_index) = part2_segment_index {
                for (i, note) in music.part2.0[part2_segment_index].0.pattern.0.iter().enumerate() {
                    let notehead_x = 200.0 + i as f32 * NOTE_HORIZ_SPACE;
                    let notehead_origin = self
                        .font
                        .metadata
                        .anchors
                        .get(smufl::Glyph::NoteheadBlack)
                        .and_then(|anchors| anchors.notehead_origin)
                        .map(smufl_coord_to_tuple)
                        .unwrap_or((0.0, 0.0));
                    let (y_position, accidental) = pitch_to_y_position(*note);
                    draw_text_ex(
                        &smufl::Glyph::NoteheadBlack.codepoint().to_string(),
                        notehead_x - notehead_origin.0,
                        600.0 + y_position * STAFF_SPACE as f32 - notehead_origin.1,
                        TextParams {
                            font: Some(&self.font.font),
                            font_size: STAFF_HEIGHT as u16,
                            font_scale: 1.0,
                            font_scale_aspect: 1.0,
                            rotation: 0.0,
                            color: color::WHITE,
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
                            notehead_x - NOTE_HORIZ_SPACE * 0.3,
                            600.0 + y_position * STAFF_SPACE as f32,
                            TextParams {
                                font: Some(&self.font.font),
                                font_size: STAFF_HEIGHT as u16,
                                font_scale: 1.0,
                                font_scale_aspect: 1.0,
                                rotation: 0.0,
                                color: color::WHITE,
                            },
                        );
                    }
                    let stem_origin = self
                        .font
                        .metadata
                        .anchors
                        .get(smufl::Glyph::NoteheadBlack)
                        .and_then(|anchors| anchors.stem_down_nw)
                        .map(smufl_coord_to_tuple)
                        .unwrap_or((0.0, 0.0));
                    draw_line(
                        notehead_x + stem_origin.0,
                        600.0 + y_position * STAFF_SPACE as f32 + stem_origin.1,
                        notehead_x + stem_origin.0,
                        600.0 + STAFF_HEIGHT as f32 + 3.0 * STAFF_SPACE as f32,
                        stem_thickness,
                        color::WHITE,
                    );
                }
            }
        }

        true
    }
}
