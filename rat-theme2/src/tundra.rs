use crate::Scheme;

/// An adaption of nvchad's tundra theme.
///
/// -- Thanks to original theme for existing <https://github.com/sam4llis/nvim-tundra>
/// -- this is a modified version of it
pub const TUNDRA: Scheme = Scheme {
    primary: Scheme::interpolate(0xe6eaf2, 0xffffff, 63),
    secondary: Scheme::interpolate(0xa8bbd4, 0x719bd3, 63),

    white: Scheme::interpolate(0xe6eaf2, 0xffffff, 63),
    black: Scheme::interpolate(0x0b1221, 0x1a2130, 63),
    gray: Scheme::interpolate(0x3e4554, 0x5f6675, 63),

    red: Scheme::interpolate(0xfccaca, 0xfca5a5, 63),
    orange: Scheme::interpolate(0xfad9c5, 0xfbc19d, 63),
    yellow: Scheme::interpolate(0xe8d7b7, 0xe8d4b0, 63),
    limegreen: Scheme::interpolate(0xbce8b7, 0xb5e8b0, 63),
    green: Scheme::interpolate(0xbce8b7, 0xb5e8b0, 63),
    bluegreen: Scheme::interpolate(0xa8bbd4, 0x719bd3, 63),
    cyan: Scheme::interpolate(0xc8eafc, 0xbae6fd, 63),
    blue: Scheme::interpolate(0xc7d0fc, 0xa5b4fc, 63),
    deepblue: Scheme::interpolate(0xbfcaf2, 0x9baaf2, 63),
    purple: Scheme::interpolate(0xb7abd9, 0xb3a6da, 63),
    magenta: Scheme::interpolate(0xffc9c9, 0xff8e8e, 63),
    redpink: Scheme::interpolate(0xfffcad, 0xfecdd3, 63),
};
