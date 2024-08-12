use macroquad::{
    shapes::{draw_arc, draw_circle, draw_line, draw_rectangle},
    window::clear_background,
};
use num_rational::{Ratio, Rational32};
use num_traits::{FloatConst, ToPrimitive};

use crate::{
    music::{Part, PianoPhase, Segment},
    timing::Timing,
    util::{lerp, remap},
    visualizer::{
        colors::ColorExt,
        notation::{Font, Staff, StaffPosition},
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

        const STAFF_RADIUS: f32 = 200.0;
        // TODO: adjust to window size
        if let Some(part1_segment_index) = part1_segment_index {
            draw_wheel(&self.font, current_time, &music.part1.segments[part1_segment_index], 1280.0 * 0.25, 720.0 / 2.0, STAFF_RADIUS);
        }
        if let Some(part2_segment_index) = part2_segment_index {
            draw_wheel(&self.font, current_time, &music.part2.segments[part2_segment_index], 1280.0 * 0.75, 720.0 / 2.0, STAFF_RADIUS);
        }

        draw_in_sync_staff(&self.font, music, current_time);
        draw_out_of_sync_staff(&self.font, music, current_time, part1_segment_index, part2_segment_index);

        true
    }
}

fn draw_wheel(font: &Font, current_time: f32, segment: &Segment, center_x: f32, center_y: f32, staff_outer_radius: f32) {
    let staff = Staff::new(font, StaffPosition::Circular { center_x, center_y, outer_radius: staff_outer_radius }, 10);
    staff.draw(colors::FOREGROUND_COLOR);

    let dot_radius = staff_outer_radius - staff.staff_height as f32 - 20.0;
    let spinner_radius = staff_outer_radius - staff.staff_height as f32 - 40.0;

    let offset_in_segment =
        (current_time - segment.start_time.to_f32().unwrap()) / (segment.end_time.to_f32().unwrap() - segment.start_time.to_f32().unwrap());
    let current_measure = segment.find_measure(current_time);
    let offset_in_measure = remap(current_time, current_measure.start_time.to_f32().unwrap(), current_measure.end_time.to_f32().unwrap(), 0.0, 1.0);
    let offset_in_measure_rounded = (offset_in_measure * segment.pattern.0.len() as f32).floor() / segment.pattern.0.len() as f32;

    let current_dynamic = segment.dynamic.interpolate(offset_in_segment);

    let foreground_color = colors::FOREGROUND_COLOR.modify_a(|a| a * current_dynamic);
    let highlight_color = colors::HIGHLIGHT_COLOR.modify_a(|a| a * current_dynamic);

    let spinner_end_x = center_x + (offset_in_measure * f32::TAU() - f32::PI() / 2.0).cos() * spinner_radius;
    let spinner_end_y = center_y + (offset_in_measure * f32::TAU() - f32::PI() / 2.0).sin() * spinner_radius;
    draw_line(center_x, center_y, spinner_end_x, spinner_end_y, 10.0, foreground_color);

    let dot_x = center_x + (offset_in_measure_rounded * f32::TAU() - f32::PI() / 2.0).cos() * dot_radius;
    let dot_y = center_y + (offset_in_measure_rounded * f32::TAU() - f32::PI() / 2.0).sin() * dot_radius;
    draw_circle(dot_x, dot_y, 7.0, foreground_color);

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
        let (beam_left, beam_right) = if note_i == 0 {
            // draw a beam from the first note to the bottom of the staff
            (None, Some(f32::PI()))
        } else if note_i == segment.pattern.0.len() - 1 {
            // draw a beam from the bottom of the staff to the last note
            (Some(f32::PI()), None)
        } else {
            (None, None)
        };

        staff.draw_note(note_angle, *note, foreground_color, -3.0, 2, beam_left, beam_right)
    }
}

fn draw_in_sync_staff(font: &Font, music: &PianoPhase, current_time: f32) {
    const STAFF_LEFT: f32 = 200.0;
    const STAFF_1_TOP_Y: f32 = 600.0;
    const STAFF_2_TOP_Y: f32 = 700.0;
    // TODO: calculate these positions instead of hardcoding them
    let top_staff = Staff::new(font, StaffPosition::Straight { top: STAFF_1_TOP_Y, left: STAFF_LEFT, right: 800.0 }, 10);
    let bottom_staff = Staff::new(font, StaffPosition::Straight { top: STAFF_2_TOP_Y, left: STAFF_LEFT, right: 800.0 }, 10);

    // TODO: also calculate this
    // this is measured in staff spaces now
    const NOTE_HORIZ_SPACE: f32 = 5.0;

    top_staff.draw(colors::FOREGROUND_COLOR);
    bottom_staff.draw(colors::FOREGROUND_COLOR);

    let draw_past_notes = |staff: &Staff, part: &Part, window_duration: Rational32, stem_end_y: f32| {
        let notes = part.find_note_range(
            |note| note.time.to_f32().unwrap() < (current_time - window_duration.to_f32().unwrap()),
            |note| note.time.to_f32().unwrap() <= current_time,
        );

        for note in notes {
            // TODO: clean up this code
            let base_speed_segment = &music.part1.segments[music.part1.find_segment_for_time(note.time.to_f32().unwrap()).unwrap()];
            let base_speed_measure = base_speed_segment.find_measure(note.time.to_f32().unwrap());

            let remap_time_to_x = |time| {
                remap(
                    time,
                    base_speed_measure.start_time.to_f32().unwrap(),
                    base_speed_measure.end_time.to_f32().unwrap(),
                    0.0,
                    base_speed_segment.single_measure_duration().to_f32().unwrap() * NOTE_HORIZ_SPACE,
                )
            };

            let note_x = remap_time_to_x(note.time.to_f32().unwrap());

            let note_fade = remap(
                note.time.to_f32().unwrap(),
                current_time - window_duration.to_f32().unwrap() * 0.8,
                current_time - window_duration.to_f32().unwrap() * 0.5,
                0.0,
                1.0,
            );

            let actual_segment = &part.segments[note.segment_index];
            let actual_measure = actual_segment.find_measure(note.time.to_f32().unwrap());
            let notes_in_actual_measure = part.find_note_range(|n| n.time < actual_measure.start_time, |n| n.time < actual_measure.end_time);

            let left_beam_time =
                if note.time != notes_in_actual_measure.first().expect("measure is empty").time { Some(note.time - Ratio::new(1, 2)) } else { None };
            let right_beam_time =
                if note.time != notes_in_actual_measure.last().expect("measure is empty").time { Some(note.time + Ratio::new(1, 2)) } else { None };

            let left_beam_x = left_beam_time.map(|t| remap_time_to_x(t.to_f32().unwrap()));
            let right_beam_x = right_beam_time.map(|t| remap_time_to_x(t.to_f32().unwrap()));

            staff.draw_note(note_x, note.pitch, colors::FOREGROUND_COLOR.set_a(note.volume * note_fade), stem_end_y, 2, left_beam_x, right_beam_x);
        }
    };

    let base_time_segment_index = music.part1.find_segment_for_time(current_time);
    if let Some(base_time_segment_index) = base_time_segment_index {
        let window_length = music.part1.segments[base_time_segment_index].single_measure_duration();
        draw_past_notes(&top_staff, &music.part1, window_length, -3.0);
        draw_past_notes(&bottom_staff, &music.part2, window_length, 8.0);
    }
}

fn draw_out_of_sync_staff(
    font: &Font,
    music: &PianoPhase,
    current_time: f32,
    part1_segment_index: Option<usize>,
    part2_segment_index: Option<usize>,
) {
    // TODO: this code was copied and pasted from draw_in_sync_staff and duplicates a lot of it

    const STAFF_LEFT: f32 = 200.0;
    const STAFF_1_TOP_Y: f32 = 800.0;
    const STAFF_2_TOP_Y: f32 = 900.0;

    // TODO: also calculate this
    // this is measured in staff spaces now
    const NOTE_HORIZ_SPACE: f32 = 5.0;

    // TODO: calculate these positions instead of hardcoding them
    let top_staff = Staff::new(font, StaffPosition::Straight { top: STAFF_1_TOP_Y, left: STAFF_LEFT, right: 800.0 }, 10);
    let bottom_staff = Staff::new(font, StaffPosition::Straight { top: STAFF_2_TOP_Y, left: STAFF_LEFT, right: 800.0 }, 10);

    let go = |segment: &Segment, staff: &Staff, staff_top: f32, stem_end_y: f32, hairpin_y: f32| {
        let offset_in_segment =
            (current_time - segment.start_time.to_f32().unwrap()) / (segment.end_time.to_f32().unwrap() - segment.start_time.to_f32().unwrap());
        let current_measure = segment.find_measure(current_time);
        let offset_in_measure =
            remap(current_time, current_measure.start_time.to_f32().unwrap(), current_measure.end_time.to_f32().unwrap(), 0.0, 1.0);

        let pattern_len = segment.pattern.0.len();

        let current_dynamic = segment.dynamic.interpolate(offset_in_segment);

        let foreground_color = colors::FOREGROUND_COLOR.modify_a(|a| a * current_dynamic);
        let highlight_color = colors::HIGHLIGHT_COLOR.modify_a(|a| a * current_dynamic);

        staff.draw(colors::FOREGROUND_COLOR);

        let last_note_x_position = pattern_len as f32 * NOTE_HORIZ_SPACE;
        for (note_i, note) in segment.pattern.0.iter().enumerate() {
            let note_x = remap(note_i as f32, 0.0, pattern_len as f32, 0.0, last_note_x_position);

            // only the first and last notes draw beams to simplify things
            // we can't just draw to a fixed offset because that would draw the beam to a certain position which doesn't account for the stem offset
            let (beam_left, beam_right) = if note_i == 0 {
                // draw a beam from the first note to the middle of the staff
                (None, Some(lerp(0.0, last_note_x_position, 0.5)))
            } else if note_i == segment.pattern.0.len() - 1 {
                // draw a beam from the middle of the staff to the last note
                (Some(lerp(0.0, last_note_x_position, 0.5)), None)
            } else {
                (None, None)
            };

            staff.draw_note(note_x, *note, foreground_color, stem_end_y, 2, beam_left, beam_right)
        }

        draw_rectangle(
            STAFF_LEFT,
            staff_top,
            lerp(0.0, last_note_x_position, offset_in_measure) * staff.staff_space as f32,
            staff.staff_height as f32,
            highlight_color,
        );

        match segment.dynamic {
            crate::music::Dynamic::Crescendo => staff.draw_crescendo(hairpin_y, 0.0, last_note_x_position, colors::FOREGROUND_COLOR),
            crate::music::Dynamic::Decrescendo => staff.draw_decrescendo(hairpin_y, 0.0, last_note_x_position, colors::FOREGROUND_COLOR),
            crate::music::Dynamic::Flat | crate::music::Dynamic::Silent => {}
        }
    };

    if let Some(part1_segment_index) = part1_segment_index {
        go(&music.part1.segments[part1_segment_index], &top_staff, STAFF_1_TOP_Y, -3.0, 9.0);
    }
    if let Some(part2_segment_index) = part2_segment_index {
        go(&music.part2.segments[part2_segment_index], &bottom_staff, STAFF_2_TOP_Y, 7.0, 9.0);
    }
}
