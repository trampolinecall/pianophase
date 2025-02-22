use std::{
    fs::File,
    sync::{Arc, Mutex},
};

use itertools::Itertools;
use num_traits::ToPrimitive;
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};
use tinyaudio::{run_output_device, BaseAudioOutputDevice, OutputDeviceParameters};

use crate::{
    music::{Part, PianoPhase},
    timing::Timing,
    util::remap,
};

pub struct Player {
    synthesizer: Arc<Mutex<Synthesizer>>,
    _device: Box<dyn BaseAudioOutputDevice>,
}

const SAMPLE_RATE: i32 = 44100;

impl Player {
    pub fn new() -> Result<Player, Box<dyn std::error::Error>> {
        let mut sf2 = File::open("data/UprightPianoKW-small-SF2-20190703/UprightPianoKW-small-20190703.sf2")?;
        let sound_font = Arc::new(SoundFont::new(&mut sf2)?);

        let settings = SynthesizerSettings::new(SAMPLE_RATE);
        let synthesizer = Arc::new(Mutex::new(Synthesizer::new(&sound_font, &settings)?));

        let params =
            OutputDeviceParameters { channels_count: 2, sample_rate: SAMPLE_RATE as usize, channel_sample_count: SAMPLE_RATE as usize / 100 }; // dividing by 100 makes a maximum latency of 10ms
        let _device = run_output_device(params, {
            let mut left: Vec<f32> = vec![0_f32; params.channel_sample_count];
            let mut right: Vec<f32> = vec![0_f32; params.channel_sample_count];
            let synthesizer = Arc::clone(&synthesizer);
            move |data| {
                synthesizer.lock().unwrap().render(&mut left[..], &mut right[..]);
                for (i, value) in left.iter().interleave(right.iter()).enumerate() {
                    data[i] = *value;
                }
            }
        })
        .unwrap();

        Ok(Self { synthesizer, _device })
    }

    pub fn update(&mut self, timing: &Timing, music: &PianoPhase) {
        let last_time = timing.last_musical_time(music);
        let this_time = timing.current_musical_time(music);

        let update_part = |part: &Part, channel: i32| {
            let notes_released =
                part.find_note_range(|n| (n.time + n.length).to_f32().unwrap() < last_time, |n| (n.time + n.length).to_f32().unwrap() < this_time);
            let notes_pressed = part.find_note_range(|n| n.time.to_f32().unwrap() < last_time, |n| n.time.to_f32().unwrap() < this_time);

            {
                let mut synth = self.synthesizer.lock().unwrap();
                for note in notes_released {
                    synth.note_off(channel, note.pitch as i32);
                }
                for note in notes_pressed {
                    synth.note_on(channel, note.pitch as i32, remap(note.volume, 0.0, 1.0, 0.0, 127.0) as i32);
                }
            }
        };
        update_part(&music.part1, 0);
        update_part(&music.part2, 1);
    }
}
