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

use log::info;
use ratatui::style::Style;
use std::any::{Any, type_name, type_name_of_val};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::atomic::{AtomicBool, Ordering};

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
    pub const COMBOBOX: &'static str = "combobox";
    pub const CONTAINER: &'static str = "container";
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
    const TEXT_FOCUS: &'static str = "text-focus";
    const TEXT_SELECT: &'static str = "text-select";
    const BUTTON_BASE: &'static str = "button-base";

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

// --  Entry ---
trait StyleValue: Any + Debug {}
impl<T> StyleValue for T where T: Any + Debug {}

#[allow(clippy::type_complexity)]
enum Entry {
    Style(Style),
    Closure(Box<dyn Fn(&SalsaTheme) -> Box<dyn StyleValue> + 'static>),
}

impl Debug for Entry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Entry::Style(s) => {
                write!(f, "{:?}", s)
            }
            Entry::Closure(fun) => {
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

static LOG_DEFINES: AtomicBool = AtomicBool::new(false);

/// Log style definition.
pub fn log_style_define(log: bool) {
    LOG_DEFINES.store(log, Ordering::Release);
}

fn is_log_style_define() -> bool {
    LOG_DEFINES.load(Ordering::Acquire)
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

    /// Some display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Define a style as a plain [Style].
    pub fn define(&mut self, name: &'static str, style: Style) {
        if is_log_style_define() {
            info!("salsa-style: {:?}->{:?}", name, style);
        }
        self.styles.insert(name, Entry::Style(style));
    }

    /// Define a style a struct that will be cloned for every query.
    pub fn define_clone(&mut self, name: &'static str, sample: impl Clone + Any + Debug + 'static) {
        let boxed = Box::new(move |_th: &SalsaTheme| -> Box<dyn StyleValue> {
            Box::new(sample.clone()) //
        });
        if is_log_style_define() {
            info!("salsa-style: {:?}->{:?}", name, boxed(self));
        }
        self.styles.insert(name, Entry::Closure(boxed));
    }

    /// Define a style as a call to a constructor fn.
    ///
    /// The constructor gets access to all previously initialized styles.
    pub fn define_fn<O: Any + Debug>(
        &mut self,
        name: &'static str,
        create: impl Fn(&SalsaTheme) -> O + 'static,
    ) {
        let boxed = Box::new(move |th: &SalsaTheme| -> Box<dyn StyleValue> {
            Box::new(create(th)) //
        });
        if is_log_style_define() {
            info!("salsa-style: {:?}->{:?}", name, boxed(self));
        }
        self.styles.insert(name, Entry::Closure(boxed));
    }

    /// Define a style as a call to a constructor fn.
    ///
    /// This one takes no arguments, this is nice to set Widget::default
    /// as the style-fn.
    pub fn define_fn0<O: Any + Debug>(
        &mut self,
        name: &'static str,
        create: impl Fn() -> O + 'static,
    ) {
        let boxed = Box::new(move |_th: &SalsaTheme| -> Box<dyn StyleValue> {
            Box::new(create()) //
        });
        if is_log_style_define() {
            info!("salsa-style: {:?}->{:?}", name, boxed(self));
        }
        self.styles.insert(name, Entry::Closure(boxed));
    }

    #[allow(clippy::collapsible_else_if)]
    fn dyn_style(&self, name: &str) -> Option<Box<dyn StyleValue>> {
        if let Some(entry) = self.styles.get(name) {
            match entry {
                Entry::Style(style) => Some(Box::new(*style)),
                Entry::Closure(create) => Some(create(self)),
            }
        } else {
            if cfg!(debug_assertions) {
                panic!("unknown style {:?}", name)
            } else {
                None
            }
        }
    }

    /// Get one of the defined ratatui-Styles.
    ///
    /// This is the same as the single [style] function, it just
    /// fixes the return-type to style. This is useful if the
    /// receiver is declared as `impl Into<Style>`.
    ///
    /// It downcasts the stored value to the required out type.
    /// This may fail.
    ///
    /// * When debug_assertions are enabled it will panic when
    ///   called with an unknown style name, or if the downcast
    ///   to the out type fails.
    /// * Otherwise, it will return the default value of the out type.
    pub fn style_style(&self, name: &str) -> Style
    where
        Self: Sized,
    {
        self.style::<Style>(name)
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
    pub fn style<O: Default + Sized + 'static>(&self, name: &str) -> O
    where
        Self: Sized,
    {
        if cfg!(debug_assertions) {
            let style = match self.dyn_style(name) {
                Some(v) => v,
                None => {
                    panic!("unknown widget {:?}", name)
                }
            };
            let any_style = style as Box<dyn Any>;
            let style = match any_style.downcast::<O>() {
                Ok(v) => v,
                Err(_) => {
                    let style = self.dyn_style(name).expect("style");
                    panic!(
                        "downcast fails for '{}' to {}: {:?}",
                        name,
                        type_name::<O>(),
                        style
                    );
                }
            };
            *style
        } else {
            let Some(style) = self.dyn_style(name) else {
                return O::default();
            };
            let any_style = style as Box<dyn Any>;
            let Ok(style) = any_style.downcast::<O>() else {
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
