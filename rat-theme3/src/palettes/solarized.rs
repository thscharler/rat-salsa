use crate::Palette;

/// An adaption of nvchad's tundra theme.
///
/// -- Thanks to original theme for existing <https://github.com/sam4llis/nvim-tundra>
/// -- this is a modified version of it
pub const SOLARIZED: Palette = Palette {
    name: "Solarized",

    primary: Palette::interpolate(0xcb4b16, 0xb4542d, 63),
    secondary: Palette::interpolate(0x2aa198, 0x3e8e88, 63),

    white: Palette::interpolate(0xeee8d5, 0xfdf6e3, 63),
    black: Palette::interpolate(0x002b36, 0x073642, 63),
    gray: Palette::interpolate(0x586e75, 0x93a1a1, 63),

    red: Palette::interpolate(0xdc322f, 0xc34947, 63),
    orange: Palette::interpolate(0xcb4b16, 0xb4542d, 63),
    yellow: Palette::interpolate(0xb58900, 0xa17d12, 63),
    limegreen: Palette::interpolate(0xb2c62d, 0xa1af46, 63),
    green: Palette::interpolate(0x859900, 0x7a8a0f, 63),
    bluegreen: Palette::interpolate(0x519aba, 0x698fa0, 63),
    cyan: Palette::interpolate(0x2aa198, 0x3e8e88, 63),
    blue: Palette::interpolate(0x268bd2, 0x3f88ba, 63),
    deepblue: Palette::interpolate(0x197ec5, 0x317bb0, 63),
    purple: Palette::interpolate(0x6c71c4, 0x8184b1, 63),
    magenta: Palette::interpolate(0xd33682, 0xba4f83, 63),
    redpink: Palette::interpolate(0xeb413e, 0xd55553, 63),
};
