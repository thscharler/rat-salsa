use crate::Palette;
use ratatui::style::Color;

/// Imperial palette.
///
/// Uses purple and gold for primary/secondary.
/// Other colors are bright, strong and slightly smudged.
///
pub const IMPERIAL: Palette = Palette {
    name: "Imperial",

    text_light: Palette::color32(0xdedfe3),
    text_bright: Palette::color32(0xf6f6f3),
    text_dark: Palette::color32(0x2a2b37),
    text_black: Palette::color32(0x0f1014),

    primary: Palette::interpolate(0x300057, 0x8c00fd, 63),
    secondary: Palette::interpolate(0x574b00, 0xffde00, 63),

    white: Palette::interpolate(0xdedfe3, 0xf6f6f3, 63),
    black: Palette::interpolate(0x0f1014, 0x2a2b37, 63),
    gray: Palette::interpolate(0x3b3d4e, 0x6e7291, 63),

    red: Palette::interpolate(0x480f0f, 0xd22d2d, 63),
    orange: Palette::interpolate(0x482c0f, 0xd4812b, 63),
    yellow: Palette::interpolate(0x756600, 0xffde00, 63),
    limegreen: Palette::interpolate(0x2c4611, 0x80ce31, 63),
    green: Palette::interpolate(0x186218, 0x32cd32, 63),
    bluegreen: Palette::interpolate(0x206a52, 0x3bc49a, 63),
    cyan: Palette::interpolate(0x0f2c48, 0x2bd4d4, 63),
    blue: Palette::interpolate(0x162b41, 0x2b81d4, 63),
    deepblue: Palette::interpolate(0x202083, 0x3232cd, 63),
    purple: Palette::interpolate(0x4d008b, 0x8c00fd, 63),
    magenta: Palette::interpolate(0x401640, 0xbd42bd, 63),
    redpink: Palette::interpolate(0x47101d, 0xc33c5b, 63),
};
