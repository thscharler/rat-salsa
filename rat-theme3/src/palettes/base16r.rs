use crate::Palette;
use ratatui::style::Color;

/// Base 16 colors as a Palette.
///
/// A bit relaxed though, providing a gradient for each color.
///
pub const BASE16_RELAX: Palette = Palette {
    name: "Base16 Relax",

    text_light: Palette::color32(0xaaaaaa),
    text_bright: Palette::color32(0xffffff),
    text_dark: Palette::color32(0x555555),
    text_black: Palette::color32(0x000000),

    primary: Palette::interpolate(0x00aa00, 0x57ff57, 63),
    secondary: Palette::interpolate(0x00aaaa, 0x57ffff, 63),

    white: Palette::interpolate(0xaaaaaa, 0xffffff, 63),
    gray: Palette::interpolate(0x555555, 0xaaaaaa, 63),
    black: Palette::interpolate(0x000000, 0x555555, 63),

    red: Palette::interpolate(0xaa0000, 0xff5757, 63),
    orange: Palette::interpolate(0xaa5500, 0xffab57, 63),
    yellow: Palette::interpolate(0xffff55, 0xffffb3, 63),
    limegreen: Palette::interpolate(0x55ff55, 0xb3ffb3, 63),
    green: Palette::interpolate(0x00aa00, 0x57ff57, 63),
    bluegreen: Palette::interpolate(0x55ffff, 0xb3ffff, 63),
    cyan: Palette::interpolate(0x00aaaa, 0x57ffff, 63),
    blue: Palette::interpolate(0x5555ff, 0xb3b3ff, 63),
    deepblue: Palette::interpolate(0x0000af, 0x5757af, 63),
    purple: Palette::interpolate(0xaa00aa, 0xff57ff, 63),
    magenta: Palette::interpolate(0xff55ff, 0xffb3ff, 63),
    redpink: Palette::interpolate(0xff5555, 0xffb3b3, 63),
};
