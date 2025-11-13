use crate::Palette;

/// Radium
/// An adaption of nvchad's radium theme.
///
/// -- credits to original radium theme from <https://github.com/dharmx>
const DARKNESS: u8 = 63;

pub const RADIUM: Palette = Palette {
    name: "Radium",

    text_dark: Palette::color32(0x292c30),
    text_black: Palette::color32(0x101317),
    text_light: Palette::color32(0xc4c4c5),
    text_bright: Palette::color32(0xd4d4d5),

    primary: Palette::interpolate(0x1fa372, 0x37d99e, DARKNESS),
    secondary: Palette::interpolate(0x9759b5, 0xb68acb, DARKNESS),

    white: Palette::interpolate(0xc4c4c5, 0xd4d4d5, DARKNESS),
    black: Palette::interpolate(0x101317, 0x292c30, DARKNESS),
    gray: Palette::interpolate(0x3e4145, 0x525559, DARKNESS),
    red: Palette::interpolate(0xf64b4b, 0xf87070, DARKNESS),
    orange: Palette::interpolate(0xe6723d, 0xf0a988, DARKNESS),
    yellow: Palette::interpolate(0xffc424, 0xffe59e, DARKNESS),
    limegreen: Palette::interpolate(0x3cb77a, 0x92e2ba, DARKNESS),
    green: Palette::interpolate(0x3cb77a, 0x8bd6b0, DARKNESS),
    bluegreen: Palette::interpolate(0x21af7b, 0x38d89d, DARKNESS),
    cyan: Palette::interpolate(0x2b9fa5, 0x50cad2, DARKNESS),
    blue: Palette::interpolate(0x2b72b1, 0x7ab0df, DARKNESS),
    deepblue: Palette::interpolate(0x3476af, 0x719fc4, DARKNESS),
    purple: Palette::interpolate(0x9759b5, 0xb68acb, DARKNESS),
    magenta: Palette::interpolate(0xe03838, 0xf46666, DARKNESS),
    redpink: Palette::interpolate(0xf43f3f, 0xf77171, DARKNESS),
};
