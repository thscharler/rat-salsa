use crate::Scheme;

/// An adaption of nvchad's radium theme.
///
/// -- credits to original radium theme from https://github.com/dharmx
pub const RADIUM: Scheme = Scheme {
    primary: Scheme::linear4(0x21b07c, 0x37d99e),
    secondary: Scheme::linear4(0x9759b5, 0xb68acb),

    white: Scheme::linear4(0xc4c4c5, 0xd4d4d5),
    black: Scheme::linear4(0x101317, 0x292c30),
    gray: Scheme::linear4(0x3e4145, 0x525559),

    red: Scheme::linear4(0xf64b4b, 0xf87070),
    orange: Scheme::linear4(0xe6723d, 0xf0a988),
    yellow: Scheme::linear4(0xffc424, 0xffe59e),
    limegreen: Scheme::linear4(0x42cc88, 0x92e2ba),
    green: Scheme::linear4(0x21b07c, 0x37d99e),
    bluegreen: Scheme::linear4(0x41cd86, 0x79dcaa),
    cyan: Scheme::linear4(0x2ca3aa, 0x50cad2),
    blue: Scheme::linear4(0x2b72b1, 0x7ab0df),
    deepblue: Scheme::linear4(0x4297e1, 0x87bdec),
    purple: Scheme::linear4(0x9759b5, 0xb68acb),
    magenta: Scheme::linear4(0xff5c5c, 0xff8e8e),
    redpink: Scheme::linear4(0xff7575, 0xffa7a7),
};
