use std::time::{Duration, Instant};

use num_rational::Ratio;
use num_traits::ToPrimitive;

use crate::music::PianoPhase;

pub struct Timing {
    last_time: Duration,
    time: Duration,
    last_instant: Option<Instant>,
    stopped: bool,
    constant_fps: Option<u32>,
}

impl Timing {
    pub fn new(constant_fps: Option<u32>) -> Self {
        Self { last_time: Duration::ZERO, time: Duration::ZERO, last_instant: None, stopped: true, constant_fps }
    }

    pub fn update(&mut self) {
        self.last_time = self.time;
        if let Some(constant_fps) = self.constant_fps {
            self.time += Duration::new(0, 1_000_000_000 / constant_fps);
        } else {
            match self.last_instant {
                Some(last_instant) => {
                    let now = Instant::now();

                    if !self.stopped {
                        self.time += now.duration_since(last_instant);
                    }

                    self.last_instant = Some(now);
                }
                None => {
                    self.last_instant = Some(Instant::now());
                }
            }
        }
    }

    pub fn last_time(&self) -> Duration {
        self.last_time
    }
    pub fn last_musical_time(&self, music: &PianoPhase) -> f32 {
        music.tempo as f32 * self.last_time.as_secs_f32() / 60.0
    }

    pub fn current_time(&self) -> Duration {
        self.time
    }
    pub fn current_musical_time(&self, music: &PianoPhase) -> f32 {
        music.tempo as f32 * self.time.as_secs_f32() / 60.0
    }

    pub fn toggle_stopped(&mut self) {
        self.stopped = !self.stopped;
    }
    pub fn seek_forward(&mut self, amount: Duration) {
        self.time = self.time.saturating_add(amount);
        self.last_time = self.time;
    }
    pub fn seek_backwards(&mut self, amount: Duration) {
        self.time = self.time.saturating_sub(amount);
        self.last_time = self.time;
    }

    pub fn should_end(&self, music: &PianoPhase) -> bool {
        let last_note_end = music.part1.flattened.iter().chain(&music.part2.flattened).map(|n| n.time + n.length).max().unwrap();
        // stop one note after everything is over
        self.current_musical_time(music) > (last_note_end + Ratio::ONE).to_f32().unwrap()
    }
}
