use std::{fs::File, io::BufReader};

use macroquad::{color::Color, math::Vec2, text::TextParams};
use smufl::{Coord, Metadata};

#[allow(clippy::manual_non_exhaustive)]
pub struct Font {
    pub font: macroquad::text::Font,
    pub metadata: smufl::Metadata,
    _dont_construct: (),
}

#[allow(clippy::manual_non_exhaustive)]
pub struct StaffParams {
    pub staff_space: u16,
    pub staff_height: u16,
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

    pub fn make_text_params(&self, staff_params: &StaffParams, color: Color) -> TextParams {
        TextParams { font: Some(&self.font), font_size: staff_params.staff_height, font_scale: 1.0, font_scale_aspect: 1.0, rotation: 0.0, color }
    }
}

impl StaffParams {
    pub const fn new(staff_space: u16) -> Self {
        Self { staff_space, staff_height: staff_space * 4, _dont_construct: () }
    }
}

pub fn coord_to_tuple(coord: Coord, staff_space: f32) -> Vec2 {
    Vec2::new(coord.x().0 as f32 * staff_space as f32, -coord.y().0 as f32 * staff_space as f32)
}

pub fn optional_coord_to_tuple(coord: Option<Coord>, staff_space: f32) -> Vec2 {
    coord.map(|c| coord_to_tuple(c, staff_space)).unwrap_or(Vec2::new(0.0, 0.0))
}
