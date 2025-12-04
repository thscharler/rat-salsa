use crate::Palette;
use std::borrow::Cow;

/// Imperial palette.
///
/// Uses purple and gold for primary/secondary.
/// Other colors are bright, strong and slightly smudged.
///
pub const OXOCARBON: Palette = Palette {
    name: Cow::Borrowed("OxoCarbon"),

    text_light: Palette::color32(0xdde1e6),
    text_bright: Palette::color32(0xffffff),
    text_dark: Palette::color32(0x3c3c3c),
    text_black: Palette::color32(0x161616),

    primary: Palette::interpolate(0x993560, 0xf7569c, 63),
    secondary: Palette::interpolate(0x464646, 0x5f5f5f, 63),

    white: Palette::interpolate(0xdde1e6, 0xffffff, 63),
    black: Palette::interpolate(0x161616, 0x3c3c3c, 63),
    gray: Palette::interpolate(0x464646, 0x5f5f5f, 63),

    red: Palette::interpolate(0x993560, 0xf7569c, 63),
    orange: Palette::interpolate(0x99755c, 0xf8bd96, 63),
    yellow: Palette::interpolate(0x998b6c, 0xfae3b0, 63),
    limegreen: Palette::interpolate(0x2c8043, 0x42be665, 63),
    green: Palette::interpolate(0x05807d, 0x08bdba, 63),
    bluegreen: Palette::interpolate(0x4e7c99, 0x82cfff, 63),
    cyan: Palette::interpolate(0x24807e, 0x3ddbd9, 63),
    blue: Palette::interpolate(0x1f6a99, 0x33b1ff, 63),
    deepblue: Palette::interpolate(0x486599, 0x78a9ff, 63),
    purple: Palette::interpolate(0x725999, 0xbe95ff, 63),
    magenta: Palette::interpolate(0x725999, 0xbe95ff, 63),
    redpink: Palette::interpolate(0x994c71, 0xff7ebd, 63),
};
