use crate::Scheme;
use ratatui::style::Color;

/// Base 16
///
/// Uses the classic 16 vga colors.
/// No gradients.
///
pub const BASE16: Scheme = Scheme {
    primary: Scheme::linear4(0x00aa00, 0x00aa00),
    secondary: Scheme::linear4(0x00aaaa, 0x00aaaa),

    white: [
        Color::Rgb(0xaa, 0xaa, 0xaa),
        Color::Rgb(0xaa, 0xaa, 0xaa),
        Color::Rgb(0xff, 0xff, 0xff),
        Color::Rgb(0xff, 0xff, 0xff),
    ],
    gray: [
        Color::Rgb(0x55, 0x55, 0x55),
        Color::Rgb(0x55, 0x55, 0x55),
        Color::Rgb(0xaa, 0xaa, 0xaa),
        Color::Rgb(0xaa, 0xaa, 0xaa),
    ],
    black: [
        Color::Rgb(0x00, 0x00, 0x00),
        Color::Rgb(0x00, 0x00, 0x00),
        Color::Rgb(0x55, 0x55, 0x55),
        Color::Rgb(0x55, 0x55, 0x55),
    ],

    red: Scheme::linear4(0xaa0000, 0xaa0000),
    orange: Scheme::linear4(0xaa5500, 0xaa5500),
    yellow: Scheme::linear4(0xffff55, 0xffff55),
    limegreen: Scheme::linear4(0x55ff55, 0x55ff55),
    green: Scheme::linear4(0x00aa00, 0x00aa00),
    bluegreen: Scheme::linear4(0x55ffff, 0x55ffff),
    cyan: Scheme::linear4(0x00aaaa, 0x00aaaa),
    blue: Scheme::linear4(0x5555ff, 0x5555ff),
    deepblue: Scheme::linear4(0x0000af, 0x0000af),
    purple: Scheme::linear4(0xaa00aa, 0xaa00aa),
    magenta: Scheme::linear4(0xff55ff, 0xff55ff),
    redpink: Scheme::linear4(0xff5555, 0xff5555),
};
