use crate::Scheme;

/// My take on an ocean theme.
///
/// Primary is the color of a buoy marking off a spot.
/// Secondary is a vague blue.
///
/// Colors are either bleached or shifted towards blueish.
/// Overall it's not very dark, more of a sunny day than
/// a foggy, rainy night.
pub const OCEAN: Scheme = Scheme {
    primary: Scheme::interpolate(0xff8d3c, 0xffbf3c, 63),
    secondary: Scheme::interpolate(0x2b4779, 0x6688cc, 63),

    white: Scheme::interpolate(0xe5e5dd, 0xf2f2ee, 63),
    black: Scheme::interpolate(0x030305, 0x0c092c, 63),
    gray: Scheme::interpolate(0x4f6167, 0xbcc7cc, 63),

    red: Scheme::interpolate(0xff5e7f, 0xff9276, 63),
    orange: Scheme::interpolate(0xff9f5b, 0xffdc94, 63),
    yellow: Scheme::interpolate(0xffda5d, 0xfff675, 63),
    limegreen: Scheme::interpolate(0x7d8447, 0xe1e5b9, 63),
    green: Scheme::interpolate(0x658362, 0x99c794, 63),
    bluegreen: Scheme::interpolate(0x3a615c, 0x5b9c90, 63),
    cyan: Scheme::interpolate(0x24adbc, 0xb8dade, 63),
    blue: Scheme::interpolate(0x4f86ca, 0xbfdcff, 63),
    deepblue: Scheme::interpolate(0x2b4779, 0x6688cc, 63),
    purple: Scheme::interpolate(0x5068d7, 0xc7c4ff, 63),
    magenta: Scheme::interpolate(0x7952d6, 0xc9bde4, 63),
    redpink: Scheme::interpolate(0x9752d6, 0xcebde4, 63),
};
