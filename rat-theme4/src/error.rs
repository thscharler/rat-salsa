use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct LoadPaletteErr(pub u8);

impl Display for LoadPaletteErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "load palette failed: {}", self.0)
    }
}

impl Error for LoadPaletteErr {}
