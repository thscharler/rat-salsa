use crate::Scheme;

/// An adaption of nvchad's vscode_dark theme.
///
/// -- Thanks to original theme for existing https://github.com/microsoft/vscode/blob/main/extensions/theme-defaults/themes/dark_plus.json
/// -- this is a modified version of it
pub const VSCODE_DARK: Scheme = Scheme {
    primary: Scheme::linear4(0xd4d4d4, 0xffffff),
    secondary: Scheme::linear4(0x444444, 0x878787),

    white: Scheme::linear4(0xd4d4d4, 0xffffff),
    black: Scheme::linear4(0x1a1a1a, 0x3a3a3a),
    gray: Scheme::linear4(0x444444, 0x878787),

    red: Scheme::linear4(0xd0525c, 0xd16969),
    orange: Scheme::linear4(0xd57e62, 0xd3967d),
    yellow: Scheme::linear4(0xe0c485, 0xd7ba7d),
    limegreen: Scheme::linear4(0x7dc94e, 0x9cda80),
    green: Scheme::linear4(0x4ec994, 0x80daba),
    bluegreen: Scheme::linear4(0x9cdc98, 0xb5cea8),
    cyan: Scheme::linear4(0x8fd7ff, 0x9cdcfe),
    blue: Scheme::linear4(0x60a6e0, 0x89beec),
    deepblue: Scheme::linear4(0x4294d6, 0x85bae6),
    purple: Scheme::linear4(0xb77bdf, 0xbd88ed),
    magenta: Scheme::linear4(0xcb7dd4, 0xbb7cb6),
    redpink: Scheme::linear4(0xea696f, 0xe98691),
};
