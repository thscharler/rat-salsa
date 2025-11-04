use crate::SalsaTheme;
use crate::palette::Palette;
use ratatui::style::Style;
use std::any::Any;

impl SalsaTheme for Box<dyn SalsaTheme> {
    fn name(&self) -> &str {
        self.as_ref().name()
    }

    fn p(&self) -> &Palette {
        self.as_ref().p()
    }

    fn add_named(&mut self, n: &'static str, style: Style) {
        self.as_mut().add_named(n, style);
    }

    fn named(&self, n: &str) -> Style {
        self.as_ref().named(n)
    }

    fn add_style(&mut self, w: &'static str, cr: fn(&dyn SalsaTheme) -> Box<dyn Any>) {
        self.as_mut().add_style(w, cr);
    }

    fn dyn_style(&self, w: &str) -> Option<Box<(dyn Any + 'static)>> {
        self.as_ref().dyn_style(w)
    }
}
