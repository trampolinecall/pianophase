use std::path::PathBuf;

use macroquad::texture::get_screen_data;

pub struct Exporter {
    output_dir: PathBuf,
    current_frame: u32,
}

impl Exporter {
    pub fn new(output_dir: PathBuf) -> Result<Exporter, Box<dyn std::error::Error>> {
        std::fs::create_dir(&output_dir)?;
        Ok(Exporter { output_dir, current_frame: 0 })
    }

    pub fn export_frame(&mut self) {
        let screen_image = get_screen_data();
        let mut output_path = self.output_dir.clone();
        output_path.push(format!("frame{:05}", self.current_frame));
        output_path.set_extension("png");
        screen_image.export_png(output_path.to_str().unwrap());
        self.current_frame += 1;
    }
}
