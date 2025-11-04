use crate::SalsaTheme;
use crate::palette::{Contrast, Palette};
use crate::{Named, Widget};
use ratatui::style::{Color, Style};
use std::any::{Any, type_name};
use std::collections::HashMap;

pub struct MapTheme {
    pub name: Box<str>,
    pub p: Palette,
    pub named: HashMap<&'static str, Style>,
    pub widgets: HashMap<&'static str, fn(&dyn SalsaTheme) -> Box<dyn Any>>,
}

impl MapTheme {
    pub fn new(name: &str, s: Palette) -> Self {
        Self {
            p: s,
            name: Box::from(name),
            named: Default::default(),
            widgets: Default::default(),
        }
    }
}

impl SalsaTheme for MapTheme {
    /// Some display name.
    fn name(&self) -> &str {
        &self.name
    }

    /// The underlying palette.
    fn palette(&self) -> &Palette {
        &self.p
    }

    /// Create a style from the given white shade.
    /// n is `0..8`
    fn white(&self, n: usize) -> Style {
        self.p.white(n, Contrast::Normal)
    }

    /// Create a style from the given black shade.
    /// n is `0..8`
    fn black(&self, n: usize) -> Style {
        self.p.black(n, Contrast::Normal)
    }

    /// Create a style from the given gray shade.
    /// n is `0..8`
    fn gray(&self, n: usize) -> Style {
        self.p.gray(n, Contrast::Normal)
    }

    /// Create a style from the given red shade.
    /// n is `0..8`
    fn red(&self, n: usize) -> Style {
        self.p.red(n, Contrast::Normal)
    }

    /// Create a style from the given orange shade.
    /// n is `0..8`
    fn orange(&self, n: usize) -> Style {
        self.p.orange(n, Contrast::Normal)
    }

    /// Create a style from the given yellow shade.
    /// n is `0..8`
    fn yellow(&self, n: usize) -> Style {
        self.p.yellow(n, Contrast::Normal)
    }

    /// Create a style from the given limegreen shade.
    /// n is `0..8`
    fn limegreen(&self, n: usize) -> Style {
        self.p.limegreen(n, Contrast::Normal)
    }

    /// Create a style from the given green shade.
    /// n is `0..8`
    fn green(&self, n: usize) -> Style {
        self.p.green(n, Contrast::Normal)
    }

    /// Create a style from the given bluegreen shade.
    /// n is `0..8`
    fn bluegreen(&self, n: usize) -> Style {
        self.p.bluegreen(n, Contrast::Normal)
    }

    /// Create a style from the given cyan shade.
    /// n is `0..8`
    fn cyan(&self, n: usize) -> Style {
        self.p.cyan(n, Contrast::Normal)
    }

    /// Create a style from the given blue shade.
    /// n is `0..8`
    fn blue(&self, n: usize) -> Style {
        self.p.blue(n, Contrast::Normal)
    }

    /// Create a style from the given deepblue shade.
    /// n is `0..8`
    fn deepblue(&self, n: usize) -> Style {
        self.p.deepblue(n, Contrast::Normal)
    }

    /// Create a style from the given purple shade.
    /// n is `0..8`
    fn purple(&self, n: usize) -> Style {
        self.p.purple(n, Contrast::Normal)
    }

    /// Create a style from the given magenta shade.
    /// n is `0..8`
    fn magenta(&self, n: usize) -> Style {
        self.p.magenta(n, Contrast::Normal)
    }

    /// Create a style from the given redpink shade.
    /// n is `0..8`
    fn redpink(&self, n: usize) -> Style {
        self.p.redpink(n, Contrast::Normal)
    }

    /// Create a style from the given primary shade.
    /// n is `0..8`
    fn primary(&self, n: usize) -> Style {
        self.p.primary(n, Contrast::Normal)
    }

    /// Create a style from the given secondary shade.
    /// n is `0..8`
    fn secondary(&self, n: usize) -> Style {
        self.p.secondary(n, Contrast::Normal)
    }

    fn text_light(&self) -> Style {
        Style::new().fg(self.p.text_light)
    }

    fn text_bright(&self) -> Style {
        Style::new().fg(self.p.text_bright)
    }

    fn text_dark(&self) -> Style {
        Style::new().fg(self.p.text_dark)
    }

    fn text_black(&self) -> Style {
        Style::new().fg(self.p.text_black)
    }

    /// Create a style from a background color
    fn style(&self, bg: Color) -> Style {
        self.p.style(bg, Contrast::Normal)
    }

    /// Create a style from a background color
    fn high_style(&self, bg: Color) -> Style {
        self.p.style(bg, Contrast::High)
    }

    fn add_named(&mut self, n: &'static str, style: Style) {
        self.named.insert(n, style);
    }

    fn named(&self, n: &str) -> Style {
        if cfg!(debug_assertions) {
            self.named
                .get(n)
                .expect(format!("unknown style {}", n).as_str())
                .clone()
        } else {
            Style::default()
        }
    }

    fn add_widget(&mut self, w: &'static str, cr: fn(&dyn SalsaTheme) -> Box<dyn Any>) {
        self.widgets.insert(w, cr);
    }

    fn widget<O: Default + Sized + 'static>(&self, w: &str) -> O {
        if cfg!(debug_assertions) {
            let create = self
                .widgets
                .get(w)
                .expect(format!("unknown widget {}", w).as_str());
            let style = create(self);
            let style = style
                .downcast::<O>()
                .expect(format!("downcast fails for {} to {}", w, type_name::<O>()).as_str());
            *style
        } else {
            let Some(create) = self.widgets.get(w) else {
                return O::default();
            };
            let style = create(self);
            let Ok(style) = style.downcast::<O>() else {
                return O::default();
            };
            *style
        }
    }
}
