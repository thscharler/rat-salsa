use crate::Palette;
use ratatui::style::Color;

pub const BLACKOUT: Palette = Palette {
    name: "Blackout",

    text_light: Palette::color32(0x000000),
    text_bright: Palette::color32(0x000000),
    text_dark: Palette::color32(0x000000),
    text_black: Palette::color32(0x000000),

    primary: fillin(0x000000),
    secondary: fillin(0x000000),

    white: [
        Color::Rgb(0xaa, 0xaa, 0x00),
        Color::Rgb(0x00, 0x00, 0x00),
        Color::Rgb(0x00, 0x00, 0x00),
        Color::Rgb(0x00, 0x00, 0x00),
        Color::Rgb(0x00, 0x00, 0x00),
        Color::Rgb(0x00, 0x00, 0x00),
        Color::Rgb(0x00, 0x00, 0x00),
        Color::Rgb(0x00, 0x00, 0x00),
    ],
    gray: [
        Color::Rgb(0x00, 0x00, 0x00),
        Color::Rgb(0x00, 0x00, 0x00),
        Color::Rgb(0x00, 0x00, 0x00),
        Color::Rgb(0x00, 0x00, 0x00),
        Color::Rgb(0x00, 0x00, 0x00),
        Color::Rgb(0x00, 0x00, 0x00),
        Color::Rgb(0x00, 0x00, 0x00),
        Color::Rgb(0x00, 0x00, 0x00),
    ],
    black: [
        Color::Rgb(0x00, 0x00, 0x00),
        Color::Rgb(0x00, 0x00, 0x00),
        Color::Rgb(0x00, 0x00, 0x00),
        Color::Rgb(0x00, 0x00, 0x00),
        Color::Rgb(0x00, 0x00, 0x00),
        Color::Rgb(0x00, 0x00, 0x00),
        Color::Rgb(0x00, 0x00, 0x00),
        Color::Rgb(0x00, 0x00, 0x00),
    ],

    red: fillin(0x000000),
    orange: fillin(0x000000),
    yellow: fillin(0x000000),
    limegreen: fillin(0x000000),
    green: fillin(0x000000),
    bluegreen: fillin(0x000000),
    cyan: fillin(0x000000),
    blue: fillin(0x000000),
    deepblue: fillin(0x000000),
    purple: fillin(0x000000),
    magenta: fillin(0x000000),
    redpink: fillin(0x000000),
};

const fn fillin(c0: u32) -> [Color; 8] {
    let r0 = (c0 >> 16) as u8;
    let g0 = (c0 >> 8) as u8;
    let b0 = c0 as u8;
    [
        Color::Rgb(r0, g0, b0),
        Color::Rgb(r0, g0, b0),
        Color::Rgb(r0, g0, b0),
        Color::Rgb(r0, g0, b0),
        Color::Rgb(r0, g0, b0),
        Color::Rgb(r0, g0, b0),
        Color::Rgb(r0, g0, b0),
        Color::Rgb(r0, g0, b0),
    ]
}
