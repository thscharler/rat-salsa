//!
//! Clears an area with a Style and a fill char.
//!
//! There is Buffer::set_style() to fill an area with a style.
//! What is missing is the ability to overwrite the text-content too.
//!
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Widget;
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::WidgetRef;

/// Fill the area with a grapheme and a style.
/// Useful when overwriting an already rendered buffer
/// for overlays or windows.
#[derive(Debug, Default)]
pub struct Fill<'a> {
    c: Option<&'a str>,
    style: Option<Style>,
}

impl<'a> Fill<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the fill char as one graphem.
    pub fn fill_char(mut self, c: &'a str) -> Self {
        self.c = Some(c);
        self
    }

    /// Set the fill style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }
}

#[cfg(feature = "unstable-widget-ref")]
impl<'a> WidgetRef for Fill<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        render_ref(self, area, buf);
    }
}

impl<'a> Widget for Fill<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        render_ref(&self, area, buf);
    }
}

fn render_ref(widget: &Fill<'_>, area: Rect, buf: &mut Buffer) {
    let area = buf.area.intersection(area);
    match (widget.c, widget.style) {
        (Some(c), Some(s)) => {
            for y in area.top()..area.bottom() {
                for x in area.left()..area.right() {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.reset();
                        cell.set_symbol(c);
                        cell.set_style(s);
                    }
                }
            }
        }
        (None, Some(s)) => {
            for y in area.top()..area.bottom() {
                for x in area.left()..area.right() {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.reset();
                        cell.set_style(s);
                    }
                }
            }
        }
        (Some(c), None) => {
            for y in area.top()..area.bottom() {
                for x in area.left()..area.right() {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.reset();
                        cell.set_symbol(c);
                    }
                }
            }
        }
        (None, None) => {
            for y in area.top()..area.bottom() {
                for x in area.left()..area.right() {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.reset();
                    }
                }
            }
        }
    }
}
