use crate::Scheme;

/// An adaption of nvchad's monochrome theme.
///
/// -- credit to original theme for existing : <https://github.com/kdheepak/monochrome.nvim>
/// -- NOTE: This is a modified version of it
pub const MONOCHROME: Scheme = Scheme {
    primary: Scheme::interpolate(0xe6eaf2, 0xffffff, 63),
    secondary: Scheme::interpolate(0xa8bbd4, 0x719bd3, 63),

    white: Scheme::interpolate(0x677777, 0xd8dee9, 63),
    black: Scheme::interpolate(0x1a1a1a, 0x383838, 63),
    gray: Scheme::interpolate(0x424242, 0x677777, 63),

    red: Scheme::interpolate(0xeda1a1, 0xede1e1, 63),
    orange: Scheme::interpolate(0xefb6a0, 0xf0e7e4, 63),
    yellow: Scheme::interpolate(0xffe6b5, 0xfffbf2, 63),
    limegreen: Scheme::interpolate(0xeff6ab, 0xf4f5e9, 63),
    green: Scheme::interpolate(0xcdd489, 0xd3d4c9, 63),
    bluegreen: Scheme::interpolate(0x8ac3d4, 0xc9d2d4, 63),
    cyan: Scheme::interpolate(0x8abae1, 0xd5dbe0, 63),
    blue: Scheme::interpolate(0xa5c6e1, 0xd5dbe0, 63),
    deepblue: Scheme::interpolate(0x95a9de, 0xd3d6de, 63),
    purple: Scheme::interpolate(0xd8b6e0, 0xded5e0, 63),
    magenta: Scheme::interpolate(0xc7a4cf, 0xcdc5cf, 63),
    redpink: Scheme::interpolate(0xeca8a8, 0xede1e1, 63),
};
