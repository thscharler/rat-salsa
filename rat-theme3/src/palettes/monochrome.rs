use crate::Palette;

/// An adaption of nvchad's monochrome theme.
///
/// -- credit to original theme for existing : <https://github.com/kdheepak/monochrome.nvim>
/// -- NOTE: This is a modified version of it
const DARKNESS: u8 = 48;

pub const MONOCHROME: Palette = Palette {
    name: "Monochrome",

    primary: Palette::interpolate(0xb4b4b4, 0xf0f0f0, DARKNESS),
    secondary: Palette::interpolate(0x424242, 0x777777, DARKNESS),

    white: Palette::interpolate(0xb4b4b4, 0xf0f0f0, DARKNESS),
    black: Palette::interpolate(0x1a1a1a, 0x383838, DARKNESS),
    gray: Palette::interpolate(0x424242, 0x777777, DARKNESS),

    red: Palette::interpolate(0xeda1a1, 0xede1e1, DARKNESS),
    orange: Palette::interpolate(0xefb6a0, 0xf0e7e4, DARKNESS),
    yellow: Palette::interpolate(0xffe6b5, 0xfffbf2, DARKNESS),
    limegreen: Palette::interpolate(0xeff6ab, 0xf4f5e9, DARKNESS),
    green: Palette::interpolate(0xcdd489, 0xd3d4c9, DARKNESS),
    bluegreen: Palette::interpolate(0x8ac3d4, 0xc9d2d4, DARKNESS),
    cyan: Palette::interpolate(0x8abae1, 0xd5dbe0, DARKNESS),
    blue: Palette::interpolate(0xa5c6e1, 0xd5dbe0, DARKNESS),
    deepblue: Palette::interpolate(0x95a9de, 0xd3d6de, DARKNESS),
    purple: Palette::interpolate(0xd8b6e0, 0xded5e0, DARKNESS),
    magenta: Palette::interpolate(0xc7a4cf, 0xcdc5cf, DARKNESS),
    redpink: Palette::interpolate(0xeca8a8, 0xede1e1, DARKNESS),
};
