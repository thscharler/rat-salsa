use ratatui::style::Style;
use std::any::{Any, type_name, type_name_of_val};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

mod dark_theme;
mod palette;
mod palettes;
mod shell_theme;

pub use dark_theme::dark_theme;
pub use palette::Palette;
pub use palettes::*;
pub use shell_theme::shell_theme;

pub struct WidgetStyle;

impl WidgetStyle {
    pub const BUTTON: &'static str = "button";
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

pub const FG0: usize = 0;
pub const FG1: usize = 1;
pub const FG2: usize = 2;
pub const FG3: usize = 3;
pub const BG1: usize = 4;
pub const BG2: usize = 5;
pub const BG3: usize = 6;
pub const BG4: usize = 7;

#[macro_export]
macro_rules! make_dyn {
    ($fn_name:ident) => {
        |theme: &$crate::SalsaTheme| Box::new($fn_name(theme))
    };
}

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

#[derive(Debug, Default)]
pub struct SalsaTheme {
    pub name: String,
    pub p: Palette,
    styles: HashMap<&'static str, Entry>,
}

impl SalsaTheme {
    pub fn new(name: impl Into<String>, s: Palette) -> Self {
        Self {
            p: s,
            name: name.into(),
            styles: Default::default(),
        }
    }
}

impl SalsaTheme {
    /// Some display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn define(&mut self, n: &'static str, style: Style) {
        self.styles.insert(n, Entry::Style(style));
    }

    pub fn define_fn(&mut self, w: &'static str, cr: fn(&SalsaTheme) -> Box<dyn Any>) {
        self.styles.insert(w, Entry::Fn(cr));
    }

    pub fn define_closure(
        &mut self,
        w: &'static str,
        cr: Box<dyn Fn(&SalsaTheme) -> Box<dyn Any> + 'static>,
    ) {
        self.styles.insert(w, Entry::FnClosure(cr));
    }

    fn dyn_style(&self, w: &str) -> Option<Box<dyn Any>> {
        if let Some(entry) = self.styles.get(w) {
            match entry {
                Entry::Style(style) => Some(Box::new(style.clone())),
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

    /// Get the style for the named widget.
    pub fn style<O: Default + Sized + 'static>(&self, w: &str) -> O
    where
        Self: Sized,
    {
        if cfg!(debug_assertions) {
            let style = self
                .dyn_style(w)
                .expect(format!("unknown widget {}", w).as_str());
            let style = style
                .downcast::<O>()
                .expect(format!("downcast fails for {} to {}", w, type_name::<O>()).as_str());
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
    "VSCode Shell",
];

pub fn salsa_themes() -> Vec<&'static str> {
    Vec::from(THEMES)
}

// Create a theme + palette.
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
        "VSCode Shell" => shell_theme(theme, VSCODE_DARK),

        _ => return None,
    };

    Some(theme)
}

/// Create an empty SalsaTheme.
pub fn create_empty(name: &str, p: Palette) -> SalsaTheme {
    SalsaTheme::new(name, p)
}

// /// Create a style from the given white shade.
// /// n is `0..8`
// fn white(&self, n: usize) -> Style;
//
// /// Create a style from the given black shade.
// /// n is `0..8`
// fn black(&self, n: usize) -> Style;
//
// /// Create a style from the given gray shade.
// /// n is `0..8`
// fn gray(&self, n: usize) -> Style;
//
// /// Create a style from the given red shade.
// /// n is `0..8`
// fn red(&self, n: usize) -> Style;
//
// /// Create a style from the given orange shade.
// /// n is `0..8`
// fn orange(&self, n: usize) -> Style;
//
// /// Create a style from the given yellow shade.
// /// n is `0..8`
// fn yellow(&self, n: usize) -> Style;
//
// /// Create a style from the given limegreen shade.
// /// n is `0..8`
// fn limegreen(&self, n: usize) -> Style;
//
// /// Create a style from the given green shade.
// /// n is `0..8`
// fn green(&self, n: usize) -> Style;
//
// /// Create a style from the given bluegreen shade.
// /// n is `0..8`
// fn bluegreen(&self, n: usize) -> Style;
//
// /// Create a style from the given cyan shade.
// /// n is `0..8`
// fn cyan(&self, n: usize) -> Style;
//
// /// Create a style from the given blue shade.
// /// n is `0..8`
// fn blue(&self, n: usize) -> Style;
//
// /// Create a style from the given deepblue shade.
// /// n is `0..8`
// fn deepblue(&self, n: usize) -> Style;
//
// /// Create a style from the given purple shade.
// /// n is `0..8`
// fn purple(&self, n: usize) -> Style;
//
// /// Create a style from the given magenta shade.
// /// n is `0..8`
// fn magenta(&self, n: usize) -> Style;
//
// /// Create a style from the given redpink shade.
// /// n is `0..8`
// fn redpink(&self, n: usize) -> Style;
//
// /// Create a style from the given primary shade.
// /// n is `0..8`
// fn primary(&self, n: usize) -> Style;
//
// /// Create a style from the given secondary shade.
// /// n is `0..8`
// fn secondary(&self, n: usize) -> Style;
//
// /// Style with only a fg color.
// fn text_light(&self) -> Style;
//
// /// Style with only a fg color.
// fn text_bright(&self) -> Style;
//
// /// Style with only a fg color.
// fn text_dark(&self) -> Style;
//
// /// Style with only a fg color.
// fn text_black(&self) -> Style;
//
// /// Create a style from a background color
// fn normal_style(&self, bg: Color) -> Style;
//
// /// Create a style from a background color
// fn high_style(&self, bg: Color) -> Style;
