use crate::palettes::{
    BASE16, BASE16_RELAXED, BLACKWHITE, IMPERIAL, MONEKAI, MONOCHROME, OCEAN, OXOCARBON, RADIUM,
    SOLARIZED, TUNDRA, VSCODE_DARK,
};

mod dark_theme;
mod palette;
pub mod palettes;

pub use dark_theme::*;
pub use palette::*;

/// All currently existing color palettes.
pub fn color_palettes() -> Vec<(String, Palette)> {
    vec![
        ("Black&White".to_string(), BLACKWHITE),
        ("Imperial".to_string(), IMPERIAL),
        ("Radium".to_string(), RADIUM),
        ("Tundra".to_string(), SOLARIZED),
        ("Monochrome".to_string(), MONOCHROME),
        ("Monekai".to_string(), MONEKAI),
        ("Solarized".to_string(), SOLARIZED),
        ("OxoCarbon".to_string(), OXOCARBON),
        ("VSCodeDark".to_string(), VSCODE_DARK),
        ("Ocean".to_string(), OCEAN),
        ("Base16".to_string(), BASE16),
        ("Base16Relaxed".to_string(), BASE16_RELAXED),
    ]
}

/// A list of DarkTheme for all color palettes.
pub fn dark_themes() -> Vec<DarkTheme> {
    vec![
        DarkTheme::new("Black&White".to_string(), BLACKWHITE),
        DarkTheme::new("Imperial".to_string(), IMPERIAL),
        DarkTheme::new("Radium".to_string(), RADIUM),
        DarkTheme::new("Tundra".to_string(), TUNDRA),
        DarkTheme::new("Monochrome".to_string(), MONOCHROME),
        DarkTheme::new("Monekai".to_string(), MONEKAI),
        DarkTheme::new("Solarized".to_string(), SOLARIZED),
        DarkTheme::new("Oxocarbon".to_string(), OXOCARBON),
        DarkTheme::new("VSCodeDark".to_string(), VSCODE_DARK),
        DarkTheme::new("Ocean".to_string(), OCEAN),
        DarkTheme::new("Base16".to_string(), BASE16),
        DarkTheme::new("Base16Relaxed".to_string(), BASE16_RELAXED),
    ]
}
