use crate::Scheme;

/// An adaption of nvchad's monochrome theme.
///
/// -- credit to original theme for existing : <https://github.com/kdheepak/monochrome.nvim>
/// -- NOTE: This is a modified version of it
const DARKNESS: u8 = 48;

pub const MONOCHROME: Scheme = Scheme {
    primary: Scheme::interpolate(0xb4b4b4, 0xf0f0f0, DARKNESS),
    secondary: Scheme::interpolate(0x424242, 0x777777, DARKNESS),

    white: Scheme::interpolate(0xb4b4b4, 0xf0f0f0, DARKNESS),
    black: Scheme::interpolate(0x1a1a1a, 0x383838, DARKNESS),
    gray: Scheme::interpolate(0x424242, 0x777777, DARKNESS),

    red: Scheme::interpolate(0xeda1a1, 0xede1e1, DARKNESS),
    orange: Scheme::interpolate(0xefb6a0, 0xf0e7e4, DARKNESS),
    yellow: Scheme::interpolate(0xffe6b5, 0xfffbf2, DARKNESS),
    limegreen: Scheme::interpolate(0xeff6ab, 0xf4f5e9, DARKNESS),
    green: Scheme::interpolate(0xcdd489, 0xd3d4c9, DARKNESS),
    bluegreen: Scheme::interpolate(0x8ac3d4, 0xc9d2d4, DARKNESS),
    cyan: Scheme::interpolate(0x8abae1, 0xd5dbe0, DARKNESS),
    blue: Scheme::interpolate(0xa5c6e1, 0xd5dbe0, DARKNESS),
    deepblue: Scheme::interpolate(0x95a9de, 0xd3d6de, DARKNESS),
    purple: Scheme::interpolate(0xd8b6e0, 0xded5e0, DARKNESS),
    magenta: Scheme::interpolate(0xc7a4cf, 0xcdc5cf, DARKNESS),
    redpink: Scheme::interpolate(0xeca8a8, 0xede1e1, DARKNESS),
};
