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
//! # use rat_theme5::{Theme, StyleName, WidgetStyle, };
//! # use rat_widget::checkbox::{Checkbox, CheckboxState, CheckboxStyle};
//! # let theme = Theme::default();
//! # let area = Rect::default();
//! # let mut buf = Buffer::default();
//! # let buf = &mut buf;
//! # let mut state = CheckboxState::default();
//!
//! Checkbox::new()
//!     .styles(theme.style(WidgetStyle::CHECKBOX))
//!     .render(area, buf, &mut state);
//! ```

use log::debug;
use ratatui::style::{Color, Style};
use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

mod core_theme;
mod dark_theme;
mod fallback_theme;
mod palette;
mod shell_theme;
mod theme;

pub mod dark_palettes;
pub mod light_palettes;
pub mod core_palettes {
    pub use crate::core_theme::SHELL;
}

pub use crate::fallback_theme::fallback_theme;
pub use core_theme::core_theme;
pub use dark_theme::dark_theme;
pub use palette::{ColorIdx, Colors, Palette};
pub use shell_theme::shell_theme;
pub use theme::{Category, SalsaTheme};

/// Anchor struct for the names of composite styles used
/// by rat-widget's.
///
/// Use as
/// ```rust
/// # use ratatui::style::Style;
/// # use rat_theme5::{Theme, StyleName, WidgetStyle};
/// # use rat_theme5::palettes::BLACKOUT;
/// # use rat_widget::checkbox::CheckboxStyle;
/// # let theme = Theme::default();
///
/// let s: CheckboxStyle = theme.style(WidgetStyle::CHECKBOX);
/// ```
/// or more likely
/// ```rust
/// # use ratatui::buffer::Buffer;
/// # use ratatui::layout::Rect;
/// # use ratatui::style::Style;
/// # use ratatui::widgets::StatefulWidget;
/// # use rat_theme5::{Theme, StyleName, WidgetStyle, };
/// # use rat_theme5::palettes::BLACKOUT;
/// # use rat_widget::checkbox::{Checkbox, CheckboxState, CheckboxStyle};
/// # let theme = Theme::default();
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
/// # use rat_theme5::{Theme, StyleName, };
/// # use rat_theme5::palettes::BLACKOUT;
/// # let theme = Theme::default();
///
/// let s: Style = theme.style(Style::INPUT);
/// ```
pub trait StyleName {
    const LABEL_FG: &'static str = "label-fg";
    const INPUT: &'static str = "input";
    const FOCUS: &'static str = "focus";
    const SELECT: &'static str = "select";
    const DISABLED: &'static str = "disabled";
    const INVALID: &'static str = "invalid";
    const HOVER: &'static str = "hover";
    const TITLE: &'static str = "title";
    const HEADER: &'static str = "header";
    const FOOTER: &'static str = "footer";
    const SHADOWS: &'static str = "shadows";
    const WEEK_HEADER_FG: &'static str = "week-header-fg";
    const MONTH_HEADER_FG: &'static str = "month-header-fg";
    const TEXT_FOCUS: &'static str = "text-focus";
    const TEXT_SELECT: &'static str = "text-select";
    const KEY_BINDING: &'static str = "key-binding";
    const BUTTON_BASE: &'static str = "button-base";
    const MENU_BASE: &'static str = "menu-base";
    const STATUS_BASE: &'static str = "status-base";

    const CONTAINER_BASE: &'static str = "container-base";
    const CONTAINER_BORDER_FG: &'static str = "container-border-fg";
    const CONTAINER_ARROW_FG: &'static str = "container-arrows-fg";

    const POPUP_BASE: &'static str = "popup-base";
    const POPUP_BORDER_FG: &'static str = "popup-border-fg";
    const POPUP_ARROW_FG: &'static str = "popup-arrow-fg";

    const DIALOG_BASE: &'static str = "dialog-base";
    const DIALOG_BORDER_FG: &'static str = "dialog-border-fg";
    const DIALOG_ARROW_FG: &'static str = "dialog-arrow-fg";
}
impl StyleName for Style {}

pub trait RatWidgetColor {
    const LABEL_FG: &'static str = "label";
    const INPUT: &'static str = "input";
    const FOCUS: &'static str = "focus";
    const SELECT: &'static str = "select";
    const DISABLED: &'static str = "disabled";
    const INVALID: &'static str = "invalid";
    const HOVER: &'static str = "hover";
    const TITLE_FG: &'static str = "title-fg";
    const TITLE: &'static str = "title";
    const HEADER_FG: &'static str = "header-fg";
    const HEADER: &'static str = "header";
    const FOOTER_FG: &'static str = "footer-fg";
    const FOOTER: &'static str = "footer";
    const SHADOWS: &'static str = "shadows";
    const WEEK_HEADER_FG: &'static str = "week-header-fg";
    const MONTH_HEADER_FG: &'static str = "month-header-fg";
    const TEXT_FOCUS: &'static str = "text-focus";
    const TEXT_SELECT: &'static str = "text-select";
    const BUTTON_BASE: &'static str = "button-base";
    const MENU_BASE: &'static str = "menu-base";
    const KEY_BINDING: &'static str = "key-binding";
    const STATUS_BASE: &'static str = "status-base";
    const CONTAINER_BASE: &'static str = "container-base";
    const CONTAINER_BORDER_FG: &'static str = "container-border";
    const CONTAINER_ARROW_FG: &'static str = "container-arrow";
    const POPUP_BASE: &'static str = "popup-base";
    const POPUP_BORDER_FG: &'static str = "popup-border";
    const POPUP_ARROW_FG: &'static str = "popup-arrow";
    const DIALOG_BASE: &'static str = "dialog-base";
    const DIALOG_BORDER_FG: &'static str = "dialog-border";
    const DIALOG_ARROW_FG: &'static str = "dialog-arrow";
}
impl RatWidgetColor for Color {}

pub fn rat_widget_color_names() -> &'static [&'static str] {
    &[
        Color::LABEL_FG,
        Color::INPUT,
        Color::FOCUS,
        Color::SELECT,
        Color::DISABLED,
        Color::INVALID,
        Color::HOVER,
        Color::TITLE_FG,
        Color::TITLE,
        Color::HEADER_FG,
        Color::HEADER,
        Color::FOOTER_FG,
        Color::FOOTER,
        Color::SHADOWS,
        Color::WEEK_HEADER_FG,
        Color::MONTH_HEADER_FG,
        Color::TEXT_FOCUS,
        Color::TEXT_SELECT,
        Color::BUTTON_BASE,
        Color::MENU_BASE,
        Color::KEY_BINDING,
        Color::STATUS_BASE,
        Color::CONTAINER_BASE,
        Color::CONTAINER_BORDER_FG,
        Color::CONTAINER_ARROW_FG,
        Color::POPUP_BASE,
        Color::POPUP_BORDER_FG,
        Color::POPUP_ARROW_FG,
        Color::DIALOG_BASE,
        Color::DIALOG_BORDER_FG,
        Color::DIALOG_ARROW_FG,
    ]
}

static LOG_DEFINES: AtomicBool = AtomicBool::new(false);

/// Log style definition.
pub fn log_style_define(log: bool) {
    LOG_DEFINES.store(log, Ordering::Release);
}

fn is_log_style_define() -> bool {
    LOG_DEFINES.load(Ordering::Acquire)
}

const PALETTE_DEF: &str = include_str!("themes.ini");

#[derive(Debug)]
struct Def {
    palette: Vec<&'static str>,
    theme: Vec<&'static str>,
    theme_init: HashMap<&'static str, (&'static str, &'static str)>,
}

static THEMES: OnceLock<Def> = OnceLock::new();

fn init_themes() -> Def {
    let mut palette = Vec::new();
    let mut theme = Vec::new();
    let mut theme_init = HashMap::new();

    for l in PALETTE_DEF.lines() {
        if !l.contains('=') {
            continue;
        }

        let mut it = l.split(['=', ',']);
        let Some(name) = it.next() else {
            continue;
        };
        let Some(cat) = it.next() else {
            continue;
        };
        let Some(pal) = it.next() else {
            continue;
        };
        let name = name.trim();
        let cat = cat.trim();
        let pal = pal.trim();

        if pal != "None" {
            if !palette.contains(&pal) {
                palette.push(pal);
            }
        }
        if name != "Blackout" && name != "Fallback" {
            if !theme.contains(&name) {
                theme.push(name);
            }
        }
        theme_init.insert(name, (cat, pal));
    }

    let d = Def {
        palette,
        theme,
        theme_init,
    };
    d
}

/// All defined color palettes.
pub fn salsa_palettes() -> Vec<&'static str> {
    let themes = THEMES.get_or_init(init_themes);
    themes.palette.clone()
}

/// Create one of the defined palettes.
///
/// The available palettes can be queried by [salsa_palettes].
pub fn create_palette(name: &str) -> Option<Palette> {
    use crate::core_palettes as core;
    use crate::dark_palettes as dark;
    use crate::light_palettes as light;
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
        "VSCode" => Some(dark::VSCODE),
        "Reds" => Some(dark::REDS),
        "Blackout" => Some(dark::BLACKOUT),
        "Shell" => Some(core::SHELL),
        "EverForest Light" => Some(light::EVERFOREST),
        _ => None,
    }
}

/// All defined rat-salsa themes.
pub fn salsa_themes() -> Vec<&'static str> {
    let themes = THEMES.get_or_init(init_themes);
    themes.theme.clone()
}

/// Create one of the defined themes.
///
/// The available themes can be queried by [salsa_themes].
pub fn create_theme(theme: &str) -> SalsaTheme {
    let themes = THEMES.get_or_init(init_themes);
    let Some(def) = themes.theme_init.get(&theme) else {
        if cfg!(debug_assertions) {
            panic!("no theme {:?}", theme);
        } else {
            return core_theme(theme);
        }
    };
    match def {
        ("dark", p) => {
            let Some(pal) = create_palette(*p) else {
                if cfg!(debug_assertions) {
                    panic!("no palette {:?}", *p);
                } else {
                    return core_theme(theme);
                }
            };
            dark_theme(theme, pal)
        }
        ("light", p) => {
            let Some(pal) = create_palette(*p) else {
                if cfg!(debug_assertions) {
                    panic!("no palette {:?}", *p);
                } else {
                    return core_theme(theme);
                }
            };
            // currently no difference, just a different
            // set of color palettes
            dark_theme(theme, pal)
        }
        ("shell", p) => {
            let Some(pal) = create_palette(*p) else {
                if cfg!(debug_assertions) {
                    panic!("no palette {:?}", *p);
                } else {
                    return core_theme(theme);
                }
            };
            shell_theme(theme, pal)
        }
        ("core", _) => core_theme(theme),
        ("blackout", _) => dark_theme(theme, dark_palettes::BLACKOUT),
        ("fallback", _) => fallback_theme(theme, dark_palettes::REDS),
        _ => {
            if cfg!(debug_assertions) {
                panic!("no theme {:?}", theme);
            } else {
                core_theme(theme)
            }
        }
    }
}
