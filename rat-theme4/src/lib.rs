//!
//! SalsaTheme provides a styling system for ratatui apps.
//!
//! It has a simple flat naming scheme.
//!
//! But it can store
//! * [ratatui Style](ratatui::style::Style)
//! * composite styles as used by [rat-widget](rat_widget).
//!   eg [CheckboxStyle](rat_widget::checkbox::CheckboxStyle)
//! * practically anything else.
//!
//! ## Naming styles
//!
//! * It has an extension trait for [Style](ratatui::style::Style) that
//!   adds constants for known styles. In the same manner you can add your
//!   application specific styles and have them with code completion.
//!
//! * For [rat-widget](rat_widget) composite style it defines an anchor struct
//!   [WidgetStyle] that performs the same purpose.
//!
//! ## Usage
//!
//! ```rust
//! # use ratatui::buffer::Buffer;
//! # use ratatui::layout::Rect;
//! # use ratatui::style::Style;
//! # use ratatui::widgets::StatefulWidget;
//! # use rat_theme4::theme::{SalsaTheme};
//! # use rat_theme4::{StyleName, WidgetStyle};
//! # use rat_theme4::palettes::dark::BLACKOUT;
//! # use rat_widget::checkbox::{Checkbox, CheckboxState, CheckboxStyle};
//! # let theme = SalsaTheme::default();
//! # let area = Rect::default();
//! # let mut buf = Buffer::default();
//! # let buf = &mut buf;
//! # let mut state = CheckboxState::default();
//!
//! // ratatui Style
//! let s: Style = theme.style(Style::SELECT);
//!
//! // composite style
//! Checkbox::new()
//!     .styles(theme.style(WidgetStyle::CHECKBOX))
//!     .render(area, buf, &mut state);
//! ```
//!
//! ## Palette
//!
//! Palette holds the color definitions and aliases for the
//! colors. This is the part of the theme that can be persisted.
//! It can be stored/loaded from file or put into a `static`.
//!
//! With [create_palette_theme] the theme can be reconstructed.
//!

use crate::palette::{Colors, Palette};
use crate::palettes::shell;
use crate::theme::SalsaTheme;
use ratatui::style::{Color, Style};
use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;
use std::io::ErrorKind;
use std::sync::atomic::{AtomicBool, Ordering};

pub mod palette;
pub mod theme;

/// Currently shipped palettes.
pub mod palettes {
    pub mod core;
    pub mod dark;
    pub mod light;
    pub mod shell;
}

pub mod themes {
    mod dark;
    mod fallback;
    mod light;
    mod shell;

    /// Creates a `dark` theme.
    pub use dark::create_dark;
    /// Creates a 'light' theme.
    pub use light::create_light;
    /// Creates a `shell` theme. This uses the dark palettes,
    /// but sets almost no backgrounds. Instead, it lets the
    /// terminal background shine.
    pub use shell::create_shell;

    /// Create the `fallback` theme.
    /// This is more for testing widgets than anything else.
    /// It just uses `Default::default()` for any style.
    /// This helps to check if a widget is still functional
    /// if no styling is applied.
    pub use fallback::create_fallback;
}

/// Anchor struct for the names of composite styles used
/// by rat-widget's.
///
/// Use as
/// ```rust
/// # use ratatui::style::Style;
/// # use rat_theme4::theme::{SalsaTheme};
/// # use rat_theme4::{ StyleName, WidgetStyle};
/// # use rat_theme4::palettes::dark::BLACKOUT;
/// # use rat_widget::checkbox::CheckboxStyle;
/// # let theme = SalsaTheme::default();
///
/// let s: CheckboxStyle = theme.style(WidgetStyle::CHECKBOX);
/// ```
/// or more likely
/// ```rust
/// # use ratatui::buffer::Buffer;
/// # use ratatui::layout::Rect;
/// # use ratatui::style::Style;
/// # use ratatui::widgets::StatefulWidget;
/// # use rat_theme4::theme::{SalsaTheme};
/// # use rat_theme4::{ StyleName, WidgetStyle};
/// # use rat_theme4::palettes::dark::BLACKOUT;
/// # use rat_widget::checkbox::{Checkbox, CheckboxState, CheckboxStyle};
/// # let theme = SalsaTheme::default();
/// # let area = Rect::default();
/// # let mut buf = Buffer::default();
/// # let buf = &mut buf;
/// # let mut state = CheckboxState::default();
///
/// Checkbox::new()
///     .styles(theme.style(WidgetStyle::CHECKBOX))
///     .render(area, buf, &mut state);
/// ```
pub struct WidgetStyle;

impl WidgetStyle {
    pub const BUTTON: &'static str = "button";
    pub const CALENDAR: &'static str = "calendar";
    pub const CHECKBOX: &'static str = "checkbox";
    pub const CHOICE: &'static str = "choice";
    pub const CLIPPER: &'static str = "clipper";
    #[cfg(feature = "color_input")]
    pub const COLOR_INPUT: &'static str = "color-input";
    pub const COMBOBOX: &'static str = "combobox";
    pub const DIALOG_FRAME: &'static str = "dialog-frame";
    pub const FILE_DIALOG: &'static str = "file-dialog";
    pub const FORM: &'static str = "form";
    pub const LINE_NR: &'static str = "line-nr";
    pub const LIST: &'static str = "list";
    pub const MENU: &'static str = "menu";
    pub const MONTH: &'static str = "month";
    pub const MSG_DIALOG: &'static str = "msg-dialog";
    pub const PARAGRAPH: &'static str = "paragraph";
    pub const RADIO: &'static str = "radio";
    pub const SCROLL: &'static str = "scroll";
    pub const SCROLL_DIALOG: &'static str = "scroll.dialog";
    pub const SCROLL_POPUP: &'static str = "scroll.popup";
    pub const SHADOW: &'static str = "shadow";
    pub const SLIDER: &'static str = "slider";
    pub const SPLIT: &'static str = "split";
    pub const STATUSLINE: &'static str = "statusline";
    pub const TABBED: &'static str = "tabbed";
    pub const TABLE: &'static str = "table";
    pub const TEXT: &'static str = "text";
    pub const TEXTAREA: &'static str = "textarea";
    pub const TEXTVIEW: &'static str = "textview";
    pub const VIEW: &'static str = "view";
}

/// Extension trait for [Style](ratatui::style::Style) that defines
/// some standard names used by rat-theme/rat-widget
///
/// Use as
/// ```rust
/// # use ratatui::style::Style;
/// # use rat_theme4::theme::{SalsaTheme};
/// # use rat_theme4::{ StyleName, WidgetStyle};
/// # use rat_theme4::palettes::dark::BLACKOUT;
/// # let theme = SalsaTheme::default();
///
/// let s: Style = theme.style(Style::INPUT);
/// ```
pub trait StyleName {
    const LABEL_FG: &'static str = "label-fg";
    const INPUT: &'static str = "input";
    const INPUT_FOCUS: &'static str = "text-focus";
    const INPUT_SELECT: &'static str = "text-select";
    const FOCUS: &'static str = "focus";
    const SELECT: &'static str = "select";
    const DISABLED: &'static str = "disabled";
    const INVALID: &'static str = "invalid";

    const TITLE: &'static str = "title";
    const HEADER: &'static str = "header";
    const FOOTER: &'static str = "footer";

    const HOVER: &'static str = "hover";
    const SHADOWS: &'static str = "shadows";

    const WEEK_HEADER_FG: &'static str = "week-header-fg";
    const MONTH_HEADER_FG: &'static str = "month-header-fg";

    const KEY_BINDING: &'static str = "key-binding";
    const BUTTON_BASE: &'static str = "button-base";
    const MENU_BASE: &'static str = "menu-base";
    const STATUS_BASE: &'static str = "status-base";

    const CONTAINER_BASE: &'static str = "container-base";
    const CONTAINER_BORDER_FG: &'static str = "container-border-fg";
    const CONTAINER_ARROW_FG: &'static str = "container-arrows-fg";

    const DOCUMENT_BASE: &'static str = "document-base";
    const DOCUMENT_BORDER_FG: &'static str = "document-border-fg";
    const DOCUMENT_ARROW_FG: &'static str = "document-arrows-fg";

    const POPUP_BASE: &'static str = "popup-base";
    const POPUP_BORDER_FG: &'static str = "popup-border-fg";
    const POPUP_ARROW_FG: &'static str = "popup-arrow-fg";

    const DIALOG_BASE: &'static str = "dialog-base";
    const DIALOG_BORDER_FG: &'static str = "dialog-border-fg";
    const DIALOG_ARROW_FG: &'static str = "dialog-arrow-fg";
}
impl StyleName for Style {}

///
/// Extension trait for [Color](ratatui::style::Color) that defines
/// standard names used by rat-theme to define color-aliases.
///
/// Use as
/// ```rust
/// # use ratatui::style::{Style, Color};
/// # use rat_theme4::theme::{SalsaTheme};
/// # use rat_theme4::RatWidgetColor;
/// # let theme = SalsaTheme::default();
///
/// let c: Color = theme.p.color_alias(Color::LABEL_FG);
/// ```
pub trait RatWidgetColor {
    const LABEL_FG: &'static str = "label.fg";
    const INPUT_BG: &'static str = "input.bg";
    const INPUT_FOCUS_BG: &'static str = "input-focus.bg";
    const INPUT_SELECT_BG: &'static str = "input-select.bg";
    const FOCUS_BG: &'static str = "focus.bg";
    const SELECT_BG: &'static str = "select.bg";
    const DISABLED_BG: &'static str = "disabled.bg";
    const INVALID_BG: &'static str = "invalid.bg";

    const TITLE_FG: &'static str = "title.fg";
    const TITLE_BG: &'static str = "title.bg";
    const HEADER_FG: &'static str = "header.fg";
    const HEADER_BG: &'static str = "header.bg";
    const FOOTER_FG: &'static str = "footer.fg";
    const FOOTER_BG: &'static str = "footer.bg";

    const HOVER_BG: &'static str = "hover.bg";
    const BUTTON_BASE_BG: &'static str = "button-base.bg";
    const KEY_BINDING_BG: &'static str = "key-binding.bg";
    const MENU_BASE_BG: &'static str = "menu-base.bg";
    const STATUS_BASE_BG: &'static str = "status-base.bg";
    const SHADOW_BG: &'static str = "shadow.bg";

    const WEEK_HEADER_FG: &'static str = "week-header.fg";
    const MONTH_HEADER_FG: &'static str = "month-header.fg";

    const CONTAINER_BASE_BG: &'static str = "container-base.bg";
    const CONTAINER_BORDER_FG: &'static str = "container-border.fg";
    const CONTAINER_ARROW_FG: &'static str = "container-arrow.fg";
    const DOCUMENT_BASE_BG: &'static str = "document-base.bg";
    const DOCUMENT_BORDER_FG: &'static str = "document-border.fg";
    const DOCUMENT_ARROW_FG: &'static str = "document-arrow.fg";
    const POPUP_BASE_BG: &'static str = "popup-base.bg";
    const POPUP_BORDER_FG: &'static str = "popup-border.fg";
    const POPUP_ARROW_FG: &'static str = "popup-arrow.fg";
    const DIALOG_BASE_BG: &'static str = "dialog-base.bg";
    const DIALOG_BORDER_FG: &'static str = "dialog-border.fg";
    const DIALOG_ARROW_FG: &'static str = "dialog-arrow.fg";
}
impl RatWidgetColor for Color {}

static LOG_DEFINES: AtomicBool = AtomicBool::new(false);

/// Log style definition.
/// May help debugging styling problems ...
pub fn log_style_define(log: bool) {
    LOG_DEFINES.store(log, Ordering::Release);
}

fn is_log_style_define() -> bool {
    LOG_DEFINES.load(Ordering::Acquire)
}

#[derive(Debug)]
pub struct LoadPaletteErr(u8);

impl Display for LoadPaletteErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "load palette failed: {}", self.0)
    }
}

impl Error for LoadPaletteErr {}

/// Stora a Palette as a .pal file.
pub fn store_palette(pal: &Palette, mut buf: impl io::Write) -> Result<(), io::Error> {
    writeln!(buf, "[theme]")?;
    writeln!(buf, "name={}", pal.theme_name)?;
    writeln!(buf, "theme={}", pal.theme)?;
    writeln!(buf, "")?;
    writeln!(buf, "[palette]")?;
    writeln!(buf, "name={}", pal.name)?;
    writeln!(buf, "docs={}", pal.doc.replace('\n', "\\\n"))?;
    writeln!(buf, "generator={}", pal.generator)?;
    writeln!(buf, "")?;
    writeln!(buf, "[color]")?;
    for c in Colors::array() {
        writeln!(
            buf,
            "{}={}, {}",
            *c, pal.color[*c as usize][0], pal.color[*c as usize][3]
        )?;
    }
    writeln!(buf, "")?;
    writeln!(buf, "[reference]")?;
    for (r, i) in pal.aliased.as_ref() {
        writeln!(buf, "{}={}", r, i)?;
    }
    Ok(())
}

/// Load a .pal file as a Palette.
pub fn load_palette(mut r: impl io::Read) -> Result<Palette, io::Error> {
    let mut buf = String::new();
    r.read_to_string(&mut buf)?;

    enum S {
        Start,
        Theme,
        Palette,
        Color,
        Reference,
        Fail(u8),
    }

    let mut pal = Palette::default();
    let mut dark = 63u8;

    let mut state = S::Start;
    'm: for l in buf.lines() {
        let l = l.trim();
        match state {
            S::Start => {
                if l == "[theme]" {
                    state = S::Theme;
                } else if l == "[palette]" {
                    state = S::Palette;
                } else {
                    state = S::Fail(1);
                    break 'm;
                }
            }
            S::Theme => {
                if l == "[palette]" {
                    state = S::Palette;
                } else if l.is_empty() || l.starts_with("#") {
                    // ok
                } else if l.starts_with("name") {
                    if let Some(s) = l.split('=').nth(1) {
                        pal.theme_name = Cow::Owned(s.trim().to_string());
                    }
                } else if l.starts_with("theme") {
                    if let Some(s) = l.split('=').nth(1) {
                        pal.theme = Cow::Owned(s.trim().to_string());
                    }
                } else {
                    state = S::Fail(2);
                    break 'm;
                }
            }
            S::Palette => {
                if l == "[color]" {
                    state = S::Color;
                } else if l.is_empty() || l.starts_with("#") {
                    // ok
                } else if l.starts_with("name") {
                    if let Some(s) = l.split('=').nth(1) {
                        pal.name = Cow::Owned(s.trim().to_string());
                    }
                } else if l.starts_with("docs") {
                    if let Some(s) = l.split('=').nth(1) {
                        pal.doc = Cow::Owned(s.trim().to_string());
                    }
                } else if l.starts_with("generator") {
                    if let Some(s) = l.split('=').nth(1) {
                        pal.generator = Cow::Owned(s.trim().to_string());
                        if s.starts_with("light-dark") {
                            if let Some(s) = l.split(':').nth(1) {
                                dark = s.trim().parse::<u8>().unwrap_or(63);
                            }
                        }
                    }
                } else if l.starts_with("dark") {
                    if let Some(s) = l.split('=').nth(1) {
                        if let Ok(v) = s.trim().parse::<u8>() {
                            dark = v;
                        } else {
                            // skip
                        }
                    }
                } else {
                    state = S::Fail(3);
                    break 'm;
                }
            }
            S::Color => {
                if l == "[reference]" {
                    state = S::Reference;
                } else if l.is_empty() || l.starts_with("#") {
                    // ok
                } else {
                    let mut kv = l.split('=');
                    let cn = if let Some(v) = kv.next() {
                        let Ok(c) = v.trim().parse::<palette::Colors>() else {
                            state = S::Fail(4);
                            break 'm;
                        };
                        c
                    } else {
                        state = S::Fail(5);
                        break 'm;
                    };
                    let (c0, c3) = if let Some(v) = kv.next() {
                        let mut vv = v.split(',');
                        let c0 = if let Some(v) = vv.next() {
                            let Ok(v) = v.trim().parse::<Color>() else {
                                state = S::Fail(6);
                                break 'm;
                            };
                            v
                        } else {
                            state = S::Fail(7);
                            break 'm;
                        };
                        let c3 = if let Some(v) = vv.next() {
                            let Ok(v) = v.trim().parse::<Color>() else {
                                state = S::Fail(8);
                                break 'm;
                            };
                            v
                        } else {
                            state = S::Fail(9);
                            break 'm;
                        };
                        (c0, c3)
                    } else {
                        state = S::Fail(10);
                        break 'm;
                    };

                    if cn == Colors::TextLight || cn == Colors::TextDark {
                        pal.color[cn as usize] =
                            Palette::interpolatec2(c0, c3, Color::default(), Color::default())
                    } else {
                        pal.color[cn as usize] = Palette::interpolatec(c0, c3, dark);
                    }
                }
            }
            S::Reference => {
                let mut kv = l.split('=');
                let rn = if let Some(v) = kv.next() {
                    v
                } else {
                    state = S::Fail(11);
                    break 'm;
                };
                let ci = if let Some(v) = kv.next() {
                    if let Ok(ci) = v.parse::<palette::ColorIdx>() {
                        ci
                    } else {
                        state = S::Fail(12);
                        break 'm;
                    }
                } else {
                    state = S::Fail(13);
                    break 'm;
                };
                pal.add_aliased(rn, ci);
            }
            S::Fail(_) => {
                unreachable!()
            }
        }
    }

    match state {
        S::Fail(n) => Err(io::Error::new(ErrorKind::Other, LoadPaletteErr(n))),
        S::Start => Err(io::Error::new(ErrorKind::Other, LoadPaletteErr(100))),
        S::Theme => Err(io::Error::new(ErrorKind::Other, LoadPaletteErr(101))),
        S::Palette => Err(io::Error::new(ErrorKind::Other, LoadPaletteErr(102))),
        S::Color | S::Reference => Ok(pal),
    }
}

/// Create the Theme based on the given Palette.
pub fn create_palette_theme(pal: Palette) -> Result<SalsaTheme, Palette> {
    match pal.theme.as_ref() {
        "Dark" => Ok(themes::create_dark(pal)),
        "Light" => Ok(themes::create_light(pal)),
        "Shell" => Ok(themes::create_shell(pal)),
        _ => Err(pal),
    }
}

static THEMES: &'static [&'static str] = &[
    "Imperial",
    "Radium",
    "Tundra",
    "Ocean",
    "Monochrome",
    "Black&White",
    "Monekai",
    "Solarized",
    "OxoCarbon",
    "EverForest",
    "Nord",
    "Rust",
    "Material",
    "Tailwind",
    "VSCode",
    "Reds",
    //
    "Imperial Light",
    "EverForest Light",
    "Tailwind Light",
    "Rust Light",
    "SunriseBreeze Light",
    //
    "Imperial Shell",
    "Radium Shell",
    "Tundra Shell",
    "Ocean Shell",
    "Monochrome Shell",
    "Black&White Shell",
    "Monekai Shell",
    "Solarized Shell",
    "OxoCarbon Shell",
    "EverForest Shell",
    "Nord Shell",
    "Rust Shell",
    "Material Shell",
    "Tailwind Shell",
    "VSCode Shell",
    "Reds Shell",
    //
    "Shell",
    "Blackout",
    "Fallback",
];

/// Create one of the predefined themes as a Palette.
///
/// The available themes can be queried by [salsa_themes].
///
/// Known palettes: Imperial, Radium, Tundra, Ocean, Monochrome,
/// Black&White, Monekai, Solarized, OxoCarbon, EverForest,
/// Nord, Rust, Material, Tailwind, VSCode, Reds, Blackout,
/// Shell, Imperial Light, EverForest Light, Tailwind Light,
/// Rust Light.
pub fn create_salsa_palette(name: &str) -> Option<Palette> {
    use crate::palettes::core;
    use crate::palettes::dark;
    use crate::palettes::light;
    match name {
        "Imperial" => Some(dark::IMPERIAL),
        "Radium" => Some(dark::RADIUM),
        "Tundra" => Some(dark::TUNDRA),
        "Ocean" => Some(dark::OCEAN),
        "Monochrome" => Some(dark::MONOCHROME),
        "Black&White" => Some(dark::BLACK_WHITE),
        "Monekai" => Some(dark::MONEKAI),
        "Solarized" => Some(dark::SOLARIZED),
        "OxoCarbon" => Some(dark::OXOCARBON),
        "EverForest" => Some(dark::EVERFOREST),
        "Nord" => Some(dark::NORD),
        "Rust" => Some(dark::RUST),
        "Material" => Some(dark::MATERIAL),
        "Tailwind" => Some(dark::TAILWIND),
        "VSCode" => Some(dark::VSCODE),
        "Reds" => Some(dark::REDS),

        "Imperial Light" => Some(light::IMPERIAL_LIGHT),
        "EverForest Light" => Some(light::EVERFOREST_LIGHT),
        "Tailwind Light" => Some(light::TAILWIND_LIGHT),
        "Rust Light" => Some(light::RUST_LIGHT),
        "SunriseBreeze Light" => Some(light::SUNRISEBREEZE_LIGHT),

        "Imperial Shell" => Some(shell::IMPERIAL_SHELL),
        "Radium Shell" => Some(shell::RADIUM_SHELL),
        "Tundra Shell" => Some(shell::TUNDRA_SHELL),
        "Ocean Shell" => Some(shell::OCEAN_SHELL),
        "Monochrome Shell" => Some(shell::MONOCHROME_SHELL),
        "Black&White Shell" => Some(shell::BLACK_WHITE_SHELL),
        "Monekai Shell" => Some(shell::MONEKAI_SHELL),
        "Solarized Shell" => Some(shell::SOLARIZED_SHELL),
        "OxoCarbon Shell" => Some(shell::OXOCARBON_SHELL),
        "EverForest Shell" => Some(shell::EVERFOREST_SHELL),
        "Nord Shell" => Some(shell::NORD_SHELL),
        "Rust Shell" => Some(shell::RUST_SHELL),
        "Material Shell" => Some(shell::MATERIAL_SHELL),
        "Tailwind Shell" => Some(shell::TAILWIND_SHELL),
        "VSCode Shell" => Some(shell::VSCODE_SHELL),
        "Reds Shell" => Some(shell::REDS_SHELL),

        "Shell" => Some(core::SHELL),
        "Blackout" => Some(core::BLACKOUT),
        "Fallback" => Some(core::FALLBACK),
        _ => None,
    }
}

/// All predefined rat-salsa themes.
pub fn salsa_themes() -> Vec<&'static str> {
    let mut r = Vec::new();
    for v in THEMES {
        r.push(*v);
    }
    r
}

#[deprecated(since = "4.0.4", note = "use create_salsa_theme() instead")]
pub fn create_theme(theme_name: &str) -> SalsaTheme {
    create_salsa_theme(theme_name)
}

/// Create one of the predefined themes.
///
/// The available themes can be queried by [salsa_themes].
///
/// Known themes: Imperial Dark, Radium Dark, Tundra Dark,
/// Ocean Dark, Monochrome Dark, Black&White Dark, Monekai Dark,
/// Solarized Dark, OxoCarbon Dark, EverForest Dark, Nord Dark,
/// Rust Dark, Material Dark, Tailwind Dark, VSCode Dark,
/// Imperial Light, EverForest Light, Tailwind Light, Rust Light,
/// Imperial Shell, Radium Shell, Tundra Shell, Ocean Shell,
/// Monochrome Shell, Black&White Shell, Monekai Shell,
/// Solarized Shell, OxoCarbon Shell, EverForest Shell, Nord Shell,
/// Rust Shell, Material Shell, Tailwind Shell, VSCode Shell,
/// Shell, Blackout and Fallback.
pub fn create_salsa_theme(theme_name: &str) -> SalsaTheme {
    if let Some(pal) = create_salsa_palette(theme_name) {
        match pal.theme.as_ref() {
            "Dark" => themes::create_dark(pal),
            "Light" => themes::create_light(pal),
            "Shell" => themes::create_shell(pal),
            "Fallback" => themes::create_fallback(pal),
            _ => themes::create_shell(palettes::core::SHELL),
        }
    } else {
        themes::create_shell(palettes::core::SHELL)
    }
}
