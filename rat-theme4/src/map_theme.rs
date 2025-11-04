use crate::SalsaTheme;
use crate::palette::Palette;
use ratatui::style::Style;
use std::any::Any;
use std::collections::HashMap;

enum Entry {
    Style(Style),
    Fn(fn(&dyn SalsaTheme) -> Box<dyn Any>),
    FnClosure(Box<dyn Fn(&dyn SalsaTheme) -> Box<dyn Any> + 'static>),
}

pub struct MapTheme {
    pub name: Box<str>,
    pub p: Palette,
    styles: HashMap<&'static str, Entry>,
}

impl MapTheme {
    pub fn new(name: &str, s: Palette) -> Self {
        Self {
            p: s,
            name: Box::from(name),
            styles: Default::default(),
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

    fn define(&mut self, n: &'static str, style: Style) {
        self.styles.insert(n, Entry::Style(style));
    }

    fn define_fn(&mut self, w: &'static str, cr: fn(&dyn SalsaTheme) -> Box<dyn Any>) {
        self.styles.insert(w, Entry::Fn(cr));
    }

    fn define_closure(
        &mut self,
        w: &'static str,
        cr: Box<dyn Fn(&dyn SalsaTheme) -> Box<dyn Any> + 'static>,
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
}
