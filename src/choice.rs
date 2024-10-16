use crate::_private::NonExhaustive;
use rat_focus::{FocusBuilder, FocusFlag, HasFocus, IsFocusContainer, ZRect};
use rat_popup::PopupCoreState;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Block;
use std::borrow::Cow;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Choice<'a> {
    items: Vec<Cow<'a, str>>,
    disabled: Vec<bool>,

    style: Style,
    select_style: Option<Style>,
    focus_style: Option<Style>,
    disabled_style: Option<Style>,
    block: Option<Block<'a>>,

    popup_boundary: Option<Rect>,
    popup_len: u16,
    popup_block: Option<Block<'a>>,
}

#[derive(Debug)]
pub struct RenderChoice<'a> {
    choice: Rc<Choice<'a>>,
}

#[derive(Debug)]
pub struct RenderChoicePopup<'a> {
    choice: Rc<Choice<'a>>,
}

#[derive(Debug, Clone)]
pub struct ChoiceState {
    /// Total area.
    pub area: Rect,
    /// All areas
    pub z_areas: [ZRect; 2],
    /// Inner area.
    pub inner: Rect,

    /// Select item.
    pub selected: Option<usize>,

    /// Popup state
    pub popup: PopupCoreState,

    pub focus: FocusFlag,

    non_exhaustive: NonExhaustive,
}

impl<'a> Default for Choice<'a> {
    fn default() -> Self {
        Self {
            items: vec![],
            disabled: vec![],
            style: Default::default(),
            select_style: None,
            focus_style: None,
            disabled_style: None,
            block: None,
            popup_boundary: None,
            popup_len: 5,
            popup_block: None,
        }
    }
}

impl<'a> Choice<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn item(mut self, item: impl Into<Cow<'a, str>>) -> Self {
        self.items.push(item.into());
        self.disabled.push(false);
        self
    }

    pub fn disabled(mut self) -> Self {
        *self.disabled.last_mut() = true;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn select_style(mut self, style: Style) -> Self {
        self.select_style = Some(style);
        self
    }

    pub fn focus_style(mut self, style: Style) -> Self {
        self.focus_style = Some(style);
        self
    }

    pub fn disabled_style(mut self, style: Style) -> Self {
        self.disabled_style = Some(style);
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn popup_boundary(mut self, boundary: Rect) -> Self {
        self.popup_boundary = Some(boundary);
        self
    }

    pub fn popup_len(mut self, len: u16) -> Self {
        self.popup_len = len;
        self
    }

    pub fn popup_block(mut self, block: Block<'a>) -> Self {
        self.popup_block = Some(block);
        self
    }

    /// Create the widgets for the Choice.
    pub fn into_widgets(self) -> (RenderChoice<'a>, RenderChoicePopup<'a>) {
        let zelf = Rc::new(self);

        (
            RenderChoice {
                choice: zelf.clone(),
            },
            RenderChoicePopup {
                choice: zelf, //
            },
        )
    }
}

impl Default for ChoiceState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            z_areas: [Default::default(); 2],
            inner: Default::default(),
            selected: None,
            popup: Default::default(),
            focus: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl IsFocusContainer for ChoiceState {
    fn build(&self, builder: &mut FocusBuilder) {
        todo!()
    }
}

impl HasFocus for ChoiceState {
    fn focus(&self) -> FocusFlag {
        todo!()
    }

    fn area(&self) -> Rect {
        todo!()
    }
}

impl ChoiceState {
    pub fn new() -> Self {
        Self::default()
    }
}
