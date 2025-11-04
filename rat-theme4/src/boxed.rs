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

    fn define(&mut self, n: &'static str, style: Style) {
        self.as_mut().define(n, style);
    }

    fn define_fn(&mut self, w: &'static str, cr: fn(&dyn SalsaTheme) -> Box<dyn Any>) {
        self.as_mut().define_fn(w, cr);
    }

    fn define_closure(
        &mut self,
        w: &'static str,
        cr: Box<dyn Fn(&dyn SalsaTheme) -> Box<dyn Any> + 'static>,
    ) {
        self.as_mut().define_closure(w, cr);
    }

    fn dyn_style(&self, w: &str) -> Option<Box<dyn Any + 'static>> {
        self.as_ref().dyn_style(w)
    }
}
