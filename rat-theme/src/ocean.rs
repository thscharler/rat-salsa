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
    primary: Scheme::linear4(0xff8d3c, 0xffbf3c),
    secondary: Scheme::linear4(0x2b4779, 0x6688cc),

    white: Scheme::linear4(0xe5e5dd, 0xf2f2ee),
    black: Scheme::linear4(0x030305, 0x0c092c),
    gray: Scheme::linear4(0x4f6167, 0xbcc7cc),

    red: Scheme::linear4(0xff5e7f, 0xff9276),
    orange: Scheme::linear4(0xff9f5b, 0xffdc94),
    yellow: Scheme::linear4(0xffda5d, 0xfff675),
    limegreen: Scheme::linear4(0x7d8447, 0xe1e5b9),
    green: Scheme::linear4(0x658362, 0x99c794),
    bluegreen: Scheme::linear4(0x3a615c, 0x5b9c90),
    cyan: Scheme::linear4(0x24adbc, 0xb8dade),
    blue: Scheme::linear4(0x4f86ca, 0xbfdcff),
    deepblue: Scheme::linear4(0x2b4779, 0x6688cc),
    purple: Scheme::linear4(0x5068d7, 0xc7c4ff),
    magenta: Scheme::linear4(0x7952d6, 0xc9bde4),
    redpink: Scheme::linear4(0x9752d6, 0xcebde4),
};
