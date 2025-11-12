use crate::Palette;

/// Everforest
/// -- Credits to original https://github.com/sainnhe/everforest
/// -- This is modified version of it
const DARKNESS: u8 = 63;

pub const EVERFOREST: Palette = Palette {
    name: "EverForest",

    text_dark: Palette::color32(0x2c2d2a),
    text_black: Palette::color32(0x090a09),
    text_light: Palette::color32(0xd8d4cb),
    text_bright: Palette::color32(0xf4f0e6),

    primary: Palette::interpolate(0xa7c080, 0xb8ce94, DARKNESS),
    secondary: Palette::interpolate(0x78b4ac, 0x8dc4bd, DARKNESS),

    white: Palette::interpolate(0xd3c6aa, 0xc4ac7b, DARKNESS),
    black: Palette::interpolate(0x272f35, 0x30383d, DARKNESS),
    gray: Palette::interpolate(0x4e565c, 0x656d73, DARKNESS),
    red: Palette::interpolate(0xe67e80, 0xfc8f93, DARKNESS),
    orange: Palette::interpolate(0xe69875, 0xf4ab8b, DARKNESS),
    yellow: Palette::interpolate(0xdbbc7f, 0xddc187, DARKNESS),
    limegreen: Palette::interpolate(0xa7c080, 0xb8ce94, DARKNESS),
    green: Palette::interpolate(0x83c092, 0x83c092, DARKNESS),
    bluegreen: Palette::interpolate(0x69a59d, 0x77ada6, DARKNESS),
    cyan: Palette::interpolate(0x95d1c9, 0xafdbd5, DARKNESS),
    blue: Palette::interpolate(0x7393b3, 0x8fa7bf, DARKNESS),
    deepblue: Palette::interpolate(0x78b4ac, 0x8dc4bd, DARKNESS),
    purple: Palette::interpolate(0xd699b6, 0xe5b7cd, DARKNESS),
    magenta: Palette::interpolate(0xff75a0, 0xf2608e, DARKNESS),
    redpink: Palette::interpolate(0xce8196, 0xbf6b82, DARKNESS),
};
