use crate::Palette;
use std::borrow::Cow;

/// An adaption of nvchad's tundra theme.
///
/// -- Thanks to original theme for existing <https://github.com/sam4llis/nvim-tundra>
/// -- this is a modified version of it
pub const TUNDRA: Palette = Palette {
    name: Cow::Borrowed("Tundra"),

    text_light: Palette::color32(0xe6eaf2),
    text_bright: Palette::color32(0xffffff),
    text_dark: Palette::color32(0x1a2130),
    text_black: Palette::color32(0x0b1221),

    primary: Palette::interpolate(0xe6eaf2, 0xffffff, 63),
    secondary: Palette::interpolate(0xa8bbd4, 0x719bd3, 63),

    white: Palette::interpolate(0xe6eaf2, 0xffffff, 63),
    black: Palette::interpolate(0x0b1221, 0x1a2130, 63),
    gray: Palette::interpolate(0x3e4554, 0x5f6675, 63),

    red: Palette::interpolate(0xfccaca, 0xfca5a5, 63),
    orange: Palette::interpolate(0xfad9c5, 0xfbc19d, 63),
    yellow: Palette::interpolate(0xe8d7b7, 0xe8d4b0, 63),
    limegreen: Palette::interpolate(0xbce8b7, 0xb5e8b0, 63),
    green: Palette::interpolate(0xbce8b7, 0xb5e8b0, 63),
    bluegreen: Palette::interpolate(0xa8bbd4, 0x719bd3, 63),
    cyan: Palette::interpolate(0xc8eafc, 0xbae6fd, 63),
    blue: Palette::interpolate(0xc7d0fc, 0xa5b4fc, 63),
    deepblue: Palette::interpolate(0xbfcaf2, 0x9baaf2, 63),
    purple: Palette::interpolate(0xb7abd9, 0xb3a6da, 63),
    magenta: Palette::interpolate(0xffc9c9, 0xff8e8e, 63),
    redpink: Palette::interpolate(0xfffcad, 0xfecdd3, 63),
};
