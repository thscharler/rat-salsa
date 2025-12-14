//!
//! [SalsaTheme] provides a styling system for ratatui apps.
//!
//! It has a flat mapping from `style-name` to either a ratatui `Style`
//! or to one of the composite styles used by [rat-widget](rat_widget).
//!
//! And it contains the underlying color [Palette].
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
//! # use rat_theme4::palettes::core::BLACKOUT;
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
//! ```rust
//! # use std::fs::File;
//! # use rat_theme4::{load_palette, create_palette_theme};
//! # use ratatui::style::Style;
//! # use rat_theme4::StyleName;
//!
//! let mut f = File::open("dark_palettes/base16.pal").expect("pal-file");
//! let pal = load_palette(f).expect("valid_pal-file");
//! let theme = match create_palette_theme(pal) {
//!     Ok(r) => r,
//!     Err(p) => panic!("unsupported theme {:?}", p.theme),
//! };
//!
//! let s: Style = theme.style(Style::INPUT);
//!
//! ```
//!

use crate::palette::Palette;
use crate::palettes::shell;
use crate::theme::SalsaTheme;
use ratatui::style::{Color, Style};
use std::sync::atomic::{AtomicBool, Ordering};

mod error;
mod pal_io;
pub mod palette;
pub mod theme;

pub use error::*;
pub use pal_io::*;

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
/// # use rat_theme4::palettes::core::BLACKOUT;
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
/// # use rat_theme4::palettes::core::BLACKOUT;
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
    #[cfg(feature = "rat-widget")]
    pub const BUTTON: &'static str = "button";
    #[cfg(feature = "rat-widget")]
    pub const CALENDAR: &'static str = "calendar";
    #[cfg(feature = "rat-widget")]
    pub const CHECKBOX: &'static str = "checkbox";
    #[cfg(feature = "rat-widget")]
    pub const CHOICE: &'static str = "choice";
    #[cfg(feature = "rat-widget")]
    pub const CLIPPER: &'static str = "clipper";
    #[cfg(feature = "color-input")]
    pub const COLOR_INPUT: &'static str = "color-input";
    #[cfg(feature = "rat-widget")]
    pub const COMBOBOX: &'static str = "combobox";
    #[cfg(feature = "rat-widget")]
    pub const DIALOG_FRAME: &'static str = "dialog-frame";
    #[cfg(feature = "rat-widget")]
    pub const FILE_DIALOG: &'static str = "file-dialog";
    #[cfg(feature = "rat-widget")]
    pub const FORM: &'static str = "form";
    #[cfg(feature = "rat-widget")]
    pub const LINE_NR: &'static str = "line-nr";
    #[cfg(feature = "rat-widget")]
    pub const LIST: &'static str = "list";
    #[cfg(feature = "rat-widget")]
    pub const MENU: &'static str = "menu";
    #[cfg(feature = "rat-widget")]
    pub const MONTH: &'static str = "month";
    #[cfg(feature = "rat-widget")]
    pub const MSG_DIALOG: &'static str = "msg-dialog";
    #[cfg(feature = "rat-widget")]
    pub const PARAGRAPH: &'static str = "paragraph";
    #[cfg(feature = "rat-widget")]
    pub const RADIO: &'static str = "radio";
    #[cfg(feature = "rat-widget")]
    pub const SCROLL: &'static str = "scroll";
    #[cfg(feature = "rat-widget")]
    pub const SCROLL_DIALOG: &'static str = "scroll.dialog";
    #[cfg(feature = "rat-widget")]
    pub const SCROLL_POPUP: &'static str = "scroll.popup";
    #[cfg(feature = "rat-widget")]
    pub const SHADOW: &'static str = "shadow";
    #[cfg(feature = "rat-widget")]
    pub const SLIDER: &'static str = "slider";
    #[cfg(feature = "rat-widget")]
    pub const SPLIT: &'static str = "split";
    #[cfg(feature = "rat-widget")]
    pub const STATUSLINE: &'static str = "statusline";
    #[cfg(feature = "rat-widget")]
    pub const TABBED: &'static str = "tabbed";
    #[cfg(feature = "rat-widget")]
    pub const TABLE: &'static str = "table";
    #[cfg(feature = "rat-widget")]
    pub const TEXT: &'static str = "text";
    #[cfg(feature = "rat-widget")]
    pub const TEXTAREA: &'static str = "textarea";
    #[cfg(feature = "rat-widget")]
    pub const TEXTVIEW: &'static str = "textview";
    #[cfg(feature = "rat-widget")]
    pub const TOOLBAR: &'static str = "toolbar";
    #[cfg(feature = "rat-widget")]
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
/// # use rat_theme4::palettes::core::BLACKOUT;
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

/// Create the Theme based on the given Palette.
#[allow(clippy::result_large_err)]
pub fn create_palette_theme(pal: Palette) -> Result<SalsaTheme, Palette> {
    match pal.theme.as_ref() {
        "Dark" => Ok(themes::create_dark(pal)),
        "Light" => Ok(themes::create_light(pal)),
        "Shell" => Ok(themes::create_shell(pal)),
        _ => Err(pal),
    }
}

static THEMES: &[&str] = &[
    "Imperial",
    "Black&White",
    "EverForest",
    "Embark",
    "FalconDark",
    "Gatekeeper",
    "Material",
    "Monekai",
    "Monochrome",
    "Nord",
    "Ocean",
    "OxoCarbon",
    "Radium",
    "Reds",
    "Rust",
    "Solarized",
    "Tailwind",
    "Tundra",
    "VSCode",
    //
    "Imperial Light",
    "Blossom Light",
    "EverForest Light",
    "Gatekeeper Light",
    "Embark Light",
    "Rust Light",
    "SunriseBreeze Light",
    "Tailwind Light",
    //
    "Imperial Shell",
    "Black&White Shell",
    "EverForest Shell",
    "Embark Shell",
    "Gatekeeper Shell",
    "Material Shell",
    "Monekai Shell",
    "Monochrome Shell",
    "Nord Shell",
    "Ocean Shell",
    "OxoCarbon Shell",
    "Radium Shell",
    "Reds Shell",
    "Rust Shell",
    "Solarized Shell",
    "Tailwind Shell",
    "Tundra Shell",
    "VSCode Shell",
    //
    "Shell",
    "Blackout",
    "Fallback",
];

/// All predefined rat-salsa themes.
#[deprecated(
    since = "4.1.0",
    note = "there is no separation between themes and palettes any more. use salsa_themes()"
)]
pub fn salsa_palettes() -> Vec<&'static str> {
    let mut r = Vec::new();
    for v in THEMES {
        r.push(*v);
    }
    r
}

/// Create one of the predefined themes as a Palette.
///
/// The available themes can be queried by [salsa_themes].
///
/// Known palettes: Imperial, Radium, Tundra, Ocean, Monochrome,
/// Black&White, Monekai, Solarized, OxoCarbon, EverForest,
/// Nord, Rust, Material, Tailwind, VSCode, Reds, Blackout,
/// Shell, Imperial Light, EverForest Light, Tailwind Light,
/// Rust Light.
#[deprecated(since = "4.1.0", note = "use create_salsa_palette() instead")]
pub fn create_palette(name: &str) -> Option<Palette> {
    create_salsa_palette(name)
}

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
        "Black&White" => Some(dark::BLACK_WHITE),
        "EverForest" => Some(dark::EVERFOREST),
        "FalconDark" => Some(dark::FALCON_DARK),
        "Gatekeeper" => Some(dark::GATEKEEPER),
        "Embark" => Some(dark::EMBARK),
        "Material" => Some(dark::MATERIAL),
        "Monekai" => Some(dark::MONEKAI),
        "Monochrome" => Some(dark::MONOCHROME),
        "Nord" => Some(dark::NORD),
        "Ocean" => Some(dark::OCEAN),
        "OxoCarbon" => Some(dark::OXOCARBON),
        "Radium" => Some(dark::RADIUM),
        "Reds" => Some(dark::REDS),
        "Rust" => Some(dark::RUST),
        "Solarized" => Some(dark::SOLARIZED),
        "Tailwind" => Some(dark::TAILWIND),
        "Tundra" => Some(dark::TUNDRA),
        "VSCode" => Some(dark::VSCODE),

        "Imperial Light" => Some(light::IMPERIAL_LIGHT),
        "Blossom Light" => Some(light::BLOSSOM_LIGHT),
        "Embark Light" => Some(light::EMBARK_LIGHT),
        "EverForest Light" => Some(light::EVERFOREST_LIGHT),
        "Gatekeeper Light" => Some(light::GATEKEEPER_LIGHT),
        "Rust Light" => Some(light::RUST_LIGHT),
        "SunriseBreeze Light" => Some(light::SUNRISEBREEZE_LIGHT),
        "Tailwind Light" => Some(light::TAILWIND_LIGHT),

        "Imperial Shell" => Some(shell::IMPERIAL_SHELL),
        "Black&White Shell" => Some(shell::BLACK_WHITE_SHELL),
        "Embark Shell" => Some(shell::EMBARK_SHELL),
        "EverForest Shell" => Some(shell::EVERFOREST_SHELL),
        "Gatekeeper Shell" => Some(shell::GATEKEEPER_SHELL),
        "Material Shell" => Some(shell::MATERIAL_SHELL),
        "Monekai Shell" => Some(shell::MONEKAI_SHELL),
        "Monochrome Shell" => Some(shell::MONOCHROME_SHELL),
        "Nord Shell" => Some(shell::NORD_SHELL),
        "Ocean Shell" => Some(shell::OCEAN_SHELL),
        "OxoCarbon Shell" => Some(shell::OXOCARBON_SHELL),
        "Radium Shell" => Some(shell::RADIUM_SHELL),
        "Reds Shell" => Some(shell::REDS_SHELL),
        "Rust Shell" => Some(shell::RUST_SHELL),
        "Solarized Shell" => Some(shell::SOLARIZED_SHELL),
        "Tailwind Shell" => Some(shell::TAILWIND_SHELL),
        "Tundra Shell" => Some(shell::TUNDRA_SHELL),
        "VSCode Shell" => Some(shell::VSCODE_SHELL),

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

#[deprecated(since = "4.1.0", note = "use create_salsa_theme() instead")]
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
