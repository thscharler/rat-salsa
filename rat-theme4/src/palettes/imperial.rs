use crate::Palette;

/// Imperial
///
/// Uses purple and gold for primary/secondary.
/// Other colors are bright, strong and slightly smudged.
const DARKNESS: u8 = 63;

pub const IMPERIAL: Palette = Palette {
    name: "Imperial",

    text_dark: Palette::color32(0x2a2b37),
    text_black: Palette::color32(0x0f1014),
    text_light: Palette::color32(0xdedfe3),
    text_bright: Palette::color32(0xf6f6f3),

    primary: Palette::interpolate(0x3d0070, 0x8900f9, DARKNESS),
    secondary: Palette::interpolate(0x726100, 0xe0c200, DARKNESS),

    white: Palette::interpolate(0xdedfe3, 0xf6f6f3, DARKNESS),
    black: Palette::interpolate(0x0f1014, 0x2a2b37, DARKNESS),
    gray: Palette::interpolate(0x3b3d4e, 0x6e7291, DARKNESS),
    red: Palette::interpolate(0x601414, 0xd22d2d, DARKNESS),
    orange: Palette::interpolate(0x5e3913, 0xd3802c, DARKNESS),
    yellow: Palette::interpolate(0x756600, 0xd6b900, DARKNESS),
    limegreen: Palette::interpolate(0x3c5e17, 0x80ce31, DARKNESS),
    green: Palette::interpolate(0x186218, 0x32c932, DARKNESS),
    bluegreen: Palette::interpolate(0x1b5944, 0x3abc93, DARKNESS),
    cyan: Palette::interpolate(0x1b5184, 0x2bcece, DARKNESS),
    blue: Palette::interpolate(0x234668, 0x2b81d4, DARKNESS),
    deepblue: Palette::interpolate(0x202083, 0x3232cc, DARKNESS),
    purple: Palette::interpolate(0x4b0089, 0x8c00fd, DARKNESS),
    magenta: Palette::interpolate(0x4f1b4f, 0xbd42bd, DARKNESS),
    redpink: Palette::interpolate(0x47101d, 0xc33c5b, DARKNESS),
};
