use crate::Scheme;

/// Imperial scheme.
///
/// Uses purple and gold for primary/secondary.
/// Other colors are bright, strong and slightly smudged.
///
pub const OXOCARBON: Scheme = Scheme {
    primary: Scheme::interpolate(0x993560, 0xf7569c, 63),
    secondary: Scheme::interpolate(0x464646, 0x5f5f5f, 63),

    white: Scheme::interpolate(0xdde1e6, 0xffffff, 63),
    black: Scheme::interpolate(0x161616, 0x3c3c3c, 63),
    gray: Scheme::interpolate(0x464646, 0x5f5f5f, 63),

    red: Scheme::interpolate(0x993560, 0xf7569c, 63),
    orange: Scheme::interpolate(0x99755c, 0xf8bd96, 63),
    yellow: Scheme::interpolate(0x998b6c, 0xfae3b0, 63),
    limegreen: Scheme::interpolate(0x2c8043, 0x42be665, 63),
    green: Scheme::interpolate(0x05807d, 0x08bdba, 63),
    bluegreen: Scheme::interpolate(0x4e7c99, 0x82cfff, 63),
    cyan: Scheme::interpolate(0x24807e, 0x3ddbd9, 63),
    blue: Scheme::interpolate(0x1f6a99, 0x33b1ff, 63),
    deepblue: Scheme::interpolate(0x486599, 0x78a9ff, 63),
    purple: Scheme::interpolate(0x725999, 0xbe95ff, 63),
    magenta: Scheme::interpolate(0x725999, 0xbe95ff, 63),
    redpink: Scheme::interpolate(0x994c71, 0xff7ebd, 63),
};
