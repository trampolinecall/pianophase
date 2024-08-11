use std::cmp::Ordering;

use num_rational::{Ratio, Rational32};

pub struct PianoPhase {
    // tempo is bpm for 16th note
    pub tempo: u16,

    pub part1: Part,
    pub part2: Part,
}

pub struct Part(pub Vec<(Segment, Rational32, Rational32)>, pub Vec<FlattenedNote>);
#[derive(Debug)]
pub struct Segment {
    pub pattern: Pattern,
    pub speed: Rational32,
    pub repetitions: u32,
    pub dynamic: Dynamic,
}
#[derive(Clone, Debug)]
pub struct Pattern(pub Vec<u8>);
#[derive(Debug)]
pub enum Dynamic {
    Crescendo,
    Decrescendo,
    Flat,
    Silent,
}

pub struct FlattenedNote {
    pub pitch: u8,
    pub time: Rational32,
    pub length: Rational32,
    pub velocity: i32,
}

impl PianoPhase {
    pub fn new(tempo: u16) -> Self {
        let (part1, part2) = parts(false);
        Self { tempo, part1, part2 }
    }
    pub fn new_shortened(tempo: u16) -> Self {
        let (part1, part2) = parts(true);
        Self { tempo, part1, part2 }
    }
}
impl Part {
    fn new(segments: Vec<Segment>) -> Self {
        let mut flattened_notes = Vec::new();
        let mut timed_segments = Vec::new();

        let mut time = Rational32::ZERO;
        for segment in segments {
            let segment_start_time = time;
            let mut cur_velocity = match segment.dynamic {
                Dynamic::Crescendo => Rational32::ZERO,
                Dynamic::Decrescendo => Rational32::from_integer(127),
                Dynamic::Flat => Rational32::from_integer(127),
                Dynamic::Silent => Rational32::ZERO,
            };
            let toatal_number_of_notes = segment.pattern.0.len() as i32 * segment.repetitions as i32;
            for _ in 0..segment.repetitions {
                for note in &segment.pattern.0 {
                    if cur_velocity != Rational32::ZERO {
                        flattened_notes.push(FlattenedNote {
                            pitch: *note,
                            time,
                            length: Rational32::ONE / segment.speed,
                            velocity: cur_velocity.to_integer(),
                        });
                    }
                    time += Rational32::ONE / segment.speed;

                    match segment.dynamic {
                        Dynamic::Crescendo => {
                            cur_velocity += Ratio::new(127, toatal_number_of_notes);
                        }
                        Dynamic::Decrescendo => {
                            cur_velocity -= Ratio::new(127, toatal_number_of_notes);
                        }
                        Dynamic::Flat => {}
                        Dynamic::Silent => {}
                    }
                }
            }
            timed_segments.push((segment, segment_start_time, time));
        }

        Self(timed_segments, flattened_notes)
    }

    pub fn find_current_segment(&self, time: Rational32) -> Option<usize> {
        let result = self.0.binary_search_by(|segment| match (segment.1.cmp(&time), segment.2.cmp(&time)) {
            (Ordering::Less, Ordering::Less) => Ordering::Less,
            (Ordering::Less, Ordering::Equal) => Ordering::Less,
            (Ordering::Less, Ordering::Greater) => Ordering::Equal,
            (Ordering::Equal, Ordering::Less) => panic!("segment starts later than it ends"),
            (Ordering::Equal, Ordering::Equal) => Ordering::Equal,
            (Ordering::Equal, Ordering::Greater) => Ordering::Equal,
            (Ordering::Greater, Ordering::Less) => panic!("segment starts later than it ends"),
            (Ordering::Greater, Ordering::Equal) => panic!("segment starts later than it ends"),
            (Ordering::Greater, Ordering::Greater) => Ordering::Greater,
        });
        match result {
            Ok(index) => Some(index),
            Err(index) if index >= self.0.len() => None,
            Err(index) => panic!("searching for segment for time {time} resulted in index {index}, segments {:?}", self.0),
        }
    }
}

impl Segment {
    pub(crate) fn single_pattern_duration(&self) -> Rational32 {
        Ratio::from_integer(self.pattern.0.len() as i32) / self.speed
    }
}

impl Dynamic {
    pub fn interpolate(&self, t: f32) -> f32 {
        fn lerp(a: f32, b: f32, t: f32) -> f32 {
            a + (b - a) * t
        }
        match self {
            Dynamic::Crescendo => lerp(0.0, 1.0, t),
            Dynamic::Decrescendo => lerp(1.0, 0.0, t),
            Dynamic::Flat => 1.0,
            Dynamic::Silent => 0.0,
        }
    }
}

fn parts(shorten: bool) -> (Part, Part) {
    let part_1_fade_in = |part1_pattern: Pattern, part2_pattern: Pattern, repetitions: u32| -> (Segment, Segment) {
        let repetitions = if shorten { 1 } else { repetitions };
        (
            Segment { pattern: part1_pattern, speed: Ratio::ONE, repetitions, dynamic: Dynamic::Crescendo },
            Segment { pattern: part2_pattern, speed: Ratio::ONE, repetitions, dynamic: Dynamic::Flat },
        )
    };
    let part_1_fade_out = |part1_pattern: Pattern, part2_pattern: Pattern, repetitions: u32| -> (Segment, Segment) {
        let repetitions = if shorten { 1 } else { repetitions };
        (
            Segment { pattern: part1_pattern, speed: Ratio::ONE, repetitions, dynamic: Dynamic::Decrescendo },
            Segment { pattern: part2_pattern, speed: Ratio::ONE, repetitions, dynamic: Dynamic::Flat },
        )
    };
    let part_2_fade_in = |part1_pattern: Pattern, part2_pattern: Pattern, repetitions: u32| -> (Segment, Segment) {
        let repetitions = if shorten { 1 } else { repetitions };
        (
            Segment { pattern: part1_pattern, speed: Ratio::ONE, repetitions, dynamic: Dynamic::Flat },
            Segment { pattern: part2_pattern, speed: Ratio::ONE, repetitions, dynamic: Dynamic::Crescendo },
        )
    };
    let part_2_fade_out = |part1_pattern: Pattern, part2_pattern: Pattern, repetitions: u32| -> (Segment, Segment) {
        let repetitions = if shorten { 1 } else { repetitions };
        (
            Segment { pattern: part1_pattern, speed: Ratio::ONE, repetitions, dynamic: Dynamic::Flat },
            Segment { pattern: part2_pattern, speed: Ratio::ONE, repetitions, dynamic: Dynamic::Decrescendo },
        )
    };

    let part_1_alone = |part1_pattern: Pattern, repetitions: u32| -> (Segment, Segment) {
        let repetitions = if shorten { 1 } else { repetitions };
        (
            Segment { pattern: part1_pattern.clone(), speed: Ratio::ONE, repetitions, dynamic: Dynamic::Flat },
            Segment { pattern: part1_pattern, speed: Ratio::ONE, repetitions, dynamic: Dynamic::Silent },
        )
    };
    let part_2_alone = |part2_pattern: Pattern, repetitions: u32| -> (Segment, Segment) {
        let repetitions = if shorten { 1 } else { repetitions };
        (
            Segment { pattern: part2_pattern.clone(), speed: Ratio::ONE, repetitions, dynamic: Dynamic::Silent },
            Segment { pattern: part2_pattern, speed: Ratio::ONE, repetitions, dynamic: Dynamic::Flat },
        )
    };

    let parts_repeat = |part1_pattern: Pattern, part2_pattern: Pattern, repetitions: u32| -> (Segment, Segment) {
        let repetitions = if shorten { 1 } else { repetitions };
        (
            Segment { pattern: part1_pattern, speed: Ratio::ONE, repetitions, dynamic: Dynamic::Flat },
            Segment { pattern: part2_pattern, speed: Ratio::ONE, repetitions, dynamic: Dynamic::Flat },
        )
    };

    let part_2_phase = |part1_pattern: Pattern, part2_pattern: Pattern, repetitions: u32| -> (Segment, Segment) {
        let repetitions = if shorten { 1 } else { repetitions };
        assert_eq!(part1_pattern.0.len(), part2_pattern.0.len());
        let pattern_len = part1_pattern.0.len() as i32;
        // speed multiplifier = (pattern_len * repetitions) / (pattern_len * repetitions - 1)
        let speed_multiplier = Ratio::new(pattern_len * repetitions as i32, pattern_len * repetitions as i32 - 1);
        (
            Segment { pattern: part1_pattern, speed: Ratio::ONE, repetitions, dynamic: Dynamic::Flat },
            Segment { pattern: part2_pattern, speed: speed_multiplier, repetitions, dynamic: Dynamic::Flat },
        )
    };

    let part_2_catch_up = |part1_pattern: Pattern, part2_pattern: Pattern| -> (Segment, Segment) {
        (
            Segment { pattern: part1_pattern, speed: Ratio::ONE, repetitions: 1, dynamic: Dynamic::Flat },
            Segment { pattern: part2_pattern, speed: Ratio::ONE, repetitions: 2, dynamic: Dynamic::Flat },
        )
    };

    let mut parts = Vec::new();
    parts.push(part_1_alone(pat1(), 8));
    parts.push(part_2_fade_in(pat1(), pat1(), 12));

    for _ in 0..11 {
        parts.push(part_2_phase(pat1(), pat1(), 8));
        parts.push(parts_repeat(pat1(), pat1(), 18));
    }
    parts.push(part_2_phase(pat1(), pat1(), 8));
    parts.push(part_2_catch_up(pat1(), pat1()));
    parts.push(part_2_fade_out(pat1(), pat1(), 8));

    parts.push(part_1_alone(pat1(), 6));

    parts.push(part_1_alone(pat2_1(), 6));
    parts.push(part_2_fade_in(pat2_1(), pat2_2(), 16));

    for _ in 0..7 {
        parts.push(part_2_phase(pat2_1(), pat2_2(), 12));
        parts.push(parts_repeat(pat2_1(), pat2_2(), 16));
    }
    parts.push(part_2_phase(pat2_1(), pat2_2(), 12));
    parts.push(part_2_catch_up(pat2_1(), pat2_2()));
    parts.push(part_1_fade_out(pat2_1(), pat2_2(), 16));

    parts.push(part_2_alone(pat2_2(), 8));

    parts.push(part_2_alone(pat2_into_3(), 1));
    parts.push(part_2_alone(pat3(), 16));
    parts.push(part_1_fade_in(pat3(), pat3(), 24));

    for _ in 0..3 {
        parts.push(part_2_phase(pat3(), pat3(), 18));
        parts.push(parts_repeat(pat3(), pat3(), 48));
    }
    parts.push(part_2_phase(pat3(), pat3(), 18));
    parts.push(part_2_catch_up(pat3(), pat3()));
    parts.push(parts_repeat(pat3(), pat3(), 48));

    let (part1, part2) = parts.into_iter().unzip();

    (Part::new(part1), Part::new(part2))
}

fn pat1() -> Pattern {
    Pattern(vec![64, 66, 71, 73, 74, 66, 64, 73, 71, 66, 74, 73])
}

fn pat2_1() -> Pattern {
    Pattern(vec![64, 66, 71, 73, 74, 66, 71, 73])
}

fn pat2_2() -> Pattern {
    Pattern(vec![64, 76, 69, 71, 74, 76, 69, 71])
}

fn pat2_into_3() -> Pattern {
    Pattern(vec![64, 76, 69, 71, 74, 76])
}

fn pat3() -> Pattern {
    Pattern(vec![69, 71, 74, 76])
}
