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
        ("Imperial".to_string(), IMPERIAL),
        ("Radium".to_string(), RADIUM),
        ("Tundra".to_string(), TUNDRA),
        ("Ocean".to_string(), OCEAN),
        ("Monochrome".to_string(), MONOCHROME),
        ("Black&White".to_string(), BLACKWHITE),
        ("Base16".to_string(), BASE16),
        ("Base16Relaxed".to_string(), BASE16_RELAXED),
        ("Monekai".to_string(), MONEKAI),
        ("Solarized".to_string(), SOLARIZED),
        ("OxoCarbon".to_string(), OXOCARBON),
        ("VSCodeDark".to_string(), VSCODE_DARK),
    ]
}

/// A list of DarkTheme for all color palettes.
pub fn dark_themes() -> Vec<DarkTheme> {
    vec![
        DarkTheme::new("Imperial", IMPERIAL),
        DarkTheme::new("Radium", RADIUM),
        DarkTheme::new("Tundra", TUNDRA),
        DarkTheme::new("Ocean", OCEAN),
        DarkTheme::new("Monochrome", MONOCHROME),
        DarkTheme::new("Black&White", BLACKWHITE),
        DarkTheme::new("Base16", BASE16),
        DarkTheme::new("Base16Relaxed", BASE16_RELAXED),
        DarkTheme::new("Monekai", MONEKAI),
        DarkTheme::new("Solarized", SOLARIZED),
        DarkTheme::new("Oxocarbon", OXOCARBON),
        DarkTheme::new("VSCodeDark", VSCODE_DARK),
    ]
}
