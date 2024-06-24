use crate::Scheme;

/// An adaption of nvchad's monochrome theme.
///
/// -- credit to original theme for existing : https://github.com/kdheepak/monochrome.nvim
/// -- NOTE: This is a modified version of it
pub const MONOCHROME: Scheme = Scheme {
    primary: Scheme::linear4(0xe6eaf2, 0xffffff),
    secondary: Scheme::linear4(0xa8bbd4, 0x719bd3),

    white: Scheme::linear4(0x677777, 0xd8dee9),
    black: Scheme::linear4(0x1a1a1a, 0x383838),
    gray: Scheme::linear4(0x424242, 0x677777),

    red: Scheme::linear4(0xeda1a1, 0xede1e1),
    orange: Scheme::linear4(0xefb6a0, 0xf0e7e4),
    yellow: Scheme::linear4(0xffe6b5, 0xfffbf2),
    limegreen: Scheme::linear4(0xeff6ab, 0xf4f5e9),
    green: Scheme::linear4(0xcdd489, 0xd3d4c9),
    bluegreen: Scheme::linear4(0x8ac3d4, 0xc9d2d4),
    cyan: Scheme::linear4(0x8abae1, 0xd5dbe0),
    blue: Scheme::linear4(0xa5c6e1, 0xd5dbe0),
    deepblue: Scheme::linear4(0x95a9de, 0xd3d6de),
    purple: Scheme::linear4(0xd8b6e0, 0xded5e0),
    magenta: Scheme::linear4(0xc7a4cf, 0xcdc5cf),
    redpink: Scheme::linear4(0xeca8a8, 0xede1e1),
};
