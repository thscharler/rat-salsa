use crate::Scheme;

pub const IMPERIAL: Scheme = Scheme {
    primary: Scheme::linear4(0x4d008b, 0x8c00fd),
    secondary: Scheme::linear4(0x8a7800, 0xffde00),

    white: Scheme::linear4(0xdedfe3, 0xf6f6f3),
    black: Scheme::linear4(0x0f1014, 0x2a2b37),
    gray: Scheme::linear4(0x3b3d4e, 0x6e7291),

    red: Scheme::linear4(0x480f0f, 0xd22d2d),
    orange: Scheme::linear4(0x482c0f, 0xd4812b),
    yellow: Scheme::linear4(0x756600, 0xffde00),
    limegreen: Scheme::linear4(0x2c4611, 0x80ce31),
    green: Scheme::linear4(0x186218, 0x32cd32),
    bluegreen: Scheme::linear4(0x206a52, 0x3bc49a),
    cyan: Scheme::linear4(0x0f2c48, 0x2bd4d4),
    blue: Scheme::linear4(0x162b41, 0x2b81d4),
    deepblue: Scheme::linear4(0x202083, 0x3232cd),
    purple: Scheme::linear4(0x4d008b, 0x8c00fd),
    magenta: Scheme::linear4(0x401640, 0xbd42bd),
    redpink: Scheme::linear4(0x47101d, 0xc33c5b),
};
