use crate::Scheme;

/// An adaption of nvchad's radium theme.
///
/// -- credits to original radium theme from <https://github.com/dharmx>
pub const RADIUM: Scheme = Scheme {
    primary: Scheme::interpolate(0x21b07c, 0x37d99e, 63),
    secondary: Scheme::interpolate(0x9759b5, 0xb68acb, 63),

    white: Scheme::interpolate(0xc4c4c5, 0xd4d4d5, 63),
    black: Scheme::interpolate(0x101317, 0x292c30, 63),
    gray: Scheme::interpolate(0x3e4145, 0x525559, 63),

    red: Scheme::interpolate(0xf64b4b, 0xf87070, 63),
    orange: Scheme::interpolate(0xe6723d, 0xf0a988, 63),
    yellow: Scheme::interpolate(0xffc424, 0xffe59e, 63),
    limegreen: Scheme::interpolate(0x42cc88, 0x92e2ba, 63),
    green: Scheme::interpolate(0x21b07c, 0x37d99e, 63),
    bluegreen: Scheme::interpolate(0x41cd86, 0x79dcaa, 63),
    cyan: Scheme::interpolate(0x2ca3aa, 0x50cad2, 63),
    blue: Scheme::interpolate(0x2b72b1, 0x7ab0df, 63),
    deepblue: Scheme::interpolate(0x4297e1, 0x87bdec, 63),
    purple: Scheme::interpolate(0x9759b5, 0xb68acb, 63),
    magenta: Scheme::interpolate(0xff5c5c, 0xff8e8e, 63),
    redpink: Scheme::interpolate(0xff7575, 0xffa7a7, 63),
};
