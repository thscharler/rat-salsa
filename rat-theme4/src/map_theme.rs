use crate::SalsaTheme;
use crate::palette::Palette;
use ratatui::style::Style;
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
    fn p(&self) -> &Palette {
        &self.p
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

    fn add_style(&mut self, w: &'static str, cr: fn(&dyn SalsaTheme) -> Box<dyn Any>) {
        self.widgets.insert(w, cr);
    }

    fn dyn_style(&self, w: &str) -> Option<Box<dyn Any>> {
        if cfg!(debug_assertions) {
            let create = self
                .widgets
                .get(w)
                .expect(format!("unknown widget {}", w).as_str());
            Some(create(self))
        } else {
            None
        }
    }

    fn style<O: Default + Sized + 'static>(&self, w: &str) -> O {
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
