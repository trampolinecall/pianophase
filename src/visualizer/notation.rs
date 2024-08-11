use std::{fs::File, io::BufReader};

use smufl::Metadata;

#[allow(clippy::manual_non_exhaustive)]
pub struct Font {
    pub font: macroquad::text::Font,
    pub metadata: smufl::Metadata,
    _dont_construct: (),
}

impl Font {
    pub async fn load_bravura() -> Result<Font, Box<dyn std::error::Error>> {
        let file = File::open("data/bravura/redist/bravura_metadata.json")?;
        let reader = BufReader::new(file);
        let metadata = Metadata::from_reader(reader)?;
        let font = macroquad::text::load_ttf_font("data/bravura/redist/otf/Bravura.otf").await?;
        Ok(Font { font, metadata, _dont_construct: () })
    }
}
