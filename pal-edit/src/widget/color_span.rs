use rat_theme4::palette::Palette;
use rat_widget::color_input::{ColorInput, ColorInputState};
use rat_widget::reloc::RelocatableState;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Style;
use ratatui_core::widgets::StatefulWidget;

#[derive(Default, Debug)]
pub struct ColorSpan<'a> {
    half: bool,
    dark: u8,
    color0: ColorInput<'a>,
    color3: ColorInput<'a>,
}

pub struct ColorSpanState<'a> {
    pub color0: &'a mut ColorInputState,
    pub color3: &'a mut ColorInputState,
}

impl<'a> RelocatableState for ColorSpanState<'a> {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.color0.relocate(shift, clip);
        self.color3.relocate(shift, clip);
    }
}

impl<'a> ColorSpan<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn half(mut self) -> Self {
        self.half = true;
        self
    }

    pub fn dark(mut self, dark: u8) -> Self {
        self.dark = dark;
        self
    }

    pub fn color0(mut self, color: ColorInput<'a>) -> Self {
        self.color0 = color;
        self
    }

    pub fn color3(mut self, color: ColorInput<'a>) -> Self {
        self.color3 = color;
        self
    }
}

impl<'a> StatefulWidget for ColorSpan<'a> {
    type State = ColorSpanState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.color0
            .render(Rect::new(area.x, area.y, 16, 1), buf, state.color0);
        self.color3
            .render(Rect::new(area.x + 17, area.y, 16, 1), buf, state.color3);

        if self.half {
            let width = (area.width.saturating_sub(33)) / 4;
            let colors = Palette::interpolate(
                state.color0.value_u32(),
                state.color3.value_u32(),
                self.dark,
            );
            for i in 0usize..4usize {
                let color_area =
                    Rect::new(area.x + 34 + (i as u16) * width, area.y, width, area.height);
                buf.set_style(color_area, Style::new().bg(colors[i]));
            }
        } else {
            let width = (area.width.saturating_sub(33)) / 8;
            let colors = Palette::interpolate(
                state.color0.value_u32(),
                state.color3.value_u32(),
                self.dark,
            );
            for i in 0usize..8usize {
                let color_area =
                    Rect::new(area.x + 34 + (i as u16) * width, area.y, width, area.height);
                buf.set_style(color_area, Style::new().bg(colors[i]));
            }
        }
    }
}
