use crate::dark_theme::dark_theme;
use crate::map_theme::MapTheme;
use crate::palette::Palette;
use ratatui::style::{Color, Style};
use std::any::{Any, type_name};

mod boxed;
mod dark_theme;
mod map_theme;
mod palette;
pub mod palettes;
mod test;

pub struct WidgetStyle;

impl WidgetStyle {
    pub const CONTAINER: &'static str = "container";
    pub const SCROLL: &'static str = "scroll";

    pub const POPUP_SCROLL: &'static str = "scroll.popup";
    pub const DIALOG_SCROLL: &'static str = "scroll.dialog";
}

pub struct NamedStyle;

impl NamedStyle {
    pub const FOCUS: &'static str = "focus";
    pub const SELECT: &'static str = "select";
}

pub enum ThemeColor {
    C0 = 0,
    C1,
    C2,
    C3,
    D0,
    D1,
    D2,
    D3,
}

#[macro_export]
macro_rules! make_dyn {
    ($fn_name:ident) => {
        |theme: &dyn $crate::SalsaTheme| {
            let theme = (theme as &dyn core::any::Any)
                .downcast_ref::<$crate::map_theme::MapTheme>()
                .expect("map-theme");
            Box::new($fn_name(theme))
        }
    };
}

pub trait SalsaTheme: Any {
    /// Theme name.
    fn name(&self) -> &str;

    /// Color palette.
    fn p(&self) -> &Palette;

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

    /// Add a named base_style.
    fn add_named(&mut self, n: &'static str, style: Style);

    /// Get a named base-style.
    fn named(&self, n: &str) -> Style;

    fn add_style(&mut self, w: &'static str, cr: fn(&dyn SalsaTheme) -> Box<dyn Any>);

    fn dyn_style(&self, w: &str) -> Option<Box<dyn Any>>;

    /// Get the style for the named widget.
    fn style<O: Default + Sized + 'static>(&self, w: &str) -> O
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
pub fn create_theme(theme: &str) -> Option<Box<dyn SalsaTheme>> {
    use crate::palettes::*;
    let theme: Box<dyn SalsaTheme> = match theme {
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
        //
        // "Imperial Shell" => Box::new(ShellTheme::new(theme, IMPERIAL)),
        // "Radium Shell" => Box::new(ShellTheme::new(theme, RADIUM)),
        // "Tundra Shell" => Box::new(ShellTheme::new(theme, TUNDRA)),
        // "Ocean Shell" => Box::new(ShellTheme::new(theme, OCEAN)),
        // "Monochrome Shell" => Box::new(ShellTheme::new(theme, MONOCHROME)),
        // "Black & White Shell" => Box::new(ShellTheme::new(theme, BLACKWHITE)),
        // "Base16 Shell" => Box::new(ShellTheme::new(theme, BASE16)),
        // "Base16 Relax Shell" => Box::new(ShellTheme::new(theme, BASE16_RELAX)),
        // "Monekai Shell" => Box::new(ShellTheme::new(theme, MONEKAI)),
        // "Solarized Shell" => Box::new(ShellTheme::new(theme, SOLARIZED)),
        // "OxoCarbon Shell" => Box::new(ShellTheme::new(theme, OXOCARBON)),
        // "Rust Shell" => Box::new(ShellTheme::new(theme, RUST)),
        // "VSCode Shell" => Box::new(ShellTheme::new(theme, VSCODE_DARK)),
        //
        _ => return None,
    };

    Some(theme)
}

/// Create an empty SalsaTheme.
pub fn create_empty(name: &str, p: Palette) -> Box<dyn SalsaTheme> {
    Box::new(MapTheme::new(name, p))
}
