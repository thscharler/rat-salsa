use crate::Palette;

/// My take on an ocean theme.
///
/// Primary is the color of a buoy marking off a spot.
/// Secondary is a vague blue.
///
/// Colors are either bleached or shifted towards blueish.
/// Overall it's not very dark, more of a sunny day than
/// a foggy, rainy night.
pub const OCEAN: Palette = Palette {
    name: "Ocean",

    text_light: Palette::color32(0xe5e5dd),
    text_bright: Palette::color32(0xf2f2ee),
    text_dark: Palette::color32(0x0c092c),
    text_black: Palette::color32(0x030305),

    primary: Palette::interpolate(0xff8d3c, 0xffbf3c, 63),
    secondary: Palette::interpolate(0x2b4779, 0x6688cc, 63),

    white: Palette::interpolate(0xe5e5dd, 0xf2f2ee, 63),
    black: Palette::interpolate(0x030305, 0x0c092c, 63),
    gray: Palette::interpolate(0x4f6167, 0xbcc7cc, 63),

    red: Palette::interpolate(0xff5e7f, 0xff9276, 63),
    orange: Palette::interpolate(0xff9f5b, 0xffdc94, 63),
    yellow: Palette::interpolate(0xffda5d, 0xfff675, 63),
    limegreen: Palette::interpolate(0x7d8447, 0xe1e5b9, 63),
    green: Palette::interpolate(0x658362, 0x99c794, 63),
    bluegreen: Palette::interpolate(0x3a615c, 0x5b9c90, 63),
    cyan: Palette::interpolate(0x24adbc, 0xb8dade, 63),
    blue: Palette::interpolate(0x4f86ca, 0xbfdcff, 63),
    deepblue: Palette::interpolate(0x2b4779, 0x6688cc, 63),
    purple: Palette::interpolate(0x5068d7, 0xc7c4ff, 63),
    magenta: Palette::interpolate(0x7952d6, 0xc9bde4, 63),
    redpink: Palette::interpolate(0x9752d6, 0xcebde4, 63),
};
