use std::{fs::File, io::BufReader};

use smufl::Metadata;

#[allow(clippy::manual_non_exhaustive)]
pub struct Font {
    // pub font: sfml::SfBox<sfml::graphics::Font>, TODO
    pub metadata: smufl::Metadata,
    _dont_construct: (),
}

impl Font {
    pub fn load_bravura() -> Result<Font, Box<dyn std::error::Error>> {
        let file = File::open("data/bravura/redist/bravura_metadata.json")?;
        let reader = BufReader::new(file);
        let metadata = Metadata::from_reader(reader)?;
        // TODO
        // let font = sfml::graphics::Font::from_file("data/bravura/redist/otf/Bravura.otf").ok_or("could not load font")?;
        Ok(Font { metadata, _dont_construct: () })
    }
}
