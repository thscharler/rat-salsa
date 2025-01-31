use crate::Scheme;

/// Imperial scheme.
///
/// Uses purple and gold for primary/secondary.
/// Other colors are bright, strong and slightly smudged.
///
pub const IMPERIAL: Scheme = Scheme {
    primary: Scheme::interpolate(0x300057, 0x8c00fd, 63),
    secondary: Scheme::interpolate(0x574b00, 0xffde00, 63),

    white: Scheme::interpolate(0xdedfe3, 0xf6f6f3, 63),
    black: Scheme::interpolate(0x0f1014, 0x2a2b37, 63),
    gray: Scheme::interpolate(0x3b3d4e, 0x6e7291, 63),

    red: Scheme::interpolate(0x480f0f, 0xd22d2d, 63),
    orange: Scheme::interpolate(0x482c0f, 0xd4812b, 63),
    yellow: Scheme::interpolate(0x756600, 0xffde00, 63),
    limegreen: Scheme::interpolate(0x2c4611, 0x80ce31, 63),
    green: Scheme::interpolate(0x186218, 0x32cd32, 63),
    bluegreen: Scheme::interpolate(0x206a52, 0x3bc49a, 63),
    cyan: Scheme::interpolate(0x0f2c48, 0x2bd4d4, 63),
    blue: Scheme::interpolate(0x162b41, 0x2b81d4, 63),
    deepblue: Scheme::interpolate(0x202083, 0x3232cd, 63),
    purple: Scheme::interpolate(0x4d008b, 0x8c00fd, 63),
    magenta: Scheme::interpolate(0x401640, 0xbd42bd, 63),
    redpink: Scheme::interpolate(0x47101d, 0xc33c5b, 63),
};
