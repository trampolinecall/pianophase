use macroquad::prelude::next_frame;

mod music;
mod player;
mod timing;
mod visualizer;

#[macroquad::main("Piano Phase")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let music = music::PianoPhase::new(110 * 4);
    let music = music::PianoPhase::new_shortened(110 * 4);
    // let music = music::PianoPhase::new_shortened(50);
    let mut timing = timing::Timing::new();
    let mut player = player::Player::new()?;
    let mut visualizer = visualizer::Visualizer::new()?;

    loop {
        timing.update();
        let r#continue = visualizer.update(&timing, &music);
        player.update(&timing, &music);

        if !r#continue {
            break;
        }

        next_frame().await
    }

    Ok(())
}
