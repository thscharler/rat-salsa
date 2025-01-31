use crate::schemes::{
    BASE16, BASE16_RELAXED, IMPERIAL, MONEKAI, MONOCHROME, OCEAN, OXOCARBON, RADIUM, TUNDRA,
    VSCODE_DARK,
};

mod base16;
mod base16r;
mod dark_theme;
mod imperial;
mod monekai;
mod monochrome;
mod ocean;
mod oxocarbon;
mod radium;
mod scheme;
mod tundra;
mod vscode_dark;

pub use dark_theme::*;
pub use scheme::*;

/// Color schemes
pub mod schemes {
    pub use crate::base16::BASE16;
    pub use crate::base16r::BASE16_RELAXED;
    pub use crate::imperial::IMPERIAL;
    pub use crate::monekai::MONEKAI;
    pub use crate::monochrome::MONOCHROME;
    pub use crate::ocean::OCEAN;
    pub use crate::oxocarbon::OXOCARBON;
    pub use crate::radium::RADIUM;
    pub use crate::tundra::TUNDRA;
    pub use crate::vscode_dark::VSCODE_DARK;
}

/// All currently existing color schemes.
pub fn color_schemes() -> Vec<(String, Scheme)> {
    vec![
        ("Imperial".to_string(), IMPERIAL),
        ("Radium".to_string(), RADIUM),
        ("Tundra".to_string(), TUNDRA),
        ("Monochrome".to_string(), MONOCHROME),
        ("Monekai".to_string(), MONEKAI),
        ("OxoCarbon".to_string(), OXOCARBON),
        ("VSCodeDark".to_string(), VSCODE_DARK),
        ("Ocean".to_string(), OCEAN),
        ("Base16".to_string(), BASE16),
        ("Base16Relaxed".to_string(), BASE16_RELAXED),
    ]
}

/// A list of DarkTheme for all color schemes.
pub fn dark_themes() -> Vec<DarkTheme> {
    vec![
        DarkTheme::new("Imperial".to_string(), IMPERIAL),
        DarkTheme::new("Radium".to_string(), RADIUM),
        DarkTheme::new("Tundra".to_string(), TUNDRA),
        DarkTheme::new("Monochrome".to_string(), MONOCHROME),
        DarkTheme::new("Monekai".to_string(), MONEKAI),
        DarkTheme::new("Oxocarbon".to_string(), OXOCARBON),
        DarkTheme::new("VSCodeDark".to_string(), VSCODE_DARK),
        DarkTheme::new("Ocean".to_string(), OCEAN),
        DarkTheme::new("Base16".to_string(), BASE16),
        DarkTheme::new("Base16Relaxed".to_string(), BASE16_RELAXED),
    ]
}
