use crate::Palette;

/// Some rust colored palette.
pub const RED: Palette = Palette {
    name: "Reds",

    text_light: Palette::color32(0xcfafaf),
    text_bright: Palette::color32(0xe2cfcf),
    text_dark: Palette::color32(0x704343),
    text_black: Palette::color32(0x301d1d),

    primary: Palette::interpolate(0xff0000, 0xff8080, 63),
    secondary: Palette::interpolate(0xff4000, 0xff9f80, 63),

    white: Palette::interpolate(0xcfafaf, 0xe2cfcf, 63),
    gray: Palette::interpolate(0x6f4343, 0xbd8f8f, 63),
    black: Palette::interpolate(0x301d1d, 0x704343, 63),

    red: Palette::interpolate(0xff0000, 0xff8080, 63),
    orange: Palette::interpolate(0xff0000, 0xff8080, 63),
    yellow: Palette::interpolate(0xff0000, 0xff8080, 63),
    limegreen: Palette::interpolate(0xff0000, 0xff8080, 63),
    green: Palette::interpolate(0xff0000, 0xff8080, 63),
    bluegreen: Palette::interpolate(0xff0000, 0xff8080, 63),
    cyan: Palette::interpolate(0xff0000, 0xff8080, 63),
    blue: Palette::interpolate(0xff0000, 0xff8080, 63),
    deepblue: Palette::interpolate(0xff0000, 0xff8080, 63),
    purple: Palette::interpolate(0xff0000, 0xff8080, 63),
    magenta: Palette::interpolate(0xff0000, 0xff8080, 63),
    redpink: Palette::interpolate(0xff0000, 0xff8080, 63),
};
