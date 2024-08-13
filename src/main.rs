use std::time::Duration;

use macroquad::{
    input::{is_key_pressed, is_quit_requested, prevent_quit, KeyCode},
    prelude::{next_frame, Conf},
};

mod exporter;
mod music;
mod player;
mod timing;
mod util;
mod visualizer;

const BPM_FOR_EIGTH_NOTE: u16 = 72 * 3;
const SHORTEN: bool = true;

const WINDOW_WIDTH: i32 = 1000;
const WINDOW_HEIGHT: i32 = 1000;

const EXPORT: bool = true;
const EXPORT_DIR: &str = "output/";
const EXPORT_FPS: u32 = 30;
const NUM_EXPORT_THREADS: usize = 10;
const MAX_EXPORT_QUEUE_SIZE: usize = 40;

const PLAY_ON_EXPORT: bool = false;

fn window_conf() -> Conf {
    Conf {
        window_title: "Piano Phase".to_string(),
        window_width: WINDOW_WIDTH,
        window_height: WINDOW_HEIGHT,
        sample_count: 4,
        window_resizable: false,
        ..Default::default()
    }
}
#[macroquad::main(window_conf)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    prevent_quit();

    let music = if SHORTEN { music::PianoPhase::new_shortened(BPM_FOR_EIGTH_NOTE * 2) } else { music::PianoPhase::new(BPM_FOR_EIGTH_NOTE * 2) };

    let mut timing = timing::Timing::new(if EXPORT { Some(EXPORT_FPS) } else { None });
    let mut exporter = exporter::Exporter::new(EXPORT_DIR.into(), NUM_EXPORT_THREADS, MAX_EXPORT_QUEUE_SIZE)?;
    let mut player = player::Player::new()?;
    let mut visualizer = visualizer::Visualizer::new().await?;

    let should_play = if EXPORT { PLAY_ON_EXPORT } else { true };

    loop {
        if !EXPORT {
            if is_key_pressed(KeyCode::Right) {
                timing.seek_forward(Duration::from_secs(5));
            }
            if is_key_pressed(KeyCode::Left) {
                timing.seek_backwards(Duration::from_secs(5));
            }
            if is_key_pressed(KeyCode::Space) {
                timing.toggle_stopped();
            }
        }

        if is_quit_requested() {
            break;
        }

        timing.update();
        visualizer.update(&timing, &music);
        if should_play {
            player.update(&timing, &music);
        }

        if EXPORT {
            exporter.export_frame();
        }

        if !EXPORT {
            next_frame().await
        }
    }

    exporter.finish();

    Ok(())
}
