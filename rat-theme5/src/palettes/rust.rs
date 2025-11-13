use crate::Palette;

/// Rust
const DARKNESS: u8 = 63;

pub const RUST: Palette = Palette {
    name: "Rust", 

    text_dark: Palette::color32(0x161514), 
    text_black: Palette::color32(0x0f0e0d), 
    text_light: Palette::color32(0xd1ccc8), 
    text_bright: Palette::color32(0xefe6e6), 

    primary: Palette::interpolate(0x75311a, 0xd25a32, DARKNESS), 
    secondary: Palette::interpolate(0x77551d, 0xcd9537, DARKNESS), 

    white: Palette::interpolate(0xc4bfbb, 0xede3e3, DARKNESS), 
    black: Palette::interpolate(0x101011, 0x464251, DARKNESS), 
    gray: Palette::interpolate(0x726e6b, 0xa39d99, DARKNESS), 
    red: Palette::interpolate(0x75311a, 0xd25a32, DARKNESS), 
    orange: Palette::interpolate(0x75431a, 0xd27a32, DARKNESS), 
    yellow: Palette::interpolate(0x77551d, 0xcd9537, DARKNESS), 
    limegreen: Palette::interpolate(0x44664d, 0x699b76, DARKNESS), 
    green: Palette::interpolate(0x44664d, 0x699b76, DARKNESS), 
    bluegreen: Palette::interpolate(0x1a7574, 0x32d2d1, DARKNESS), 
    cyan: Palette::interpolate(0x1a7574, 0x32d2d1, DARKNESS), 
    blue: Palette::interpolate(0x005d94, 0x38b6ff, DARKNESS), 
    deepblue: Palette::interpolate(0x005d94, 0x38b6ff, DARKNESS), 
    purple: Palette::interpolate(0x722234, 0xc63f5d, DARKNESS), 
    magenta: Palette::interpolate(0x7b1964, 0xd332ad, DARKNESS), 
    redpink: Palette::interpolate(0x7b1964, 0xd332ad, DARKNESS), 
};
