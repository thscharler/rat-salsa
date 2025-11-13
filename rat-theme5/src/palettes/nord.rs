use crate::Palette;

/// Nord
const DARKNESS: u8 = 63;

pub const NORD: Palette = Palette {
    name: "Nord", 

    text_dark: Palette::color32(0x3b4252), 
    text_black: Palette::color32(0x2e3440), 
    text_light: Palette::color32(0xe5e9f0), 
    text_bright: Palette::color32(0xeceff4), 

    primary: Palette::interpolate(0xd8dee9, 0xeceff4, DARKNESS), 
    secondary: Palette::interpolate(0x9ab3d3, 0x719bd3, DARKNESS), 

    white: Palette::interpolate(0xd8dee9, 0xeceff4, DARKNESS), 
    black: Palette::interpolate(0x2e3440, 0x3b4252, DARKNESS), 
    gray: Palette::interpolate(0x434c5e, 0x4c566a, DARKNESS), 
    red: Palette::interpolate(0xbf616a, 0xd66d77, DARKNESS), 
    orange: Palette::interpolate(0xd08770, 0xe8967d, DARKNESS), 
    yellow: Palette::interpolate(0xeadbbe, 0xffdd99, DARKNESS), 
    limegreen: Palette::interpolate(0xa3be8c, 0xb8d69e, DARKNESS), 
    green: Palette::interpolate(0x8fbcbb, 0xa2d3d2, DARKNESS), 
    bluegreen: Palette::interpolate(0x88c0d0, 0x99d6e8, DARKNESS), 
    cyan: Palette::interpolate(0x88c0d0, 0x99d6e8, DARKNESS), 
    blue: Palette::interpolate(0x81a1c1, 0x91b4d8, DARKNESS), 
    deepblue: Palette::interpolate(0x5e81ac, 0x6b93c4, DARKNESS), 
    purple: Palette::interpolate(0xb48ead, 0xcca1c4, DARKNESS), 
    magenta: Palette::interpolate(0xbf616a, 0xd66d77, DARKNESS), 
    redpink: Palette::interpolate(0xbf616a, 0xd66d77, DARKNESS), 
};
