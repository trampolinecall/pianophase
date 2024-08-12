use std::time::{Duration, Instant};

use crate::music::PianoPhase;

pub struct Timing {
    last_time: Duration,
    time: Duration,
    last_instant: Option<Instant>,
    stopped: bool,
}

impl Timing {
    pub fn new() -> Self {
        Self { last_time: Duration::ZERO, time: Duration::ZERO, last_instant: None, stopped: true }
    }

    pub fn update(&mut self) {
        self.last_time = self.time;
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
}
