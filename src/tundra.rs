use crate::Scheme;

/// An adaption of nvchad's tundra theme.
///
/// -- Thanks to original theme for existing https://github.com/sam4llis/nvim-tundra
/// -- this is a modified version of it
pub const TUNDRA: Scheme = Scheme {
    primary: Scheme::linear4(0xe6eaf2, 0xffffff),
    secondary: Scheme::linear4(0xa8bbd4, 0x719bd3),

    white: Scheme::linear4(0xe6eaf2, 0xffffff),
    black: Scheme::linear4(0x0b1221, 0x1a2130),
    gray: Scheme::linear4(0x3e4554, 0x5f6675),

    red: Scheme::linear4(0xfccaca, 0xfca5a5),
    orange: Scheme::linear4(0xfad9c5, 0xfbc19d),
    yellow: Scheme::linear4(0xe8d7b7, 0xe8d4b0),
    limegreen: Scheme::linear4(0xbce8b7, 0xb5e8b0),
    green: Scheme::linear4(0xbce8b7, 0xb5e8b0),
    bluegreen: Scheme::linear4(0xa8bbd4, 0x719bd3),
    cyan: Scheme::linear4(0xc8eafc, 0xbae6fd),
    blue: Scheme::linear4(0xc7d0fc, 0xa5b4fc),
    deepblue: Scheme::linear4(0xbfcaf2ff, 0x9baaf2),
    purple: Scheme::linear4(0xb7abd9, 0xb3a6da),
    magenta: Scheme::linear4(0xffc9c9, 0xff8e8e),
    redpink: Scheme::linear4(0xfffcad0, 0xfecdd3),
};
