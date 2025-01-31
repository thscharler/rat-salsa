use crate::Scheme;

/// Base 16 colors as a Scheme.
///
/// A bit relaxed though, providing a gradient for each color.
///
pub const BASE16_RELAXED: Scheme = Scheme {
    primary: Scheme::interpolate(0x00aa00, 0x57ff57, 63),
    secondary: Scheme::interpolate(0x00aaaa, 0x57ffff, 63),

    white: Scheme::interpolate(0xaaaaaa, 0xffffff, 63),
    gray: Scheme::interpolate(0x555555, 0xaaaaaa, 63),
    black: Scheme::interpolate(0x000000, 0x555555, 63),

    red: Scheme::interpolate(0xaa0000, 0xff5757, 63),
    orange: Scheme::interpolate(0xaa5500, 0xffab57, 63),
    yellow: Scheme::interpolate(0xffff55, 0xffffb3, 63),
    limegreen: Scheme::interpolate(0x55ff55, 0xb3ffb3, 63),
    green: Scheme::interpolate(0x00aa00, 0x57ff57, 63),
    bluegreen: Scheme::interpolate(0x55ffff, 0xb3ffff, 63),
    cyan: Scheme::interpolate(0x00aaaa, 0x57ffff, 63),
    blue: Scheme::interpolate(0x5555ff, 0xb3b3ff, 63),
    deepblue: Scheme::interpolate(0x0000af, 0x5757af, 63),
    purple: Scheme::interpolate(0xaa00aa, 0xff57ff, 63),
    magenta: Scheme::interpolate(0xff55ff, 0xffb3ff, 63),
    redpink: Scheme::interpolate(0xff5555, 0xffb3b3, 63),
};
