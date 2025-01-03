use crate::Scheme;

/// An adaption of nvchad's monochrome theme.
///
/// -- Credits to original theme <https://monokai.pro/>
/// -- This is modified version of it
pub const MONEKAI: Scheme = Scheme {
    primary: Scheme::linear4(0x80133a, 0xd12060),
    secondary: Scheme::linear4(0x5e748c, 0x81a1c1),

    white: Scheme::linear4(0xb0b2a8, 0xf5f4f1),
    gray: Scheme::linear4(0x4d4e48, 0x64655f),
    black: Scheme::linear4(0x272822, 0x464741),

    red: Scheme::linear4(0x804c10, 0xfd971f),
    orange: Scheme::linear4(0x584180, 0xae81ff),
    yellow: Scheme::linear4(0x80643d, 0xf4bf75),
    limegreen: Scheme::linear4(0x5e801a, 0xa6e22e),
    green: Scheme::linear4(0x628043, 0x96c367),
    bluegreen: Scheme::linear4(0x207580, 0x34bfd0),
    cyan: Scheme::linear4(0x235d80, 0x41afef),
    blue: Scheme::linear4(0x2f668c, 0x51afef),
    deepblue: Scheme::linear4(0x5e748c, 0x81a1c1),
    purple: Scheme::linear4(0x764980, 0xb26fc1),
    magenta: Scheme::linear4(0x80133a, 0xf92672),
    redpink: Scheme::linear4(0x804020, 0xcc6633),
};
