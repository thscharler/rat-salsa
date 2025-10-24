use crate::Palette;

/// Some rust colored palette.
pub const RUST: Palette = Palette {
    primary: Palette::interpolate(0x75311a, 0xd25a32, 63),
    secondary: Palette::interpolate(0x1a7574, 0x32d2d1, 63),

    white: Palette::interpolate(0xc4bfbb, 0xede3e3, 63),
    gray: Palette::interpolate(0x4f4845, 0xbab3b0, 63),
    black: Palette::interpolate(0x131111, 0x4f4845, 63),

    red: Palette::interpolate(0x75311a, 0xd25a32, 63),
    orange: Palette::interpolate(0x75431a, 0xd27a32, 63),
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
