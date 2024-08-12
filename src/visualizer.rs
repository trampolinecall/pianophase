use macroquad::{
    shapes::{draw_arc, draw_circle, draw_line},
    window::clear_background,
};
use num_rational::{Ratio, Rational32};
use num_traits::{FloatConst, ToPrimitive};

use crate::{
    music::{Part, PianoPhase, Segment},
    timing::Timing,
    util::remap,
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

        const SPINNER_LENGTH: f32 = 100.0;
        const ARC_RADIUS: f32 = 150.0;
        const STAFF_RADIUS: f32 = 250.0;
        // TODO: adjust to window size
        if let Some(part1_segment_index) = part1_segment_index {
            draw_wheel(
                &self.font,
                current_time,
                &music.part1.segments[part1_segment_index],
                1280.0 * 0.25,
                720.0 / 2.0,
                SPINNER_LENGTH,
                ARC_RADIUS,
                STAFF_RADIUS,
            );
        }
        if let Some(part2_segment_index) = part2_segment_index {
            draw_wheel(
                &self.font,
                current_time,
                &music.part2.segments[part2_segment_index],
                1280.0 * 0.75,
                720.0 / 2.0,
                SPINNER_LENGTH,
                ARC_RADIUS,
                STAFF_RADIUS,
            );
        }

        draw_in_sync_staff(&self.font, music, current_time);

        true
    }
}

fn draw_wheel(
    font: &Font,
    current_time: f32,
    segment: &Segment,
    center_x: f32,
    center_y: f32,
    spinner_radius: f32,
    arc_radius: f32,
    staff_outer_radius: f32,
) {
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

    let staff = Staff::new(font, StaffPosition::Circular { center_x, center_y, outer_radius: staff_outer_radius }, 10);
    staff.draw();
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

        staff.draw_note(note_angle, *note, colors::FOREGROUND_COLOR, -3.0, 2, beam_left, beam_right)
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

    top_staff.draw();
    bottom_staff.draw();

    // notes
    let draw_past_notes = |staff: &Staff, part: &Part, window_duration: Rational32, staff_top_y: f32, stem_end_y: f32, subsequent_beams_up: bool| {
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
                current_time - window_duration.to_f32().unwrap() * 0.9,
                current_time - window_duration.to_f32().unwrap() * 0.6,
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
        draw_past_notes(&top_staff, &music.part1, window_length, STAFF_1_TOP_Y, -3.0, false);
        draw_past_notes(&bottom_staff, &music.part2, window_length, STAFF_2_TOP_Y, 8.0, true);
    }
}
