use crate::Palette;
use ratatui::style::Color;

///
/// Almost genuine black&white color-palette.
///
/// It uses black, white and bright as the standard colors.
/// To match the Palette semantics I needed two gray-scales
/// to make it work like the other color-palettes.
///
pub const BLACKWHITE: Palette = Palette {
    name: "Black&White",

    text_light: Palette::color32(0xffffff),
    text_bright: Palette::color32(0xffffff),
    text_dark: Palette::color32(0x000000),
    text_black: Palette::color32(0x000000),

    primary: interpolate(0xffffff, 0xffffff, 0xaaaaaa, 0xaaaaaa),
    secondary: interpolate(0x000000, 0x000000, 0x000000, 0x000000),

    white: interpolate(0xffffff, 0xffffff, 0xaaaaaa, 0xaaaaaa),
    black: interpolate(0x000000, 0x000000, 0x000000, 0x000000),
    gray: interpolate(0x808080, 0x808080, 0x555555, 0x555555),

    red: interpolate(0xffffff, 0xaaaaaa, 0x000000, 0x555555),
    orange: interpolate(0xffffff, 0xaaaaaa, 0x000000, 0x555555),
    yellow: interpolate(0xffffff, 0xaaaaaa, 0x000000, 0x555555),
    limegreen: interpolate(0xffffff, 0xaaaaaa, 0x000000, 0x555555),
    green: interpolate(0xffffff, 0xaaaaaa, 0x000000, 0x555555),
    bluegreen: interpolate(0xffffff, 0xaaaaaa, 0x000000, 0x555555),
    cyan: interpolate(0xffffff, 0xaaaaaa, 0x000000, 0x555555),
    blue: interpolate(0xffffff, 0xaaaaaa, 0x000000, 0x555555),
    deepblue: interpolate(0xffffff, 0xaaaaaa, 0x000000, 0x555555),
    purple: interpolate(0xffffff, 0xaaaaaa, 0x000000, 0x555555),
    magenta: interpolate(0xffffff, 0xaaaaaa, 0x000000, 0x555555),
    redpink: interpolate(0xffffff, 0xaaaaaa, 0x000000, 0x555555),
};

/// Calculates a linear interpolation for the two colors
/// and fills the first 4 colors.
///
///
const fn interpolate(c0: u32, c1: u32, c2: u32, c3: u32) -> [Color; 8] {
    let r0 = (c0 >> 16) as u8;
    let g0 = (c0 >> 8) as u8;
    let b0 = c0 as u8;

    let r2 = (c1 >> 16) as u8;
    let g2 = (c1 >> 8) as u8;
    let b2 = c1 as u8;

    let r4 = (c2 >> 16) as u8;
    let g4 = (c2 >> 8) as u8;
    let b4 = c2 as u8;

    let r6 = (c3 >> 16) as u8;
    let g6 = (c3 >> 8) as u8;
    let b6 = c3 as u8;

    [
        Color::Rgb(r0, g0, b0),
        Color::Rgb(r0, g0, b0),
        Color::Rgb(r2, g2, b2),
        Color::Rgb(r2, g2, b2),
        Color::Rgb(r4, g4, b4),
        Color::Rgb(r4, g4, b4),
        Color::Rgb(r6, g6, b6),
        Color::Rgb(r6, g6, b6),
    ]
}
