use std::cmp::Ordering;

use num_rational::{Ratio, Rational32};
use num_traits::ToPrimitive;

use crate::util::lerp;

pub struct PianoPhase {
    // tempo is bpm for 16th note
    pub tempo: u16,

    pub part1: Part,
    pub part2: Part,
}

pub struct Part {
    pub segments: Vec<Segment>,
    pub flattened: Vec<FlattenedNote>,
}
#[derive(Debug)]
pub struct Segment {
    pub pattern: Pattern,
    pub speed: Rational32,
    pub repetitions: u32,
    pub dynamic: Dynamic,

    pub start_time: Rational32,
    pub end_time: Rational32,
}
#[derive(Clone, Debug)]
pub struct Pattern(pub Vec<u8>);
#[derive(Debug, PartialEq, Eq)]
pub enum Dynamic {
    Crescendo,
    Decrescendo,
    Flat,
    Silent,
}

#[derive(Debug)]
pub struct FlattenedNote {
    pub pitch: u8,
    pub time: Rational32,
    pub length: Rational32,
    pub volume: f32,

    pub segment_index: usize,
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
    pub fn find_segment_for_time(&self, time: f32) -> Option<usize> {
        let result = self.segments.binary_search_by(|segment| {
            match (segment.start_time.to_f32().unwrap().partial_cmp(&time), segment.end_time.to_f32().unwrap().partial_cmp(&time)) {
                (Some(Ordering::Less), Some(Ordering::Less)) => Ordering::Less,
                (Some(Ordering::Less), Some(Ordering::Equal)) => Ordering::Less,
                (Some(Ordering::Less), Some(Ordering::Greater)) => Ordering::Equal,
                (Some(Ordering::Equal), Some(Ordering::Less)) => panic!("segment starts later than it ends"),
                (Some(Ordering::Equal), Some(Ordering::Equal)) => Ordering::Equal,
                (Some(Ordering::Equal), Some(Ordering::Greater)) => Ordering::Equal,
                (Some(Ordering::Greater), Some(Ordering::Less)) => panic!("segment starts later than it ends"),
                (Some(Ordering::Greater), Some(Ordering::Equal)) => panic!("segment starts later than it ends"),
                (Some(Ordering::Greater), Some(Ordering::Greater)) => Ordering::Greater,
                _ => panic!("comparison resulted in None in find_segment_for_time"),
            }
        });
        match result {
            Ok(index) => Some(index),
            Err(index) if index >= self.segments.len() => None,
            Err(index) => panic!("searching for segment for time {time} resulted in index {index}, segments {:?}", self.segments),
        }
    }

    pub fn find_note_range(&self, start: impl Fn(&FlattenedNote) -> bool, end: impl Fn(&FlattenedNote) -> bool) -> &[FlattenedNote] {
        let start_ind = self.flattened.partition_point(start);
        let end_ind = self.flattened.partition_point(end);
        &self.flattened[start_ind..end_ind]
    }
}

pub struct Measure {
    pub start_time: Rational32,
    pub end_time: Rational32,
    pub number: usize,
}
impl Segment {
    pub fn single_measure_duration(&self) -> Rational32 {
        Ratio::from_integer(self.pattern.0.len() as i32) / self.speed
    }
    pub fn find_measure(&self, time: f32) -> Measure {
        let repetition_number = ((time - self.start_time.to_f32().unwrap()) / self.single_measure_duration().to_f32().unwrap()).floor() as usize;
        let start_time = Ratio::from_integer(repetition_number as i32) * self.single_measure_duration() + self.start_time;
        let end_time = start_time + self.single_measure_duration();

        Measure { start_time, end_time, number: repetition_number }
    }
}

impl Dynamic {
    pub fn interpolate(&self, t: f32) -> f32 {
        match self {
            Dynamic::Crescendo => lerp(0.0, 1.0, t),
            Dynamic::Decrescendo => lerp(1.0, 0.0, t),
            Dynamic::Flat => 1.0,
            Dynamic::Silent => 0.0,
        }
    }
}

struct PartBuilder {
    pub segments: Vec<Segment>,
    pub flattened: Vec<FlattenedNote>,

    pub current_time: Rational32,
}
impl PartBuilder {
    fn new() -> PartBuilder {
        PartBuilder { segments: Vec::new(), flattened: Vec::new(), current_time: Ratio::ZERO }
    }

    fn add_segment(&mut self, pattern: Pattern, speed: Rational32, repetitions: u32, dynamic: Dynamic) {
        let segment_start_time = self.current_time;
        let segment_index = self.segments.len();
        let total_number_of_notes = pattern.0.len() as i32 * repetitions as i32;
        let mut note_index = 0;
        for _ in 0..repetitions {
            for note in &pattern.0 {
                if dynamic != Dynamic::Silent {
                    self.flattened.push(FlattenedNote {
                        pitch: *note,
                        time: self.current_time,
                        length: Ratio::ONE / speed,
                        volume: dynamic.interpolate(note_index as f32 / total_number_of_notes as f32),
                        segment_index,
                    });
                }

                self.current_time += Ratio::ONE / speed;
                note_index += 1;
            }
        }
        self.segments.push(Segment { pattern, speed, repetitions, dynamic, start_time: segment_start_time, end_time: self.current_time });
    }

    fn into_part(self) -> Part {
        Part { segments: self.segments, flattened: self.flattened }
    }
}
fn parts(shorten: bool) -> (Part, Part) {
    let mut parts = (PartBuilder::new(), PartBuilder::new());

    let add_part_1 = |parts: &mut (PartBuilder, PartBuilder), pattern, speed, repetitions, dynamic| {
        parts.0.add_segment(pattern, speed, repetitions, dynamic);
    };
    let add_part_2 = |parts: &mut (PartBuilder, PartBuilder), pattern, speed, repetitions, dynamic| {
        parts.1.add_segment(pattern, speed, repetitions, dynamic);
    };

    let part_1_fade_in = |parts: &mut (PartBuilder, PartBuilder), part1_pattern: Pattern, part2_pattern: Pattern, repetitions: u32| {
        let repetitions = if shorten { 1 } else { repetitions };

        add_part_1(parts, part1_pattern, Ratio::ONE, repetitions, Dynamic::Crescendo);
        add_part_2(parts, part2_pattern, Ratio::ONE, repetitions, Dynamic::Flat);
    };
    let part_1_fade_out = |parts: &mut (PartBuilder, PartBuilder), part1_pattern: Pattern, part2_pattern: Pattern, repetitions: u32| {
        let repetitions = if shorten { 1 } else { repetitions };

        add_part_1(parts, part1_pattern, Ratio::ONE, repetitions, Dynamic::Decrescendo);
        add_part_2(parts, part2_pattern, Ratio::ONE, repetitions, Dynamic::Flat);
    };
    let part_2_fade_in = |parts: &mut (PartBuilder, PartBuilder), part1_pattern: Pattern, part2_pattern: Pattern, repetitions: u32| {
        let repetitions = if shorten { 1 } else { repetitions };

        add_part_1(parts, part1_pattern, Ratio::ONE, repetitions, Dynamic::Flat);
        add_part_2(parts, part2_pattern, Ratio::ONE, repetitions, Dynamic::Crescendo);
    };
    let part_2_fade_out = |parts: &mut (PartBuilder, PartBuilder), part1_pattern: Pattern, part2_pattern: Pattern, repetitions: u32| {
        let repetitions = if shorten { 1 } else { repetitions };

        add_part_1(parts, part1_pattern, Ratio::ONE, repetitions, Dynamic::Flat);
        add_part_2(parts, part2_pattern, Ratio::ONE, repetitions, Dynamic::Decrescendo);
    };

    let part_1_alone = |parts: &mut (PartBuilder, PartBuilder), part1_pattern: Pattern, repetitions: u32| {
        let repetitions = if shorten { 1 } else { repetitions };

        add_part_1(parts, part1_pattern.clone(), Ratio::ONE, repetitions, Dynamic::Flat);
        add_part_2(parts, part1_pattern, Ratio::ONE, repetitions, Dynamic::Silent);
    };
    let part_2_alone = |parts: &mut (PartBuilder, PartBuilder), part2_pattern: Pattern, repetitions: u32| {
        let repetitions = if shorten { 1 } else { repetitions };

        add_part_1(parts, part2_pattern.clone(), Ratio::ONE, repetitions, Dynamic::Silent);
        add_part_2(parts, part2_pattern, Ratio::ONE, repetitions, Dynamic::Flat);
    };

    let parts_repeat = |parts: &mut (PartBuilder, PartBuilder), part1_pattern: Pattern, part2_pattern: Pattern, repetitions: u32| {
        let repetitions = if shorten { 1 } else { repetitions };

        add_part_1(parts, part1_pattern, Ratio::ONE, repetitions, Dynamic::Flat);
        add_part_2(parts, part2_pattern, Ratio::ONE, repetitions, Dynamic::Flat);
    };

    let part_2_phase = |parts: &mut (PartBuilder, PartBuilder), part1_pattern: Pattern, part2_pattern: Pattern, repetitions: u32| {
        let repetitions = if shorten { 1 } else { repetitions };
        assert_eq!(part1_pattern.0.len(), part2_pattern.0.len());
        let pattern_len = part1_pattern.0.len() as i32;
        // speed multiplifier = (pattern_len * repetitions) / (pattern_len * repetitions - 1)
        let speed_multiplier = Ratio::new(pattern_len * repetitions as i32, pattern_len * repetitions as i32 - 1);

        add_part_1(parts, part1_pattern, Ratio::ONE, repetitions, Dynamic::Flat);
        add_part_2(parts, part2_pattern, speed_multiplier, repetitions, Dynamic::Flat);
    };

    let part_2_catch_up = |parts: &mut (PartBuilder, PartBuilder), _: Pattern, part2_pattern: Pattern| {
        add_part_2(parts, part2_pattern, Ratio::ONE, 1, Dynamic::Flat);
    };

    (part_1_alone(&mut parts, pat1(), 8));
    (part_2_fade_in(&mut parts, pat1(), pat1(), 12));

    for _ in 0..11 {
        (part_2_phase(&mut parts, pat1(), pat1(), 8));
        (parts_repeat(&mut parts, pat1(), pat1(), 18));
    }
    (part_2_phase(&mut parts, pat1(), pat1(), 8));
    (part_2_catch_up(&mut parts, pat1(), pat1()));
    (part_2_fade_out(&mut parts, pat1(), pat1(), 8));

    (part_1_alone(&mut parts, pat1(), 6));

    (part_1_alone(&mut parts, pat2_1(), 6));
    (part_2_fade_in(&mut parts, pat2_1(), pat2_2(), 16));

    for _ in 0..7 {
        (part_2_phase(&mut parts, pat2_1(), pat2_2(), 12));
        (parts_repeat(&mut parts, pat2_1(), pat2_2(), 16));
    }
    (part_2_phase(&mut parts, pat2_1(), pat2_2(), 12));
    (part_2_catch_up(&mut parts, pat2_1(), pat2_2()));
    (part_1_fade_out(&mut parts, pat2_1(), pat2_2(), 16));

    (part_2_alone(&mut parts, pat2_2(), 8));

    (part_2_alone(&mut parts, pat2_into_3(), 1));
    (part_2_alone(&mut parts, pat3(), 16));
    (part_1_fade_in(&mut parts, pat3(), pat3(), 24));

    for _ in 0..3 {
        (part_2_phase(&mut parts, pat3(), pat3(), 18));
        (parts_repeat(&mut parts, pat3(), pat3(), 48));
    }
    part_2_phase(&mut parts, pat3(), pat3(), 18);
    part_2_catch_up(&mut parts, pat3(), pat3());
    parts_repeat(&mut parts, pat3(), pat3(), 48);

    (parts.0.into_part(), parts.1.into_part())
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
