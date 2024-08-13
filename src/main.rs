use std::time::Duration;

use macroquad::{
    input::{is_key_pressed, KeyCode},
    prelude::{next_frame, Conf},
};

mod exporter;
mod music;
mod player;
mod timing;
mod util;
mod visualizer;

const WINDOW_WIDTH: i32 = 200;
const WINDOW_HEIGHT: i32 = 200;
const EXPORT: bool = true;
const EXPORT_DIR: &str = "output/";
const EXPORT_FPS: u32 = 30;

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
    // let music = music::PianoPhase::new(120 * 4);
    let music = music::PianoPhase::new_shortened(72 * 6);
    // let music = music::PianoPhase::new_shortened(50);
    let mut timing = timing::Timing::new(if EXPORT { Some(EXPORT_FPS) } else { None });
    let mut exporter = exporter::Exporter::new(EXPORT_DIR.into())?;
    let mut player = player::Player::new()?;
    let mut visualizer = visualizer::Visualizer::new().await?;

    loop {
        if is_key_pressed(KeyCode::Right) {
            timing.seek_forward(Duration::from_secs(5));
        }
        if is_key_pressed(KeyCode::Left) {
            timing.seek_backwards(Duration::from_secs(5));
        }
        if is_key_pressed(KeyCode::Space) {
            timing.toggle_stopped();
        }

        timing.update();
        visualizer.update(&timing, &music);
        player.update(&timing, &music);

        if EXPORT {
            exporter.export_frame();
        }

        next_frame().await
    }
}
