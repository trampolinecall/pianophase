use macroquad::{
    color::{self, Color},
    shapes::{draw_arc, draw_circle, draw_line},
    text::draw_text_ex,
    window::clear_background,
};
use num_rational::Rational32;
use num_traits::{FloatConst, ToPrimitive};
use smufl::StaffSpaces;

use crate::{
    music::{Part, PianoPhase, Segment},
    timing::Timing,
    util::{lerp, remap},
    visualizer::{
        colors::ColorExt,
        notation::{Font, StaffParams},
    },
};

mod colors;
mod notation;

pub struct Visualizer {
    font: Font,
}

impl Visualizer {
    pub async fn new() -> Result<Visualizer, Box<dyn std::error::Error>> {
        Ok(Visualizer { font: Font::load_bravura().await? })
    }

    pub fn update(&mut self, timing: &Timing, music: &PianoPhase) -> bool {
        clear_background(colors::BACKGROUND_COLOR);

        let current_time = timing.current_musical_time(music);

        let part1_segment_index = music.part1.find_segment_for_time(current_time);
        let part2_segment_index = music.part2.find_segment_for_time(current_time);

        const SPINNER_LENGTH: f32 = 100.0;
        const ARC_RADIUS: f32 = 150.0;
        // TODO: adjust to window size
        if let Some(part1_segment_index) = part1_segment_index {
            draw_wheel(current_time, &music.part1.segments[part1_segment_index], 1280.0 * 0.25, 720.0 / 2.0, SPINNER_LENGTH, ARC_RADIUS);
        }
        if let Some(part2_segment_index) = part2_segment_index {
            draw_wheel(current_time, &music.part2.segments[part2_segment_index], 1280.0 * 0.75, 720.0 / 2.0, SPINNER_LENGTH, ARC_RADIUS);
        }

        draw_in_sync_staff(music, current_time, &self.font);

        true
    }
}

fn draw_wheel(current_time: f32, segment: &Segment, center_x: f32, center_y: f32, spinner_radius: f32, arc_radius: f32) {
    let offset_in_segment =
        (current_time - segment.start_time.to_f32().unwrap()) / (segment.end_time.to_f32().unwrap() - segment.start_time.to_f32().unwrap());
    let current_measure = segment.find_measure(current_time);
    let offset_in_measure = remap(current_time, current_measure.start_time.to_f32().unwrap(), current_measure.end_time.to_f32().unwrap(), 0.0, 1.0);
    let offset_in_measure_rounded = (offset_in_measure * segment.pattern.0.len() as f32).floor() / segment.pattern.0.len() as f32;

    let current_dynamic = segment.dynamic.interpolate(offset_in_segment);

    let color = colors::FOREGROUND_COLOR.set_a(current_dynamic);

    let spinner_end_x = center_x + (offset_in_measure * f32::TAU() - f32::PI() / 2.0).cos() * spinner_radius;
    let spinner_end_y = center_y + (offset_in_measure * f32::TAU() - f32::PI() / 2.0).sin() * spinner_radius;
    draw_line(center_x, center_y, spinner_end_x, spinner_end_y, 10.0, color);

    let dot_x = center_x + (offset_in_measure_rounded * f32::TAU() - f32::PI() / 2.0).cos() * ((spinner_radius + arc_radius) / 2.0);
    let dot_y = center_y + (offset_in_measure_rounded * f32::TAU() - f32::PI() / 2.0).sin() * ((spinner_radius + arc_radius) / 2.0);
    draw_circle(dot_x, dot_y, 7.0, color);

    draw_arc(center_x, center_y, 56, arc_radius, -90.0, 10.0, 360.0 * offset_in_measure, color);
}

fn draw_in_sync_staff(music: &PianoPhase, current_time: f32, font: &Font) {
    const STAFF_PARAMS: StaffParams = StaffParams::new(10);
    // TODO: eventually calculate these instead of hardcoding them
    const STAFF_LEFT: f32 = 200.0;
    const STAFF_RIGHT: f32 = 800.0;
    const STAFF_1_TOP_Y: f32 = 600.0;
    const STAFF_2_TOP_Y: f32 = 700.0;

    // TODO: also calculate this
    const NOTE_HORIZ_SPACE: f32 = 50.0;

    draw_staff(font, &STAFF_PARAMS, STAFF_LEFT, STAFF_RIGHT, STAFF_1_TOP_Y);
    draw_staff(font, &STAFF_PARAMS, STAFF_LEFT, STAFF_RIGHT, STAFF_2_TOP_Y);

    // notes
    let draw_past_notes = |part: &Part, window_duration: Rational32, staff_top_y: f32, stem_end_y: f32, subsequent_beams_up: bool| {
        let notes = part.find_note_range(
            |note| note.time.to_f32().unwrap() < (current_time - window_duration.to_f32().unwrap()),
            |note| note.time.to_f32().unwrap() <= current_time,
        );

        for note in notes {
            // TODO: clean up this code
            let base_speed_segment = &music.part1.segments[music.part1.find_segment_for_time(note.time.to_f32().unwrap()).unwrap()];
            let base_speed_measure = base_speed_segment.find_measure(note.time.to_f32().unwrap());
            let note_x = remap(
                note.time.to_f32().unwrap(),
                base_speed_measure.start_time.to_f32().unwrap(),
                base_speed_measure.end_time.to_f32().unwrap(),
                STAFF_LEFT,
                STAFF_LEFT + base_speed_segment.single_measure_duration().to_f32().unwrap() * NOTE_HORIZ_SPACE,
            );

            let note_fade = remap(
                note.time.to_f32().unwrap(),
                current_time - window_duration.to_f32().unwrap() * 0.9,
                current_time - window_duration.to_f32().unwrap() * 0.6,
                0.0,
                1.0,
            );

            draw_note(font, &STAFF_PARAMS, staff_top_y, note_x, note.pitch, stem_end_y, colors::FOREGROUND_COLOR.set_a(note.volume * note_fade));

            let part_segment_index = part.find_segment_for_time(note.time.to_f32().unwrap());
            if let Some(part_segment_index) = part_segment_index {
                let part_segment = &part.segments[part_segment_index];
                let part_measure = part_segment.find_measure(note.time.to_f32().unwrap());

                // TODO: fix overalpping beams
                // TODO: fading is messed up because the same beam will be drawn multiple times
                // TODO: shift beam to the stem offset
                let beam_start_x = remap(
                    part_measure.start_time.to_f32().unwrap(),
                    base_speed_measure.start_time.to_f32().unwrap(),
                    base_speed_measure.end_time.to_f32().unwrap(),
                    STAFF_LEFT,
                    STAFF_LEFT + base_speed_segment.single_measure_duration().to_f32().unwrap() * NOTE_HORIZ_SPACE,
                );
                let beam_end_x = remap(
                    part_measure.end_time.to_f32().unwrap() - 1.0,
                    base_speed_measure.start_time.to_f32().unwrap(),
                    base_speed_measure.end_time.to_f32().unwrap(),
                    STAFF_LEFT,
                    STAFF_LEFT + base_speed_segment.single_measure_duration().to_f32().unwrap() * NOTE_HORIZ_SPACE,
                );

                draw_beam(
                    font,
                    &STAFF_PARAMS,
                    beam_start_x,
                    beam_end_x,
                    2,
                    stem_end_y,
                    subsequent_beams_up,
                    colors::FOREGROUND_COLOR.set_a(note.volume * note_fade),
                );
            }
        }
    };

    let part1_segment_index = music.part1.find_segment_for_time(current_time);
    if let Some(part1_segment_index) = part1_segment_index {
        let part1_segment_length = music.part1.segments[part1_segment_index].single_measure_duration();

        draw_past_notes(&music.part1, part1_segment_length, STAFF_1_TOP_Y, STAFF_1_TOP_Y - 3.0 * STAFF_PARAMS.staff_space as f32, false);
        draw_past_notes(
            &music.part2,
            part1_segment_length,
            STAFF_2_TOP_Y,
            STAFF_2_TOP_Y + STAFF_PARAMS.staff_height as f32 + 3.0 * STAFF_PARAMS.staff_space as f32,
            true,
        );
    }
}

fn draw_beam(font: &Font, staff_params: &StaffParams, start_x: f32, end_x: f32, num_beams: u16, y: f32, subsequent_beams_up: bool, color: Color) {
    let beam_thickness = font.metadata.engraving_defaults.beam_thickness.unwrap_or(StaffSpaces(0.5)).0 as f32 * staff_params.staff_space as f32;
    let beam_spacing = font.metadata.engraving_defaults.beam_spacing.unwrap_or(StaffSpaces(0.25)).0 as f32 * staff_params.staff_space as f32;

    let dy = (beam_thickness + beam_spacing) * if subsequent_beams_up { -1.0 } else { 1.0 };
    let mut current_y = y;
    for _ in 0..num_beams {
        draw_line(start_x, current_y, end_x, current_y, beam_thickness, color);
        current_y += dy;
    }
}

fn draw_staff(font: &Font, staff_params: &StaffParams, left: f32, right: f32, top: f32) {
    let line_thickness =
        font.metadata.engraving_defaults.staff_line_thickness.unwrap_or(StaffSpaces(1.0 / 8.0)).0 as f32 * staff_params.staff_space as f32;
    for i in 0..5 {
        draw_line(
            left,
            top + i as f32 * staff_params.staff_space as f32,
            right,
            top + i as f32 * staff_params.staff_space as f32,
            line_thickness,
            colors::FOREGROUND_COLOR,
        );
    }
}
fn draw_note(font: &Font, staff_params: &StaffParams, staff_top_y: f32, x_coord: f32, pitch: u8, stem_end_y: f32, color: Color) {
    let stem_thickness =
        font.metadata.engraving_defaults.stem_thickness.unwrap_or(StaffSpaces(3.0 / 25.0)).0 as f32 * staff_params.staff_space as f32;

    enum Accidental {
        Natural,
        Sharp,
        #[allow(dead_code)]
        Flat,
    }
    let (y_position_staff_spaces, accidental) = match pitch {
        64 => (4.0, Accidental::Natural),
        69 => (2.5, Accidental::Natural),
        66 => (3.5, Accidental::Sharp),
        71 => (2.0, Accidental::Natural),
        73 => (1.5, Accidental::Sharp),
        74 => (1.0, Accidental::Natural),
        76 => (0.5, Accidental::Natural),
        _ => unimplemented!("{} not implemented", pitch),
    };
    let y_coord = staff_top_y + y_position_staff_spaces * staff_params.staff_space as f32;

    let notehead_origin = notation::optional_coord_to_tuple(
        font.metadata.anchors.get(smufl::Glyph::NoteheadBlack).and_then(|anchors| anchors.notehead_origin),
        staff_params.staff_space as f32,
    );

    draw_text_ex(
        &smufl::Glyph::NoteheadBlack.codepoint().to_string(),
        x_coord - notehead_origin.x,
        y_coord - notehead_origin.y,
        font.make_text_params(staff_params, color),
    );

    if let Accidental::Sharp | Accidental::Flat = accidental {
        const ACCIDENTAL_SHIFT: StaffSpaces = StaffSpaces(1.5);
        draw_text_ex(
            &match accidental {
                Accidental::Natural => unreachable!(),
                Accidental::Sharp => smufl::Glyph::AccidentalSharp,
                Accidental::Flat => smufl::Glyph::AccidentalFlat,
            }
            .codepoint()
            .to_string(),
            x_coord - ACCIDENTAL_SHIFT.0 as f32 * staff_params.staff_space as f32,
            y_coord,
            font.make_text_params(staff_params, color),
        );
    }

    let stem_up = stem_end_y < y_coord;

    let stem_origin = if stem_up {
        notation::optional_coord_to_tuple(
            font.metadata.anchors.get(smufl::Glyph::NoteheadBlack).and_then(|anchors| anchors.stem_up_se),
            staff_params.staff_space as f32,
        )
    } else {
        notation::optional_coord_to_tuple(
            font.metadata.anchors.get(smufl::Glyph::NoteheadBlack).and_then(|anchors| anchors.stem_down_nw),
            staff_params.staff_space as f32,
        )
    };

    draw_line(
        x_coord - notehead_origin.x + stem_origin.x,
        y_coord - notehead_origin.y + stem_origin.y,
        x_coord - notehead_origin.x + stem_origin.x,
        stem_end_y,
        stem_thickness,
        color,
    );
}
