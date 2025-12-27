use crate::Palette;
use ratatui_core::style::Color;

/// Base 16
///
/// Uses the classic 16 vga colors.
/// No gradients.
const DARKNESS: u8 = 63;

pub const BASE16: Palette = Palette {
    primary: fillin(0x00aa00, DARKNESS),
    secondary: fillin(0x00aaaa, DARKNESS),

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

    red: fillin(0xaa0000, DARKNESS),
    orange: fillin(0xaa5500, DARKNESS),
    yellow: fillin(0xffff55, DARKNESS),
    limegreen: fillin(0x55ff55, DARKNESS),
    green: fillin(0x00aa00, DARKNESS),
    bluegreen: fillin(0x55ffff, DARKNESS),
    cyan: fillin(0x00aaaa, DARKNESS),
    blue: fillin(0x5555ff, DARKNESS),
    deepblue: fillin(0x0000af, DARKNESS),
    purple: fillin(0xaa00aa, DARKNESS),
    magenta: fillin(0xff55ff, DARKNESS),
    redpink: fillin(0xff5555, DARKNESS),
};

const fn fillin(c0: u32, dark_scale_to: u8) -> [Color; 8] {
    let r0 = (c0 >> 16) as u8;
    let g0 = (c0 >> 8) as u8;
    let b0 = c0 as u8;

    // dark
    let r4 = Palette::scale_to(r0, dark_scale_to);
    let g4 = Palette::scale_to(g0, dark_scale_to);
    let b4 = Palette::scale_to(b0, dark_scale_to);

    [
        Color::Rgb(r0, g0, b0),
        Color::Rgb(r0, g0, b0),
        Color::Rgb(r0, g0, b0),
        Color::Rgb(r0, g0, b0),
        Color::Rgb(r4, g4, b4),
        Color::Rgb(r4, g4, b4),
        Color::Rgb(r4, g4, b4),
        Color::Rgb(r4, g4, b4),
    ]
}
