use crate::Palette;

/// An adaption of nvchad's monochrome theme.
///
/// -- Credits to original theme <https://monokai.pro/>
/// -- This is modified version of it
pub const MONEKAI: Palette = Palette {
    name: "Monekai",

    text_light: Palette::color32(0xc9c9c0),
    text_bright: Palette::color32(0xf5f4f1),
    text_dark: Palette::color32(0x464741),
    text_black: Palette::color32(0x272822),

    primary: Palette::interpolate(0x80133a, 0xd12060, 63),
    secondary: Palette::interpolate(0x5e748c, 0x81a1c1, 63),

    white: Palette::interpolate(0xb0b2a8, 0xf5f4f1, 63),
    gray: Palette::interpolate(0x4d4e48, 0x64655f, 63),
    black: Palette::interpolate(0x272822, 0x464741, 63),

    red: Palette::interpolate(0x804c10, 0xfd971f, 63),
    orange: Palette::interpolate(0x584180, 0xae81ff, 63),
    yellow: Palette::interpolate(0x80643d, 0xf4bf75, 63),
    limegreen: Palette::interpolate(0x5e801a, 0xa6e22e, 63),
    green: Palette::interpolate(0x628043, 0x96c367, 63),
    bluegreen: Palette::interpolate(0x207580, 0x34bfd0, 63),
    cyan: Palette::interpolate(0x235d80, 0x41afef, 63),
    blue: Palette::interpolate(0x2f668c, 0x51afef, 63),
    deepblue: Palette::interpolate(0x5e748c, 0x81a1c1, 63),
    purple: Palette::interpolate(0x764980, 0xb26fc1, 63),
    magenta: Palette::interpolate(0x80133a, 0xf92672, 63),
    redpink: Palette::interpolate(0x804020, 0xcc6633, 63),
};
