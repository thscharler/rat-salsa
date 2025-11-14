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
//! # use rat_theme4::{create_empty, SalsaTheme, StyleName, WidgetStyle, };
//! # use rat_theme4::palettes::BLACKOUT;
//! # use rat_widget::checkbox::{Checkbox, CheckboxState, CheckboxStyle};
//! # let theme = create_empty("", BLACKOUT);
//! # let area = Rect::default();
//! # let mut buf = Buffer::default();
//! # let buf = &mut buf;
//! # let mut state = CheckboxState::default();
//!
//! Checkbox::new()
//!     .styles(theme.style(WidgetStyle::CHECKBOX))
//!     .render(area, buf, &mut state);
//! ```

use ratatui::style::Style;
use std::sync::atomic::{AtomicBool, Ordering};

mod dark_theme;
// mod fallback_theme;
// mod light_theme;
mod palette;
pub mod palettes;
mod theme;
// mod shell_theme;

pub use dark_theme::dark_theme;
// pub use fallback_theme::fallback_theme;
// pub use light_theme::light_theme;
pub use palette::{ColorIdx, Colors, ColorsExt, Palette};
pub use theme::{Category, Theme};
// pub use shell_theme::shell_theme;

/// Anchor struct for the names of composite styles used
/// by rat-widget's.
///
/// Use as
/// ```rust
/// # use ratatui::style::Style;
/// # use rat_theme4::{create_empty, SalsaTheme, StyleName, WidgetStyle};
/// # use rat_theme4::palettes::BLACKOUT;
/// # use rat_widget::checkbox::CheckboxStyle;
/// # let theme = create_empty("", BLACKOUT);
///
/// let s: CheckboxStyle = theme.style(WidgetStyle::CHECKBOX);
/// ```
/// or more likely
/// ```rust
/// # use ratatui::buffer::Buffer;
/// # use ratatui::layout::Rect;
/// # use ratatui::style::Style;
/// # use ratatui::widgets::StatefulWidget;
/// # use rat_theme4::{create_empty, SalsaTheme, StyleName, WidgetStyle, };
/// # use rat_theme4::palettes::BLACKOUT;
/// # use rat_widget::checkbox::{Checkbox, CheckboxState, CheckboxStyle};
/// # let theme = create_empty("", BLACKOUT);
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
/// some standard names used by rat-theme.
///
/// Use as
/// ```rust
/// # use ratatui::style::Style;
/// # use rat_theme4::{create_empty, SalsaTheme, StyleName, };
/// # use rat_theme4::palettes::BLACKOUT;
/// # let theme = create_empty("", BLACKOUT);
///
/// let s: Style = theme.style(Style::INPUT);
/// ```
pub trait StyleName {
    const LABEL: &'static str = "label";
    const INPUT: &'static str = "input";
    const FOCUS: &'static str = "focus";
    const SELECT: &'static str = "select";
    const DISABLED: &'static str = "disabled";
    const INVALID: &'static str = "invalid";
    const TITLE: &'static str = "title";
    const HEADER: &'static str = "header";
    const FOOTER: &'static str = "footer";
    const SHADOW: &'static str = "shadow";
    const TEXT_FOCUS: &'static str = "text-focus";
    const TEXT_SELECT: &'static str = "text-select";
    const KEY_BINDING: &'static str = "key-binding";
    const BUTTON_BASE: &'static str = "button-base";
    const MENU_BASE: &'static str = "menu-base";
    const STATUS_BASE: &'static str = "status-base";

    const CONTAINER_BASE: &'static str = "container-base";
    const CONTAINER_BORDER: &'static str = "container-border";
    const CONTAINER_ARROW: &'static str = "container-arrows";

    const POPUP_BASE: &'static str = "popup-base";
    const POPUP_BORDER: &'static str = "popup-border";
    const POPUP_ARROW: &'static str = "popup-arrow";

    const DIALOG_BASE: &'static str = "dialog-base";
    const DIALOG_BORDER: &'static str = "dialog-border";
    const DIALOG_ARROW: &'static str = "dialog-arrow";
}
impl StyleName for Style {}

static LOG_DEFINES: AtomicBool = AtomicBool::new(false);

/// Log style definition.
pub fn log_style_define(log: bool) {
    LOG_DEFINES.store(log, Ordering::Release);
}

fn is_log_style_define() -> bool {
    LOG_DEFINES.load(Ordering::Acquire)
}

const PALETTES: &[&str] = &[
    "Imperial",
    // "Radium",
    // "Tundra",
    // "Ocean",
    // "Monochrome",
    // "Black & White",
    // "Base16",
    // "Base16 Relax",
    // "Monekai",
    // "Solarized",
    // "OxoCarbon",
    // "EverForest",
    // "Nord",
    // "Rust",
    // "Red",
    // "Blackout",
    // "Shell",
    // "VSCode",
];

/// All currently existing color palettes.
pub fn salsa_palettes() -> Vec<&'static str> {
    Vec::from(PALETTES)
}

/// Get a Palette by name.
pub fn create_palette(name: &str) -> Option<Palette> {
    use crate::palettes::*;
    match name {
        "Imperial" => Some(IMPERIAL),
        // "Radium" => Some(RADIUM),
        // "Tundra" => Some(TUNDRA),
        // "Ocean" => Some(OCEAN),
        // "Monochrome" => Some(MONOCHROME),
        // "Black & White" => Some(BLACKWHITE),
        // "Base16" => Some(BASE16),
        // "Base16 Relax" => Some(BASE16_RELAX),
        // "Monekai" => Some(MONEKAI),
        // "Solarized" => Some(SOLARIZED),
        // "OxoCarbon" => Some(OXOCARBON),
        // "EverForest" => Some(EVERFOREST),
        // "Nord" => Some(NORD),
        // "Rust" => Some(RUST),
        // "Red" => Some(RED),
        // "Blackout" => Some(BLACKOUT),
        // "Shell" => Some(SHELL),
        // "VSCode" => Some(VSCODE_DARK),
        _ => None,
    }
}

const THEMES: &[&str] = &[
    "Imperial Dark",
    // "Radium Dark",
    // "Tundra Dark",
    // "Ocean Dark",
    // "Monochrome Dark",
    // "Black & White Dark",
    // "Base16 Dark",
    // "Base16 Relax Dark",
    // "Monekai Dark",
    // "Solarized Dark",
    // "OxoCarbon Dark",
    // "EverForest Dark",
    // "Nord Dark",
    // "Rust Dark",
    // "Red Dark",
    // "Shell Dark",
    // "VSCode Dark",
    // //
    // "Imperial Shell",
    // "Radium Shell",
    // "Tundra Shell",
    // "Ocean Shell",
    // "Monochrome Shell",
    // "Black & White Shell",
    // "Base16 Shell",
    // "Base16 Relax Shell",
    // "Monekai Shell",
    // "Solarized Shell",
    // "OxoCarbon Shell",
    // "EverForest Shell",
    // "Nord Shell",
    // "Rust Shell",
    // "Red Shell",
    // "Shell Shell",
    // "VSCode Shell",
    // //
    // "Blackout Dark",
    // "Blackout Shell",
    // "Fallback",
];

/// Get all Salsa themes.
pub fn salsa_themes() -> Vec<&'static str> {
    Vec::from(THEMES)
}

/// Create a theme.
pub fn create_theme(theme: &str) -> Option<Theme> {
    use crate::palettes::*;
    let theme = match theme {
        "Imperial Dark" => dark_theme(theme, IMPERIAL),
        // "Radium Dark" => dark_theme(theme, RADIUM),
        // "Tundra Dark" => dark_theme(theme, TUNDRA),
        // "Ocean Dark" => dark_theme(theme, OCEAN),
        // "Monochrome Dark" => dark_theme(theme, MONOCHROME),
        // "Black & White Dark" => dark_theme(theme, BLACKWHITE),
        // "Base16 Dark" => dark_theme(theme, BASE16),
        // "Base16 Relax Dark" => dark_theme(theme, BASE16_RELAX),
        // "Monekai Dark" => dark_theme(theme, MONEKAI),
        // "Solarized Dark" => dark_theme(theme, SOLARIZED),
        // "OxoCarbon Dark" => dark_theme(theme, OXOCARBON),
        // "EverForest Dark" => dark_theme(theme, EVERFOREST),
        // "Nord Dark" => dark_theme(theme, NORD),
        // "Rust Dark" => dark_theme(theme, RUST),
        // "Red Dark" => dark_theme(theme, RED),
        // "Shell Dark" => dark_theme(theme, SHELL),
        // "VSCode Dark" => dark_theme(theme, VSCODE_DARK),
        //
        // "Imperial Shell" => shell_theme(theme, IMPERIAL),
        // "Radium Shell" => shell_theme(theme, RADIUM),
        // "Tundra Shell" => shell_theme(theme, TUNDRA),
        // "Ocean Shell" => shell_theme(theme, OCEAN),
        // "Monochrome Shell" => shell_theme(theme, MONOCHROME),
        // "Black & White Shell" => shell_theme(theme, BLACKWHITE),
        // "Base16 Shell" => shell_theme(theme, BASE16),
        // "Base16 Relax Shell" => shell_theme(theme, BASE16_RELAX),
        // "Monekai Shell" => shell_theme(theme, MONEKAI),
        // "Solarized Shell" => shell_theme(theme, SOLARIZED),
        // "OxoCarbon Shell" => shell_theme(theme, OXOCARBON),
        // "EverForest Shell" => shell_theme(theme, EVERFOREST),
        // "Nord Shell" => shell_theme(theme, NORD),
        // "Rust Shell" => shell_theme(theme, RUST),
        // "Red Shell" => shell_theme(theme, RED),
        // "Shell Shell" => shell_theme(theme, SHELL),
        // "VSCode Shell" => shell_theme(theme, VSCODE_DARK),
        //
        // "Blackout Dark" => dark_theme(theme, BLACKOUT),
        // "Blackout Shell" => shell_theme(theme, BLACKOUT),
        // "Fallback" => fallback_theme(theme, RED),
        _ => return None,
    };

    Some(theme)
}
