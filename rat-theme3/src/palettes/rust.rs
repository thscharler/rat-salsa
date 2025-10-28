use crate::Palette;

/// Some rust colored palette.
pub const RUST: Palette = Palette {
    name: "Rust",

    text_light: Palette::color32(0xc4bfbb),
    text_bright: Palette::color32(0xede3e3),
    text_dark: Palette::color32(0x312d2b),
    text_black: Palette::color32(0x131111),

    primary: Palette::interpolate(0x75311a, 0xd25a32, 63),
    secondary: Palette::interpolate(0x1a7574, 0x32d2d1, 63),

    white: Palette::interpolate(0xc4bfbb, 0xede3e3, 63),
    gray: Palette::interpolate(0x4f4845, 0xbab3b0, 63),
    black: Palette::interpolate(0x131111, 0x4f4845, 63),

    red: Palette::interpolate(0x75311a, 0xd25a32, 63),
    orange: Palette::interpolate(0x75431a, 0xd27a32, 63),
    yellow: Palette::interpolate(0x77551d, 0xcd9537, 63),
    limegreen: Palette::interpolate(0x44664d, 0x699b76, 63),
    green: Palette::interpolate(0x44664d, 0x699b76, 63),
    bluegreen: Palette::interpolate(0x1a7574, 0x32d2d1, 63),
    cyan: Palette::interpolate(0x1a7574, 0x32d2d1, 63),
    blue: Palette::interpolate(0x005d94, 0x38b6ff, 63),
    deepblue: Palette::interpolate(0x005d94, 0x38b6ff, 63),
    purple: Palette::interpolate(0x722234, 0xc63f5d, 63),
    magenta: Palette::interpolate(0x7b1964, 0xd332ad, 63),
    redpink: Palette::interpolate(0x7b1964, 0xd332ad, 63),
};
