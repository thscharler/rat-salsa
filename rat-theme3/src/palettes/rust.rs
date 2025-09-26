use crate::Palette;

/// Some rust colored palette.
pub const RUST: Palette = Palette {
    name: "Rust",

    text_light: Palette::color32(0x928781),
    text_bright: Palette::color32(0xede3e3),
    text_dark: Palette::color32(0x4f4845),
    text_black: Palette::color32(0x131111),

    primary: Palette::interpolate(0x75381a, 0xd26932, 63),
    secondary: Palette::interpolate(0x1a7574, 0x32d2d1, 63),

    white: Palette::interpolate(0x928781, 0xede3e3, 63),
    black: Palette::interpolate(0x131111, 0x4f4845, 63),
    gray: Palette::interpolate(0x4f4845, 0x928781, 63),

    red: Palette::interpolate(0x75381a, 0xd26932, 63),
    orange: Palette::interpolate(0x75381a, 0xd26932, 63),
    yellow: Palette::interpolate(0x77551d, 0xcd9537, 63),
    limegreen: Palette::interpolate(0x4e7a1a, 0x89d232, 63),
    green: Palette::interpolate(0x4e7a1a, 0x89d232, 63),
    bluegreen: Palette::interpolate(0x1a7574, 0x32d2d1, 63),
    cyan: Palette::interpolate(0x1a7574, 0x32d2d1, 63),
    blue: Palette::interpolate(0x1b427e, 0x3271d2, 63),
    deepblue: Palette::interpolate(0x1b427e, 0x3271d2, 63),
    purple: Palette::interpolate(0x5f1a7a, 0xa731d3, 63),
    magenta: Palette::interpolate(0x7b1964, 0xd332ad, 63),
    redpink: Palette::interpolate(0x7b1964, 0xd332ad, 63),
};
