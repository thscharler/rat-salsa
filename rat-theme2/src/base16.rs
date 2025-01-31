use crate::Scheme;
use ratatui::prelude::Color;

/// Base 16
///
/// Uses the classic 16 vga colors.
/// No gradients.
const DARKNESS: u8 = 63;

pub const BASE16: Scheme = Scheme {
    primary: Scheme::interpolate(0x00aa00, 0x00aa00, DARKNESS),
    secondary: Scheme::interpolate(0x00aaaa, 0x00aaaa, DARKNESS),

    white: [
        Color::Rgb(0xaa, 0xaa, 0xaa),
        Color::Rgb(0xaa, 0xaa, 0xaa),
        Color::Rgb(0xff, 0xff, 0xff),
        Color::Rgb(0xff, 0xff, 0xff),
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
        Color::Rgb(0x00, 0x00, 0x00),
        Color::Rgb(0x00, 0x00, 0x00),
        Color::Rgb(0x55, 0x55, 0x55),
        Color::Rgb(0x55, 0x55, 0x55),
    ],

    red: Scheme::interpolate(0xaa0000, 0xaa0000, DARKNESS),
    orange: Scheme::interpolate(0xaa5500, 0xaa5500, DARKNESS),
    yellow: Scheme::interpolate(0xffff55, 0xffff55, DARKNESS),
    limegreen: Scheme::interpolate(0x55ff55, 0x55ff55, DARKNESS),
    green: Scheme::interpolate(0x00aa00, 0x00aa00, DARKNESS),
    bluegreen: Scheme::interpolate(0x55ffff, 0x55ffff, DARKNESS),
    cyan: Scheme::interpolate(0x00aaaa, 0x00aaaa, DARKNESS),
    blue: Scheme::interpolate(0x5555ff, 0x5555ff, DARKNESS),
    deepblue: Scheme::interpolate(0x0000af, 0x0000af, DARKNESS),
    purple: Scheme::interpolate(0xaa00aa, 0xaa00aa, DARKNESS),
    magenta: Scheme::interpolate(0xff55ff, 0xff55ff, DARKNESS),
    redpink: Scheme::interpolate(0xff5555, 0xff5555, DARKNESS),
};
