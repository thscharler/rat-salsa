use crate::widget::table::TableExtState;
use crate::widget::{ActionTrigger, HasVerticalScroll};
use crate::{ControlUI, DefaultKeys, FocusFlag, HandleCrossterm, HasFocusFlag, MouseOnly};
use crossterm::event::Event;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{StatefulWidget, Style};
use ratatui::style::Styled;
use ratatui::widgets::{Block, HighlightSpacing, List, ListDirection, ListItem, ListState};

///
/// Extensions for [ratatui::widgets::List]
///
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct ListExt<'a> {
    pub list: List<'a>,
    /// Style for selected + not focused.
    pub select_style: Style,
    /// Style for selected + focused.
    pub focus_style: Style,
}

/// Combined style
#[derive(Debug, Default)]
pub struct ListExtStyle {
    pub style: Style,
    pub select_style: Style,
    pub focus_style: Style,
}

impl<'a> ListExt<'a> {
    pub fn new<T>(items: T) -> Self
    where
        T: IntoIterator,
        T::Item: Into<ListItem<'a>>,
    {
        Self {
            list: List::new(items),
            select_style: Default::default(),
            focus_style: Default::default(),
        }
    }

    pub fn items<T>(mut self, items: T) -> Self
    where
        T: IntoIterator,
        T::Item: Into<ListItem<'a>>,
    {
        self.list = self.list.items(items);
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.list = self.list.block(block);
        self
    }

    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.list = self.list.style(style);
        self
    }

    pub fn highlight_symbol(mut self, highlight_symbol: &'a str) -> Self {
        self.list = self.list.highlight_symbol(highlight_symbol);
        self
    }

    pub fn highlight_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.list = self.list.highlight_style(style);
        self
    }

    pub fn repeat_highlight_symbol(mut self, repeat: bool) -> Self {
        self.list = self.list.repeat_highlight_symbol(repeat);
        self
    }

    pub fn highlight_spacing(mut self, value: HighlightSpacing) -> Self {
        self.list = self.list.highlight_spacing(value);
        self
    }

    pub fn direction(mut self, direction: ListDirection) -> Self {
        self.list = self.list.direction(direction);
        self
    }

    pub fn scroll_padding(mut self, padding: usize) -> Self {
        self.list = self.list.scroll_padding(padding);
        self
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn styles(mut self, styles: ListExtStyle) -> Self {
        self.list = self.list.style(styles.style);
        self.select_style = styles.select_style;
        self.focus_style = styles.focus_style;
        self
    }

    pub fn select_style<S: Into<Style>>(mut self, select_style: S) -> Self {
        self.select_style = select_style.into();
        self
    }

    pub fn focus_style<S: Into<Style>>(mut self, focus_style: S) -> Self {
        self.focus_style = focus_style.into();
        self
    }
}

impl<'a> Styled for ListExt<'a> {
    type Item = ListExt<'a>;

    fn style(&self) -> Style {
        <List<'_> as Styled>::style(&self.list)
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.list = self.list.set_style(style);
        self
    }
}

impl<'a, Item> FromIterator<Item> for ListExt<'a>
where
    Item: Into<ListItem<'a>>,
{
    fn from_iter<Iter: IntoIterator<Item = Item>>(iter: Iter) -> Self {
        Self {
            list: List::new(iter),
            select_style: Default::default(),
            focus_style: Default::default(),
        }
    }
}

impl<'a> StatefulWidget for ListExt<'a> {
    type State = ListExtState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        state.len = self.len();

        if state.gained_focus() {
            if state.list_state.selected().is_none() {
                state.list_state.select(Some(0))
            }
        }

        let list = if state.focus.get() {
            self.list.highlight_style(self.focus_style)
        } else {
            self.list.highlight_style(self.select_style)
        };

        StatefulWidget::render(list, area, buf, &mut state.list_state);
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ListExtState {
    pub focus: FocusFlag,
    pub area: Rect,
    pub len: usize,
    pub trigger: ActionTrigger,
    pub list_state: ListState,
}

impl HasFocusFlag for ListExtState {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl HasVerticalScroll for ListExtState {
    fn vlen(&self) -> usize {
        self.len
    }

    fn voffset(&self) -> usize {
        self.list_state.offset()
    }

    fn set_voffset(&mut self, offset: usize) {
        *self.list_state.offset_mut() = offset;
    }

    fn vpage(&self) -> usize {
        self.area.height as usize
    }
}

impl ListExtState {
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.list_state = self.list_state.with_offset(offset);
        self
    }

    pub fn with_selected(mut self, selected: Option<usize>) -> Self {
        self.list_state = self.list_state.with_selected(selected);
        self
    }

    pub fn offset(&self) -> usize {
        self.list_state.offset()
    }

    pub fn offset_mut(&mut self) -> &mut usize {
        self.list_state.offset_mut()
    }

    pub fn selected(&self) -> Option<usize> {
        self.list_state.selected()
    }

    pub fn selected_mut(&mut self) -> &mut Option<usize> {
        self.list_state.selected_mut()
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.list_state.select(index);
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>> for ListExtState {
    fn handle(&mut self, event: &Event, _: DefaultKeys) -> ControlUI<A, E> {
        ControlUI::Continue
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for TableExtState {
    fn handle(&mut self, event: &Event, _: MouseOnly) -> ControlUI<A, E> {
        ControlUI::Continue
    }
}
