use crate::Palette;

/// An adaption of nvchad's vscode_dark theme.
///
/// -- Thanks to original theme for existing <https://github.com/microsoft/vscode/blob/main/extensions/theme-defaults/themes/dark_plus.json>
/// -- this is a modified version of it
pub const VSCODE_DARK: Palette = Palette {
    primary: Palette::interpolate(0xd4d4d4, 0xffffff, 63),
    secondary: Palette::interpolate(0x444444, 0x878787, 63),

    white: Palette::interpolate(0xd4d4d4, 0xffffff, 63),
    black: Palette::interpolate(0x1a1a1a, 0x3a3a3a, 63),
    gray: Palette::interpolate(0x444444, 0x878787, 63),

    red: Palette::interpolate(0xd0525c, 0xd16969, 63),
    orange: Palette::interpolate(0xd57e62, 0xd3967d, 63),
    yellow: Palette::interpolate(0xe0c485, 0xd7ba7d, 63),
    limegreen: Palette::interpolate(0x7dc94e, 0x9cda80, 63),
    green: Palette::interpolate(0x4ec994, 0x80daba, 63),
    bluegreen: Palette::interpolate(0x9cdc98, 0xb5cea8, 63),
    cyan: Palette::interpolate(0x8fd7ff, 0x9cdcfe, 63),
    blue: Palette::interpolate(0x60a6e0, 0x89beec, 63),
    deepblue: Palette::interpolate(0x4294d6, 0x85bae6, 63),
    purple: Palette::interpolate(0xb77bdf, 0xbd88ed, 63),
    magenta: Palette::interpolate(0xcb7dd4, 0xbb7cb6, 63),
    redpink: Palette::interpolate(0xea696f, 0xe98691, 63),
};
