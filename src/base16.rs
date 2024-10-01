use crate::Scheme;

/// Base 16
///
///
///
pub const BASE16: Scheme = Scheme {
    primary: Scheme::linear4(0x00aa00, 0x00aa00),
    secondary: Scheme::linear4(0x00aaaa, 0x00aaaa),

    white: Scheme::linear4(0xaaaaaa, 0xffffff),
    gray: Scheme::linear4(0x555555, 0xaaaaaa),
    black: Scheme::linear4(0x000000, 0x555555),

    red: Scheme::linear4(0xaa0000, 0xaa0000),
    orange: Scheme::linear4(0xaa5500, 0xaa5500),
    yellow: Scheme::linear4(0xffff55, 0xffff55),
    limegreen: Scheme::linear4(0x55ff55, 0x55ff55),
    green: Scheme::linear4(0x00aa00, 0x00aa00),
    bluegreen: Scheme::linear4(0x55ffff, 0x55ffff),
    cyan: Scheme::linear4(0x00aaaa, 0x00aaaa),
    blue: Scheme::linear4(0x5555ff, 0x5555ff),
    deepblue: Scheme::linear4(0x0000af, 0x0000af),
    purple: Scheme::linear4(0xaa00aa, 0xaa00aa),
    magenta: Scheme::linear4(0xff55ff, 0xff55ff),
    redpink: Scheme::linear4(0xff5555, 0xff5555),
};
