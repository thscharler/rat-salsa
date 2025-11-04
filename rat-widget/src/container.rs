use crate::_private::NonExhaustive;
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::BlockExt;
use ratatui::style::Style;
use ratatui::widgets::{Block, StatefulWidget, Widget};

///
/// Empty Container widget.
///
/// Renders the Block and the background and gives
/// you the inner area where you can render your other widgets.
///
#[derive(Debug, Default)]
pub struct Container<'a> {
    style: Style,
    symbol: Option<&'a str>,
    block: Option<Block<'a>>,
}

/// All styles of the container.
#[derive(Debug)]
pub struct ContainerStyle {
    pub style: Style,
    pub symbol: Option<&'static str>,
    pub block: Option<Block<'static>>,
    pub non_exhaustive: NonExhaustive,
}

/// Container state. Optional
#[derive(Debug, Default)]
pub struct ContainerState {
    pub area: Rect,
    pub widget_area: Rect,

    pub focus: FocusFlag,
}

impl Default for ContainerStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            symbol: Default::default(),
            block: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<'a> Container<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn styles(mut self, styles: ContainerStyle) -> Self {
        self.style = styles.style;
        self.block = self.block.map(|v| v.style(self.style));
        if let Some(symbol) = styles.symbol {
            self.symbol = Some(symbol);
        }
        if let Some(block) = styles.block {
            self.block = Some(block);
        }
        self
    }

    pub fn widget_area(&self, area: Rect) -> Rect {
        self.block.inner_if_some(area)
    }
}

impl Widget for &Container<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        render_widget(self, area, buf);
    }
}

impl Widget for Container<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        render_widget(&self, area, buf);
    }
}

fn render_widget(widget: &Container<'_>, area: Rect, buf: &mut Buffer) {
    if let Some(block) = &widget.block {
        block.render(area, buf);

        if let Some(symbol) = widget.symbol {
            for y in area.top()..area.bottom() {
                for x in area.left()..area.right() {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_symbol(symbol);
                    }
                }
            }
        }
    } else {
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.reset();
                    if let Some(symbol) = widget.symbol {
                        cell.set_symbol(symbol);
                    }
                    cell.set_style(widget.style);
                }
            }
        }
    }
}

impl<'a> StatefulWidget for Container<'a> {
    type State = ContainerState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        state.widget_area = self.block.inner_if_some(area);

        render_widget(&self, area, buf);
    }
}

impl<'a> StatefulWidget for &Container<'a> {
    type State = ContainerState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        state.widget_area = self.block.inner_if_some(area);

        render_widget(self, area, buf);
    }
}

impl Clone for ContainerState {
    fn clone(&self) -> Self {
        Self {
            area: self.area,
            widget_area: self.widget_area,
            focus: Default::default(),
        }
    }
}

impl HasFocus for ContainerState {
    fn build(&self, _builder: &mut FocusBuilder) {}

    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }
}
