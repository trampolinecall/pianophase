use std::{
    fs::File,
    ops::{Div, Mul, Rem},
    path::{Path, PathBuf},
};

use macroquad::texture::get_screen_data;
use num_rational::{Ratio, Rational32};
use threadpool::ThreadPool;

use crate::music::{Part, PianoPhase};

pub struct Exporter {
    output_dir: PathBuf,
    current_frame: u32,
    thread_pool: ThreadPool,
    maximum_queue_size: usize,
}

impl Exporter {
    pub fn new(output_dir: PathBuf, num_export_threads: usize, maximum_queue_size: usize) -> Result<Exporter, Box<dyn std::error::Error>> {
        if !output_dir.exists() {
            std::fs::create_dir(&output_dir)?;
        }
        Ok(Exporter { output_dir, current_frame: 0, thread_pool: ThreadPool::new(num_export_threads), maximum_queue_size })
    }

    pub fn export_frame(&mut self) {
        let screen_image = get_screen_data();

        let mut output_path = self.output_dir.clone();
        output_path.push(format!("frame{:06}", self.current_frame));
        output_path.set_extension("png");

        self.thread_pool.execute({
            let current_frame = self.current_frame;
            move || {
                screen_image.export_png(output_path.to_str().unwrap());
                println!("frame {} exported", current_frame);
            }
        });
        self.current_frame += 1;

        if self.thread_pool.queued_count() > self.maximum_queue_size {
            println!(
                "maximum queue size reached; waiting for {} threads to finish until frame {}",
                self.thread_pool.queued_count(),
                self.current_frame
            );
            self.thread_pool.join();
        }
    }

    pub fn finish(&self) {
        println!("waiting for {} frames to finish exporting; total frame count {}", self.thread_pool.queued_count(), self.current_frame);
        self.thread_pool.join();
    }

    pub fn export_midi(&self, music: &PianoPhase, output_path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        use midly::{
            num::{u15, u24, u28, u4, u7},
            write_std, Format, Header, MetaMessage, MidiMessage, Timing, Track, TrackEvent, TrackEventKind,
        };

        // even though the music is written so that each note is a 16th note, we pretend that all of the notes are quarter notes for ease of exporting
        let ticks_per_quarter_note = music
            .part1
            .flattened
            .iter()
            .chain(music.part2.flattened.iter())
            .flat_map(|note| [*note.time.denom(), *(note.time + note.length).denom()])
            .reduce(lcm)
            .unwrap();
        let convert_time_to_ticks = |time: Rational32| {
            let time_multiplied = time * Ratio::from_integer(ticks_per_quarter_note);
            assert_eq!(*time_multiplied.numer(), time_multiplied.to_integer()); // this is just a sanity check because this should be mathematically true anyways
            time_multiplied.to_integer()
        };

        fn make_track(mut events: Vec<(i32, TrackEventKind)>) -> Track {
            events.sort_by_key(|(ev_time, _)| *ev_time);

            let mut new_events = Vec::new();
            let mut last_time = 0;
            for (event_time, event_kind) in events {
                new_events.push(TrackEvent { delta: u28::try_from((event_time - last_time).try_into().unwrap()).unwrap(), kind: event_kind });
                last_time = event_time;
            }

            new_events.push(TrackEvent { delta: 0.into(), kind: TrackEventKind::Meta(MetaMessage::EndOfTrack) });

            new_events
        }
        let convert_part = |part: &Part, track_number: u16, channel_number: u4| -> Track {
            let header_events = [
                (0, TrackEventKind::Meta(MetaMessage::TrackNumber(Some(track_number)))),
                (0, TrackEventKind::Meta(MetaMessage::MidiChannel(channel_number))),
                (0, TrackEventKind::Meta(MetaMessage::Tempo(u24::try_from(60_000_000u32 / music.tempo as u32).unwrap()))),
                (0, TrackEventKind::Meta(MetaMessage::TimeSignature(1, 2, 24 * 2, 8))), // metronome clicks every 2 quarter notes (every 2 notes)
            ];

            let midi_events: Vec<_> = part
                .flattened
                .iter()
                .flat_map(|flattened_note| {
                    [
                        (
                            convert_time_to_ticks(flattened_note.time),
                            TrackEventKind::Midi {
                                channel: channel_number,
                                message: MidiMessage::NoteOn {
                                    key: flattened_note.pitch.into(),
                                    vel: u7::new((flattened_note.volume * u7::max_value().as_int() as f32).floor() as u8),
                                },
                            },
                        ),
                        (
                            convert_time_to_ticks(flattened_note.time + flattened_note.length),
                            TrackEventKind::Midi {
                                channel: channel_number,
                                message: MidiMessage::NoteOff { key: flattened_note.pitch.into(), vel: 0.into() },
                            },
                        ),
                    ]
                })
                .collect();

            make_track(header_events.into_iter().chain(midi_events).collect())
        };

        let output_file = File::create(output_path)?;

        let header = Header {
            format: Format::Parallel,
            timing: Timing::Metrical(
                u15::try_from(ticks_per_quarter_note.try_into().expect("ticks per beat cannot fit into a u16"))
                    .expect("ticks per beat cannot fit into a u15"),
            ),
        };

        write_std(&header, [&convert_part(&music.part1, 0, 0.into()), &convert_part(&music.part2, 1, 1.into())], output_file)?;

        Ok(())
    }
}

fn lcm<T: Mul<T, Output = T> + Copy + Ord + Rem<T, Output = T> + num_traits::Zero + Div<Output = T>>(x: T, y: T) -> T {
    x * y / gcd(x, y)
}
fn gcd<T: Copy + Ord + Rem<T, Output = T> + num_traits::Zero>(x: T, y: T) -> T {
    let mut max = std::cmp::max(x, y);
    let mut min = std::cmp::min(x, y);

    while min > T::zero() {
        let next_max = min;
        let next_min = max % min;

        max = next_max;
        min = next_min;
    }

    max
}
