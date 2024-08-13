use macroquad::{
    math::{clamp, Rect},
    shapes::{draw_arc, draw_circle, draw_line, draw_rectangle},
    text::{draw_text_ex, Font, TextParams},
    window::{clear_background, screen_height, screen_width},
};
use num_rational::{Ratio, Rational32};
use num_traits::{FloatConst, ToPrimitive};

use crate::{
    music::{Part, PianoPhase, Segment},
    timing::Timing,
    util::{lerp, remap},
    visualizer::{
        colors::ChangeAlpha,
        notation::{Staff, StaffPosition, CLEF_OFFSET, CLEF_WIDTH, DYNAMICS_Y, REPEAT_WIDTH, STEM_ABOVE_Y, STEM_BELOW_Y},
    },
};

mod colors;
mod notation;

pub struct Visualizer {
    notation_font: notation::Font,
    text_font: Font,
}

impl Visualizer {
    pub async fn new() -> Result<Visualizer, Box<dyn std::error::Error>> {
        let text_font = macroquad::text::load_ttf_font("data/Besley/static/Besley-Regular.ttf").await?;
        Ok(Visualizer { notation_font: notation::Font::load_bravura().await?, text_font })
    }

    pub fn update(&mut self, timing: &Timing, music: &PianoPhase) {
        clear_background(colors::BACKGROUND_COLOR);

        let current_time = timing.current_musical_time(music);

        let part1_segment_index = music.part1.find_segment_for_time(current_time);
        let part2_segment_index = music.part2.find_segment_for_time(current_time);

        let screen_width = screen_width();
        let screen_height = screen_height();

        draw_status_text(&self.text_font, &self.notation_font, music, current_time, part1_segment_index, part2_segment_index);

        let wheel_radius = f32::min(screen_width * 0.5 * 0.4 * 0.8, screen_height * 0.5 * 0.4 * 0.8);
        if let Some(part1_segment_index) = part1_segment_index {
            draw_wheel(
                &self.notation_font,
                current_time,
                &music.part1.segments[part1_segment_index],
                screen_width * 0.25,
                screen_height * 0.3,
                wheel_radius,
            );
        }
        if let Some(part2_segment_index) = part2_segment_index {
            draw_wheel(
                &self.notation_font,
                current_time,
                &music.part2.segments[part2_segment_index],
                screen_width * 0.75,
                screen_height * 0.3,
                wheel_radius,
            );
        }

        draw_in_sync_staff(&self.notation_font, music, Rect::new(0.0, screen_height * 0.5, screen_width, screen_height * 0.5 * 0.333), current_time);
        draw_out_of_sync_staff(
            &self.notation_font,
            music,
            Rect::new(0.0, screen_height * (0.5 + 0.5 * 0.333), screen_width, screen_height * 0.5 * 0.667),
            current_time,
            part1_segment_index,
            part2_segment_index,
        );
    }
}

fn draw_status_text(
    text_font: &Font,
    notation_font: &notation::Font,
    music: &PianoPhase,
    current_time: f32,
    part1_segment_index: Option<usize>,
    part2_segment_index: Option<usize>,
) {
    const FONT_SIZE: u16 = 24;
    const LEFT_X: f32 = 10.0;

    let go = |segment: &Segment, part_name: &'static str, y_position: f32| {
        let status = if segment.speed != Ratio::ONE { "Phasing" } else { "Steady" };

        let bpm = music.tempo as f32 / 2.0 * segment.speed.to_f32().unwrap();

        let current_measure = segment.find_measure(current_time).number + 1;
        let measures_in_segment = segment.repetitions;

        let first_part_dims = draw_text_ex(
            &format!("{part_name}: {status} "),
            LEFT_X,
            y_position,
            TextParams { font: Some(text_font), font_size: FONT_SIZE, color: colors::FOREGROUND_COLOR, ..Default::default() },
        );
        let eigth_note_dims = draw_text_ex(
            &smufl::Glyph::MetNote8thUp.codepoint().to_string(),
            LEFT_X + first_part_dims.width,
            y_position,
            notation_font.make_text_params_with_size(FONT_SIZE, colors::FOREGROUND_COLOR),
        );
        draw_text_ex(
            &format!(" = {bpm:.1} ({current_measure}/{measures_in_segment})"),
            LEFT_X + first_part_dims.width + eigth_note_dims.width,
            y_position,
            TextParams { font: Some(text_font), font_size: FONT_SIZE, color: colors::FOREGROUND_COLOR, ..Default::default() },
        );
    };

    if let Some(part1_segment_index) = part1_segment_index {
        go(&music.part1.segments[part1_segment_index], "Piano 1", 30.0);
    }
    if let Some(part2_segment_index) = part2_segment_index {
        go(&music.part2.segments[part2_segment_index], "Piano 2", 60.0);
    }
}

fn draw_wheel(font: &notation::Font, current_time: f32, segment: &Segment, center_x: f32, center_y: f32, staff_outer_radius: f32) {
    let staff =
        Staff::new(font, StaffPosition::Circular { center_x, center_y, outer_radius: staff_outer_radius }, (staff_outer_radius * 0.15 / 4.0) as u16);
    staff.draw(colors::FOREGROUND_COLOR);

    let dot_radius = staff_outer_radius - STEM_BELOW_Y * staff.staff_space as f32 - 20.0;
    let spinner_radius = staff_outer_radius - STEM_BELOW_Y * staff.staff_space as f32 - 40.0;

    let offset_in_segment =
        (current_time - segment.start_time.to_f32().unwrap()) / (segment.end_time.to_f32().unwrap() - segment.start_time.to_f32().unwrap());
    let current_measure = segment.find_measure(current_time);
    let offset_in_measure = remap(current_time, current_measure.start_time.to_f32().unwrap(), current_measure.end_time.to_f32().unwrap(), 0.0, 1.0);
    let current_note_index = (offset_in_measure * segment.pattern.0.len() as f32).floor() as usize;
    let offset_in_measure_rounded = current_note_index as f32 / segment.pattern.0.len() as f32;

    let current_dynamic = segment.dynamic.interpolate(offset_in_segment);

    let thing_color = colors::FOREGROUND_COLOR.modify_a(|a| a * current_dynamic);
    let normal_note_color = thing_color;
    let highlighted_note_color = colors::IMPORTANT_FOREGROUND_COLOR.modify_a(|a| a * current_dynamic);
    let highlight_color = colors::HIGHLIGHT_COLOR.modify_a(|a| a * current_dynamic);

    let spinner_end_x = center_x + (offset_in_measure * f32::TAU() - f32::PI() / 2.0).cos() * spinner_radius;
    let spinner_end_y = center_y + (offset_in_measure * f32::TAU() - f32::PI() / 2.0).sin() * spinner_radius;
    draw_line(center_x, center_y, spinner_end_x, spinner_end_y, 5.0, thing_color);

    let dot_x = center_x + (offset_in_measure_rounded * f32::TAU() - f32::PI() / 2.0).cos() * dot_radius;
    let dot_y = center_y + (offset_in_measure_rounded * f32::TAU() - f32::PI() / 2.0).sin() * dot_radius;
    draw_circle(dot_x, dot_y, 2.7, thing_color);

    draw_arc(
        center_x,
        center_y,
        56,
        staff_outer_radius - staff.staff_height as f32,
        -90.0,
        staff.staff_height as f32,
        360.0 * offset_in_measure,
        highlight_color,
    );

    for (note_i, note) in segment.pattern.0.iter().enumerate() {
        let note_angle = remap(note_i as f32, 0.0, segment.pattern.0.len() as f32, 0.0, f32::TAU());

        // only the first and last notes draw beams to simplify things
        // we can't just draw to a fixed offset because that would draw the beam to a certain angle which doesn't account for the stem offset
        let (beam_left, beam_right) = if note_i == 0 || note_i == 1 {
            // draw a beam from the first note to the bottom of the staff
            (None, Some(f32::PI() - 1.0))
        } else if note_i == segment.pattern.0.len() - 1 || note_i == segment.pattern.0.len() - 2 {
            // draw a beam from the bottom of the staff to the last note
            (Some(f32::PI() - 1.0), None)
        } else {
            (None, None)
        };

        let stem_end_y = match note.hand {
            crate::music::Hand::Left => STEM_BELOW_Y,
            crate::music::Hand::Right => STEM_ABOVE_Y,
        };

        let note_color = if note_i == current_note_index { highlighted_note_color } else { normal_note_color };

        staff.draw_note(note_angle, note.pitch, note_color, normal_note_color, stem_end_y, 2, beam_left, beam_right)
    }
}

fn draw_in_sync_staff(font: &notation::Font, music: &PianoPhase, window: Rect, current_time: f32) {
    let base_time_segment_index = music.part1.find_segment_for_time(current_time);
    if let Some(base_time_segment_index) = base_time_segment_index {
        let window_length = music.part1.segments[base_time_segment_index].single_measure_duration();

        let staff_space = (window.w / 120.0) as u16;
        let note_horiz_space = 8.0;
        let staff_width = (window_length.to_f32().unwrap() * note_horiz_space + CLEF_OFFSET + CLEF_WIDTH) * staff_space as f32;
        let staff_left = window.x + window.w * 0.5 - staff_width * 0.5;
        let staff_top = window.y + window.h * 0.5 - staff_space as f32 * 2.0; // center the staff vertically

        let staff = Staff::new(font, StaffPosition::Straight { top: staff_top, left: staff_left, right: staff_left + staff_width }, staff_space);

        staff.draw(colors::FOREGROUND_COLOR);

        let draw_past_notes = |staff: &Staff, part: &Part, window_duration: Rational32, stem_end_y: f32| {
            let notes = part.find_note_range(
                |note| note.time.to_f32().unwrap() < (current_time - window_duration.to_f32().unwrap()),
                |note| note.time.to_f32().unwrap() <= current_time,
            );

            staff.draw_treble_clef(CLEF_OFFSET, colors::FOREGROUND_COLOR);
            let notes_left = CLEF_OFFSET + CLEF_WIDTH;

            for note in notes {
                // TODO: clean up this code
                let base_speed_segment = &music.part1.segments[music.part1.find_segment_for_time(note.time.to_f32().unwrap()).unwrap()];
                let base_speed_measure = base_speed_segment.find_measure(note.time.to_f32().unwrap());

                let remap_time_to_x = |time| {
                    remap(
                        time,
                        base_speed_measure.start_time.to_f32().unwrap(),
                        base_speed_measure.end_time.to_f32().unwrap(),
                        notes_left,
                        base_speed_segment.single_measure_duration().to_f32().unwrap() * note_horiz_space + notes_left,
                    )
                };

                let note_x = remap_time_to_x(note.time.to_f32().unwrap());

                let note_fade = clamp(
                    remap(
                        note.time.to_f32().unwrap(),
                        current_time - window_duration.to_f32().unwrap() * 0.75,
                        current_time - window_duration.to_f32().unwrap() * 0.25,
                        0.3,
                        1.0,
                    ),
                    0.3,
                    1.0,
                );

                let actual_segment = &part.segments[note.segment_index];
                let actual_measure = actual_segment.get_measure(note.measure_number);
                let notes_in_actual_measure = part.find_note_range(|n| n.time < actual_measure.start_time, |n| n.time < actual_measure.end_time);

                let left_beam_time = if note.time != notes_in_actual_measure[0].time { Some(note.time - Ratio::new(1, 2)) } else { None };
                let right_beam_time = if note.time != notes_in_actual_measure[notes_in_actual_measure.len() - 1].time {
                    Some(note.time + Ratio::new(1, 2))
                } else {
                    None
                };

                let left_beam_x = left_beam_time.map(|t| remap_time_to_x(t.to_f32().unwrap()));
                let right_beam_x = right_beam_time.map(|t| remap_time_to_x(t.to_f32().unwrap()));

                let note_color = if note.time == notes.last().unwrap().time { colors::IMPORTANT_FOREGROUND_COLOR } else { colors::FOREGROUND_COLOR }
                    .set_a(note.volume * note_fade);
                let beam_color = colors::FOREGROUND_COLOR.set_a(note.volume * note_fade);

                staff.draw_note(note_x, note.pitch, note_color, beam_color, stem_end_y, 2, left_beam_x, right_beam_x);
            }
        };

        draw_past_notes(&staff, &music.part1, window_length, STEM_ABOVE_Y);
        draw_past_notes(&staff, &music.part2, window_length, STEM_BELOW_Y);
    }
}

fn draw_out_of_sync_staff(
    font: &notation::Font,
    music: &PianoPhase,
    window: Rect,
    current_time: f32,
    part1_segment_index: Option<usize>,
    part2_segment_index: Option<usize>,
) {
    // TODO: this code was copied and pasted from draw_in_sync_staff and duplicates a lot of it
    // TODO: this code also duplicates a lot of draw_wheel
    let staff_space = (window.w / 120.0) as u16;
    let staff_1_top = window.y + window.h * 0.3 - staff_space as f32 * 2.0; // center the staff vertically
    let staff_2_top = window.y + window.h * 0.7 - staff_space as f32 * 2.0; // center the staff vertically

    let go = |segment: &Segment, staff_top: f32, hairpin_y: f32| {
        let note_horiz_space = 4.0;
        let staff_width =
            (segment.pattern.0.len() as f32 * note_horiz_space + CLEF_OFFSET + CLEF_WIDTH + REPEAT_WIDTH + REPEAT_WIDTH) * staff_space as f32;
        let staff_left = window.x + window.w * 0.5 - staff_width * 0.5;

        let staff = Staff::new(font, StaffPosition::Straight { top: staff_top, left: staff_left, right: staff_left + staff_width }, staff_space);

        let pattern_len = segment.pattern.0.len();

        let offset_in_segment =
            (current_time - segment.start_time.to_f32().unwrap()) / (segment.end_time.to_f32().unwrap() - segment.start_time.to_f32().unwrap());
        let current_measure = segment.find_measure(current_time);
        let offset_in_measure =
            remap(current_time, current_measure.start_time.to_f32().unwrap(), current_measure.end_time.to_f32().unwrap(), 0.0, 1.0);
        let current_note_index = (offset_in_measure * pattern_len as f32).floor() as usize;

        let current_dynamic = segment.dynamic.interpolate(offset_in_segment);

        let normal_note_color = colors::FOREGROUND_COLOR.modify_a(|a| a * current_dynamic);
        let highlighted_note_color = colors::IMPORTANT_FOREGROUND_COLOR.modify_a(|a| a * current_dynamic);
        let highlight_color = colors::HIGHLIGHT_COLOR.modify_a(|a| a * current_dynamic);

        staff.draw(colors::FOREGROUND_COLOR);

        staff.draw_treble_clef(CLEF_OFFSET, colors::FOREGROUND_COLOR);

        let notes_start_x = CLEF_OFFSET + CLEF_WIDTH + REPEAT_WIDTH;
        let last_note_x_position = notes_start_x + pattern_len as f32 * note_horiz_space;

        staff.draw_starting_repeat_sign(notes_start_x - REPEAT_WIDTH * 0.5, colors::FOREGROUND_COLOR);
        staff.draw_ending_repeat_sign(last_note_x_position + REPEAT_WIDTH * 0.5, colors::FOREGROUND_COLOR);

        draw_rectangle(
            staff_left + notes_start_x * staff.staff_space as f32,
            staff_top,
            lerp(0.0, last_note_x_position - notes_start_x, offset_in_measure) * staff.staff_space as f32,
            staff.staff_height as f32,
            highlight_color,
        );

        for (note_i, note) in segment.pattern.0.iter().enumerate() {
            let note_x = remap(note_i as f32, 0.0, pattern_len as f32, notes_start_x, last_note_x_position);

            // only the first and last notes draw beams to simplify things
            // we can't just draw to a fixed offset because that would draw the beam to a certain position which doesn't account for the stem offset
            let (beam_left, beam_right) = if note_i == 0 || note_i == 1 {
                // draw a beam from the first note to the middle of the staff
                (None, Some(lerp(notes_start_x, last_note_x_position, 0.4)))
            } else if note_i == segment.pattern.0.len() - 1 || note_i == segment.pattern.0.len() - 2 {
                // draw a beam from the middle of the staff to the last note
                (Some(lerp(notes_start_x, last_note_x_position, 0.4)), None)
            } else {
                (None, None)
            };

            let stem_end_y = match note.hand {
                crate::music::Hand::Left => STEM_BELOW_Y,
                crate::music::Hand::Right => STEM_ABOVE_Y,
            };

            let note_color = if note_i == current_note_index { highlighted_note_color } else { normal_note_color };

            staff.draw_note(note_x, note.pitch, note_color, normal_note_color, stem_end_y, 2, beam_left, beam_right)
        }

        match segment.dynamic {
            crate::music::Dynamic::Crescendo => staff.draw_crescendo(hairpin_y, notes_start_x, last_note_x_position, colors::FOREGROUND_COLOR),
            crate::music::Dynamic::Decrescendo => staff.draw_decrescendo(hairpin_y, notes_start_x, last_note_x_position, colors::FOREGROUND_COLOR),
            crate::music::Dynamic::Flat | crate::music::Dynamic::Silent => {}
        }
    };

    if let Some(part1_segment_index) = part1_segment_index {
        go(&music.part1.segments[part1_segment_index], staff_1_top, DYNAMICS_Y);
    }
    if let Some(part2_segment_index) = part2_segment_index {
        go(&music.part2.segments[part2_segment_index], staff_2_top, DYNAMICS_Y);
    }
}
