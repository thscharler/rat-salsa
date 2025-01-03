use crate::Scheme;

/// Imperial scheme.
///
/// Uses purple and gold for primary/secondary.
/// Other colors are bright, strong and slightly smudged.
///
pub const OXOCARBON: Scheme = Scheme {
    primary: Scheme::linear4(0x993560, 0xf7569c),
    secondary: Scheme::linear4(0x464646, 0x5f5f5f),

    white: Scheme::linear4(0xdde1e6, 0xffffff),
    black: Scheme::linear4(0x161616, 0x3c3c3c),
    gray: Scheme::linear4(0x464646, 0x5f5f5f),

    red: Scheme::linear4(0x993560, 0xf7569c),
    orange: Scheme::linear4(0x99755c, 0xf8bd96),
    yellow: Scheme::linear4(0x998b6c, 0xfae3b0),
    limegreen: Scheme::linear4(0x2c8043, 0x42be665),
    green: Scheme::linear4(0x05807d, 0x08bdba),
    bluegreen: Scheme::linear4(0x4e7c99, 0x82cfff),
    cyan: Scheme::linear4(0x24807e, 0x3ddbd9),
    blue: Scheme::linear4(0x1f6a99, 0x33b1ff),
    deepblue: Scheme::linear4(0x486599, 0x78a9ff),
    purple: Scheme::linear4(0x725999, 0xbe95ff),
    magenta: Scheme::linear4(0x725999, 0xbe95ff),
    redpink: Scheme::linear4(0x994c71, 0xff7ebd),
};
