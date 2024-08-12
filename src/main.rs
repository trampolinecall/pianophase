use macroquad::prelude::{next_frame, Conf};

mod music;
mod player;
mod timing;
mod util;
mod visualizer;

fn window_conf() -> Conf {
    Conf { window_title: "Piano Phase".to_string(), window_width: 1280, window_height: 720, sample_count: 4, ..Default::default() }
}
#[macroquad::main(window_conf)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let music = music::PianoPhase::new(110 * 4);
    let music = music::PianoPhase::new_shortened(110 * 16);
    // let music = music::PianoPhase::new_shortened(50);
    let mut timing = timing::Timing::new();
    let mut player = player::Player::new()?;
    let mut visualizer = visualizer::Visualizer::new().await?;

    loop {
        timing.update();
        visualizer.update(&timing, &music);
        player.update(&timing, &music);

        next_frame().await
    }
}
