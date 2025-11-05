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
use std::any::{Any, type_name, type_name_of_val};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

mod dark_theme;
mod fallback_theme;
mod palette;
pub mod palettes;
mod shell_theme;

pub use dark_theme::dark_theme;
pub use fallback_theme::fallback_theme;
pub use palette::Palette;
pub use shell_theme::shell_theme;

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
    pub const CONTAINER: &'static str = "container";
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
    const TEXT_FOCUS: &'static str = "text-focus";
    const TEXT_SELECT: &'static str = "text-select";

    const CONTAINER_BASE: &'static str = "container-base";
    const CONTAINER_BORDER: &'static str = "container-border";
    const CONTAINER_ARROWS: &'static str = "container-arrows";

    const POPUP_BASE: &'static str = "popup-base";
    const POPUP_BORDER: &'static str = "popup-border";
    const POPUP_ARROW: &'static str = "popup-arrow";

    const DIALOG_BASE: &'static str = "dialog-base";
    const DIALOG_BORDER: &'static str = "dialog-border";
    const DIALOG_ARROW: &'static str = "dialog-arrow";

    const STATUS_BASE: &'static str = "status-base";
}

impl StyleName for Style {}

/// This macro takes a function that returns some type and
/// converts the result to `Box<dyn Any + 'static>`.
///
/// Useful when creating a new theme.
#[macro_export]
macro_rules! style_fn {
    ($fn_name:ident) => {
        |theme: &$crate::SalsaTheme| Box::new($fn_name(theme))
    };
}

#[allow(clippy::type_complexity)]
enum Entry {
    Style(Style),
    Fn(fn(&SalsaTheme) -> Box<dyn Any>),
    FnClosure(Box<dyn Fn(&SalsaTheme) -> Box<dyn Any> + 'static>),
}

impl Debug for Entry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Entry::Style(s) => {
                write!(f, "{:?}", s)
            }
            Entry::Fn(fun) => {
                write!(f, "{:?}", type_name_of_val(fun))
            }
            Entry::FnClosure(fun) => {
                write!(f, "{:?}", type_name_of_val(fun))
            }
        }
    }
}

/// Categorization of themes.
/// Helpful when extending an existing theme.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Category {
    #[default]
    Other,
    /// Dark theme.
    Dark,
    /// Light theme.
    Light,
    /// Shell theme. Themes of this category rely on background colors sparingly
    /// and use any default the terminal itself provides.
    Shell,
}

///
/// SalsaTheme holds any predefined styles for the UI.  
///
/// The foremost usage is as a store of named [Style](ratatui::style::Style)s.
/// It can also hold the structured styles used by rat-widget's.
/// Or really any value that can be produced by a closure.
///
/// It uses a flat naming scheme and doesn't cascade upwards at all.
#[derive(Debug, Default)]
pub struct SalsaTheme {
    pub name: String,
    pub cat: Category,
    pub p: Palette,
    styles: HashMap<&'static str, Entry>,
}

impl SalsaTheme {
    /// Create an empty theme with a given color palette.
    pub fn new(name: impl Into<String>, cat: Category, p: Palette) -> Self {
        Self {
            name: name.into(),
            cat,
            p,
            styles: Default::default(),
        }
    }
}

impl SalsaTheme {
    /// Some display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Define a style as a plain [Style].
    pub fn define(&mut self, n: &'static str, style: Style) {
        self.styles.insert(n, Entry::Style(style));
    }

    /// Define a style a struct that will be cloned for every query.
    pub fn define_clone(&mut self, n: &'static str, sample: impl Clone + 'static) {
        let boxed = Box::new(move |_th: &SalsaTheme| -> Box<dyn Any> {
            Box::new(sample.clone()) //
        });
        self.styles.insert(n, Entry::FnClosure(boxed));
    }

    /// Define a style as a call to a constructor fn.
    pub fn define_fn(&mut self, n: &'static str, cr: fn(&SalsaTheme) -> Box<dyn Any>) {
        self.styles.insert(n, Entry::Fn(cr));
    }

    /// Define a style as a call to a constraint Fn closure.
    pub fn define_closure<O: Any>(
        &mut self,
        n: &'static str,
        cr: impl Fn(&SalsaTheme) -> O + 'static,
    ) {
        let wrapped = Box::new(move |th: &SalsaTheme| -> Box<dyn Any> {
            Box::new(cr(th)) // 
        });
        self.styles.insert(n, Entry::FnClosure(wrapped));
    }

    #[allow(clippy::collapsible_else_if)]
    fn dyn_style(&self, w: &str) -> Option<Box<dyn Any>> {
        if let Some(entry) = self.styles.get(w) {
            match entry {
                Entry::Style(style) => Some(Box::new(*style)),
                Entry::Fn(create) => Some(create(self)),
                Entry::FnClosure(create) => Some(create(self)),
            }
        } else {
            if cfg!(debug_assertions) {
                panic!("unknown style {}", w)
            } else {
                None
            }
        }
    }

    /// Get any of the defined styles.
    ///
    /// It downcasts the stored value to the required out type.
    /// This may fail.
    ///
    /// * When debug_assertions are enabled it will panic when
    ///   called with an unknown style name, or if the downcast
    ///   to the out type fails.
    /// * Otherwise, it will return the default value of the out type.
    pub fn style<O: Default + Sized + 'static>(&self, w: &str) -> O
    where
        Self: Sized,
    {
        if cfg!(debug_assertions) {
            let style = match self.dyn_style(w) {
                Some(v) => v,
                None => {
                    panic!("unknown widget {}", w)
                }
            };
            let style = match style.downcast::<O>() {
                Ok(v) => v,
                Err(e) => {
                    panic!("downcast fails for {} to {}: {:?}", w, type_name::<O>(), e);
                }
            };
            *style
        } else {
            let Some(style) = self.dyn_style(w) else {
                return O::default();
            };
            let Ok(style) = style.downcast::<O>() else {
                return O::default();
            };
            *style
        }
    }
}

const PALETTES: &[&str] = &[
    "Imperial",
    "Radium",
    "Tundra",
    "Ocean",
    "Monochrome",
    "Black & White",
    "Base16",
    "Base16 Relax",
    "Monekai",
    "Solarized",
    "OxoCarbon",
    "Rust",
    "Blackout",
    "Shell",
    "VSCode",
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
        "Radium" => Some(RADIUM),
        "Tundra" => Some(TUNDRA),
        "Ocean" => Some(OCEAN),
        "Monochrome" => Some(MONOCHROME),
        "Black & White" => Some(BLACKWHITE),
        "Base16" => Some(BASE16),
        "Base16 Relax" => Some(BASE16_RELAX),
        "Monekai" => Some(MONEKAI),
        "Solarized" => Some(SOLARIZED),
        "OxoCarbon" => Some(OXOCARBON),
        "Rust" => Some(RUST),
        "Red" => Some(RED),
        "Blackout" => Some(BLACKOUT),
        "Shell" => Some(SHELL),
        "VSCode" => Some(VSCODE_DARK),
        _ => None,
    }
}

const THEMES: &[&str] = &[
    "Imperial Dark",
    "Radium Dark",
    "Tundra Dark",
    "Ocean Dark",
    "Monochrome Dark",
    "Black & White Dark",
    "Base16 Dark",
    "Base16 Relax Dark",
    "Monekai Dark",
    "Solarized Dark",
    "OxoCarbon Dark",
    "Rust Dark",
    "Red Dark",
    "Shell Dark",
    "VSCode Dark",
    //
    "Imperial Shell",
    "Radium Shell",
    "Tundra Shell",
    "Ocean Shell",
    "Monochrome Shell",
    "Black & White Shell",
    "Base16 Shell",
    "Base16 Relax Shell",
    "Monekai Shell",
    "Solarized Shell",
    "OxoCarbon Shell",
    "Rust Shell",
    "Red Shell",
    "Shell Shell",
    "VSCode Shell",
    //
    "Blackout Dark",
    "Blackout Shell",
    "Fallback",
];

/// Get all Salsa themes.
pub fn salsa_themes() -> Vec<&'static str> {
    Vec::from(THEMES)
}

/// Create a theme.
pub fn create_theme(theme: &str) -> Option<SalsaTheme> {
    use crate::palettes::*;
    let theme = match theme {
        "Imperial Dark" => dark_theme(theme, IMPERIAL),
        "Radium Dark" => dark_theme(theme, RADIUM),
        "Tundra Dark" => dark_theme(theme, TUNDRA),
        "Ocean Dark" => dark_theme(theme, OCEAN),
        "Monochrome Dark" => dark_theme(theme, MONOCHROME),
        "Black & White Dark" => dark_theme(theme, BLACKWHITE),
        "Base16 Dark" => dark_theme(theme, BASE16),
        "Base16 Relax Dark" => dark_theme(theme, BASE16_RELAX),
        "Monekai Dark" => dark_theme(theme, MONEKAI),
        "Solarized Dark" => dark_theme(theme, SOLARIZED),
        "OxoCarbon Dark" => dark_theme(theme, OXOCARBON),
        "Rust Dark" => dark_theme(theme, RUST),
        "Red Dark" => dark_theme(theme, RED),
        "Shell Dark" => dark_theme(theme, SHELL),
        "VSCode Dark" => dark_theme(theme, VSCODE_DARK),

        "Imperial Shell" => shell_theme(theme, IMPERIAL),
        "Radium Shell" => shell_theme(theme, RADIUM),
        "Tundra Shell" => shell_theme(theme, TUNDRA),
        "Ocean Shell" => shell_theme(theme, OCEAN),
        "Monochrome Shell" => shell_theme(theme, MONOCHROME),
        "Black & White Shell" => shell_theme(theme, BLACKWHITE),
        "Base16 Shell" => shell_theme(theme, BASE16),
        "Base16 Relax Shell" => shell_theme(theme, BASE16_RELAX),
        "Monekai Shell" => shell_theme(theme, MONEKAI),
        "Solarized Shell" => shell_theme(theme, SOLARIZED),
        "OxoCarbon Shell" => shell_theme(theme, OXOCARBON),
        "Rust Shell" => shell_theme(theme, RUST),
        "Red Shell" => shell_theme(theme, RED),
        "Shell Shell" => shell_theme(theme, SHELL),
        "VSCode Shell" => shell_theme(theme, VSCODE_DARK),

        "Blackout Dark" => dark_theme(theme, BLACKOUT),
        "Blackout Shell" => shell_theme(theme, BLACKOUT),
        "Fallback" => fallback_theme(theme, RED),

        _ => return None,
    };

    Some(theme)
}

/// Create an empty SalsaTheme.
pub fn create_empty(name: &str, p: Palette) -> SalsaTheme {
    SalsaTheme::new(name, Category::Other, p)
}
