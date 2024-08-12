use std::{fs::File, io::BufReader};

use macroquad::{
    color::Color,
    math::Vec2,
    shapes::{draw_arc, draw_circle_lines, draw_line},
    text::{draw_text_ex, TextParams},
};
use num_traits::FloatConst;
use smufl::{Coord, Metadata, StaffSpaces};

use crate::{util::circle_coord, visualizer::colors};

#[allow(clippy::manual_non_exhaustive)]
pub struct Font {
    pub font: macroquad::text::Font,
    pub metadata: smufl::Metadata,
    _dont_construct: (),
}

#[allow(clippy::manual_non_exhaustive)]
pub struct Staff<'font> {
    pub font: &'font Font,
    pub position: StaffPosition,
    pub staff_space: u16,
    pub staff_height: u16,
    _dont_construct: (),
}
pub enum StaffPosition {
    Straight { top: f32, left: f32, right: f32 },
    Circular { center_x: f32, center_y: f32, outer_radius: f32 },
}

impl Font {
    pub async fn load_bravura() -> Result<Font, Box<dyn std::error::Error>> {
        let file = File::open("data/bravura/redist/bravura_metadata.json")?;
        let reader = BufReader::new(file);
        let metadata = Metadata::from_reader(reader)?;
        let font = macroquad::text::load_ttf_font("data/bravura/redist/otf/Bravura.otf").await?;
        Ok(Font { font, metadata, _dont_construct: () })
    }

    pub fn make_text_params(&self, staff: &Staff, color: Color) -> TextParams {
        TextParams { font: Some(&self.font), font_size: staff.staff_height, font_scale: 1.0, font_scale_aspect: 1.0, rotation: 0.0, color }
    }
}

impl<'font> Staff<'font> {
    pub const fn new(font: &'font Font, staff_position: StaffPosition, staff_space: u16) -> Staff<'font> {
        Staff { font, position: staff_position, staff_space, staff_height: staff_space * 4, _dont_construct: () }
    }

    // for a circular staff, x is the angle around the circle
    pub fn calculate_position(&self, x: f32, y: f32) -> (Vec2, f32) {
        match self.position {
            StaffPosition::Straight { top, left, right: _ } => {
                (Vec2::new(left + x * self.staff_space as f32, top + y * self.staff_space as f32), 0.0)
            }
            StaffPosition::Circular { center_x, center_y, outer_radius } => {
                (circle_coord(center_x, center_y, radius_for_y(outer_radius, y, self.staff_space as f32), x - f32::PI() / 2.0), x)
            }
        }
    }

    pub fn draw(&self) {
        let line_thickness =
            self.font.metadata.engraving_defaults.staff_line_thickness.unwrap_or(StaffSpaces(1.0 / 8.0)).0 as f32 * self.staff_space as f32;
        match self.position {
            StaffPosition::Straight { top, left, right } => {
                for i in 0..5 {
                    let y = top + i as f32 * self.staff_space as f32;
                    draw_line(left, y, right, y, line_thickness, colors::FOREGROUND_COLOR);
                }
            }
            StaffPosition::Circular { center_x, center_y, outer_radius } => {
                for i in 0..5 {
                    draw_circle_lines(
                        center_x,
                        center_y,
                        outer_radius - i as f32 * self.staff_space as f32,
                        line_thickness,
                        colors::FOREGROUND_COLOR,
                    );
                }
            }
        }
    }

    pub fn draw_note(
        &self,
        x_coord_on_staff: f32,
        pitch: u8,
        color: Color,
        stem_end_y: f32,
        num_beams: u32,
        beam_left: Option<f32>,
        beam_right: Option<f32>,
    ) {
        pub enum Accidental {
            Natural,
            Sharp,
            #[allow(dead_code)]
            Flat,
        }
        let (y_coord_on_staff, accidental) = match pitch {
            64 => (4.0, Accidental::Natural),
            69 => (2.5, Accidental::Natural),
            66 => (3.5, Accidental::Sharp),
            71 => (2.0, Accidental::Natural),
            73 => (1.5, Accidental::Sharp),
            74 => (1.0, Accidental::Natural),
            76 => (0.5, Accidental::Natural),
            _ => unimplemented!("{} not implemented", pitch),
        };
        let notehead_origin =
            optional_coord_to_tuple(self.font.metadata.anchors.get(smufl::Glyph::NoteheadBlack).and_then(|anchors| anchors.notehead_origin));
        let stem_thickness =
            self.font.metadata.engraving_defaults.stem_thickness.unwrap_or(StaffSpaces(3.0 / 25.0)).0 as f32 * self.staff_space as f32;

        // drawing the notehead
        {
            let (notehead_drawn_position, rotation) =
                self.calculate_position(x_coord_on_staff - notehead_origin.x, y_coord_on_staff - notehead_origin.y);

            draw_text_ex(
                &smufl::Glyph::NoteheadBlack.codepoint().to_string(),
                notehead_drawn_position.x,
                notehead_drawn_position.y,
                TextParams { rotation, ..self.font.make_text_params(self, color) },
            );
        }

        // drawing the accidental
        if let Accidental::Sharp | Accidental::Flat = accidental {
            const ACCIDENTAL_SHIFT: StaffSpaces = StaffSpaces(1.5);
            // because the x position means the angle for circular staves, we need to actually calculate the angle if the accidental is shifted
            // left by 1.5 staff spaces because shifting left by 1.5 radians is not the desired behavior
            let accidental_x = match self.position {
                StaffPosition::Straight { top: _, left: _, right: _ } => x_coord_on_staff - ACCIDENTAL_SHIFT.0 as f32,
                StaffPosition::Circular { center_x: _, center_y: _, outer_radius } => {
                    x_coord_on_staff - d_staff_spaces_to_radians(outer_radius, self.staff_space as f32, ACCIDENTAL_SHIFT.0 as f32, y_coord_on_staff)
                }
            };

            let (accidental_position, accidental_rotation) = self.calculate_position(accidental_x, y_coord_on_staff);
            draw_text_ex(
                &match accidental {
                    Accidental::Natural => unreachable!(),
                    Accidental::Sharp => smufl::Glyph::AccidentalSharp,
                    Accidental::Flat => smufl::Glyph::AccidentalFlat,
                }
                .codepoint()
                .to_string(),
                accidental_position.x,
                accidental_position.y,
                TextParams { rotation: accidental_rotation, ..self.font.make_text_params(self, color) },
            );
        }

        // drawing the stem
        let stem_up = stem_end_y < y_coord_on_staff;
        let stem_origin = if stem_up {
            optional_coord_to_tuple(self.font.metadata.anchors.get(smufl::Glyph::NoteheadBlack).and_then(|anchors| anchors.stem_up_se))
        } else {
            optional_coord_to_tuple(self.font.metadata.anchors.get(smufl::Glyph::NoteheadBlack).and_then(|anchors| anchors.stem_down_nw))
        };
        let stem_x = match self.position {
            StaffPosition::Straight { top: _, left: _, right: _ } => x_coord_on_staff + stem_origin.x,
            StaffPosition::Circular { center_x: _, center_y: _, outer_radius } => {
                x_coord_on_staff + d_staff_spaces_to_radians(outer_radius, self.staff_space as f32, stem_origin.x, y_coord_on_staff)
            }
        };
        {
            let (stem_start_drawn_position, _) = self.calculate_position(stem_x, y_coord_on_staff + stem_origin.y);
            let (stem_end_drawn_position, _) = self.calculate_position(stem_x, stem_end_y);

            draw_line(
                stem_start_drawn_position.x,
                stem_start_drawn_position.y,
                stem_end_drawn_position.x,
                stem_end_drawn_position.y,
                stem_thickness,
                color,
            );
        }

        // drawing the beam
        {
            let beam_bounds = match (beam_left, beam_right) {
                (None, None) => None,
                (None, Some(right)) => Some((stem_x, right)),
                (Some(left), None) => Some((left, stem_x)),
                (Some(left), Some(right)) => Some((left, right)),
            };

            if let Some((beam_left, beam_right)) = beam_bounds {
                let beam_thickness = self.font.metadata.engraving_defaults.beam_thickness.unwrap_or(StaffSpaces(0.5)).0 as f32;
                let beam_spacing = self.font.metadata.engraving_defaults.beam_spacing.unwrap_or(StaffSpaces(0.25)).0 as f32;

                let dy = (beam_thickness + beam_spacing) * if stem_up { 1.0 } else { -1.0 };
                let mut current_y = stem_end_y;
                for _ in 0..num_beams {
                    match self.position {
                        StaffPosition::Straight { top: _, left: _, right: _ } => {
                            let (beam_left_drawn_position, _) = self.calculate_position(beam_left, current_y);
                            let (beam_right_drawn_position, _) = self.calculate_position(beam_right, current_y);

                            draw_line(
                                beam_left_drawn_position.x,
                                beam_left_drawn_position.y,
                                beam_right_drawn_position.x,
                                beam_right_drawn_position.y,
                                beam_thickness * self.staff_space as f32,
                                color,
                            );
                        }
                        StaffPosition::Circular { center_x, center_y, outer_radius } => {
                            draw_arc(
                                center_x,
                                center_y,
                                48,
                                radius_for_y(outer_radius, current_y, self.staff_space as f32),
                                beam_left.to_degrees() - 90.0,
                                beam_thickness * self.staff_space as f32,
                                (beam_right - beam_left).to_degrees(),
                                color,
                            );
                        }
                    }

                    current_y += dy;
                }
            }
        }
    }
}

pub fn coord_to_tuple(coord: Coord) -> Vec2 {
    Vec2::new(coord.x().0 as f32, -coord.y().0 as f32)
}

pub fn optional_coord_to_tuple(coord: Option<Coord>) -> Vec2 {
    coord.map(coord_to_tuple).unwrap_or(Vec2::new(0.0, 0.0))
}

fn radius_for_y(outer_radius: f32, y: f32, staff_space: f32) -> f32 {
    outer_radius - y * staff_space
}

fn d_staff_spaces_to_radians(outer_radius: f32, staff_space: f32, d_staff_spaces: f32, y: f32) -> f32 {
    let circle_radius = radius_for_y(outer_radius, y, staff_space);
    d_staff_spaces * staff_space / circle_radius
}
