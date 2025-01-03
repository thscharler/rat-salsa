use crate::Scheme;

/// Base 16 colors as a Scheme.
///
/// A bit relaxed though, providing a gradient for each color.
///
pub const BASE16_RELAXED: Scheme = Scheme {
    primary: Scheme::linear4(0x00aa00, 0x57ff57),
    secondary: Scheme::linear4(0x00aaaa, 0x57ffff),

    white: Scheme::linear4(0xaaaaaa, 0xffffff),
    gray: Scheme::linear4(0x555555, 0xaaaaaa),
    black: Scheme::linear4(0x000000, 0x555555),

    red: Scheme::linear4(0xaa0000, 0xff5757),
    orange: Scheme::linear4(0xaa5500, 0xffab57),
    yellow: Scheme::linear4(0xffff55, 0xffffb3),
    limegreen: Scheme::linear4(0x55ff55, 0xb3ffb3),
    green: Scheme::linear4(0x00aa00, 0x57ff57),
    bluegreen: Scheme::linear4(0x55ffff, 0xb3ffff),
    cyan: Scheme::linear4(0x00aaaa, 0x57ffff),
    blue: Scheme::linear4(0x5555ff, 0xb3b3ff),
    deepblue: Scheme::linear4(0x0000af, 0x5757af),
    purple: Scheme::linear4(0xaa00aa, 0xff57ff),
    magenta: Scheme::linear4(0xff55ff, 0xffb3ff),
    redpink: Scheme::linear4(0xff5555, 0xffb3b3),
};
