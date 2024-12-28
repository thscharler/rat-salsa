//!
//! A message dialog.
//!

use crate::_private::NonExhaustive;
use crate::button::{Button, ButtonOutcome, ButtonState, ButtonStyle};
use crate::layout::{layout_dialog, DialogItem};
use crate::paragraph::{Paragraph, ParagraphState};
use crate::util::{block_padding2, reset_buf_area};
use rat_event::{ct_event, ConsumedEvent, Dialog, HandleEvent, Outcome, Regular};
use rat_focus::{Focus, FocusBuilder};
use rat_scrolled::{Scroll, ScrollStyle};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Flex, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Text};
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::StatefulWidgetRef;
use ratatui::widgets::{Block, Padding, StatefulWidget, Widget};
use std::cell::{Cell, RefCell};
use std::cmp::max;
use std::fmt::Debug;

/// Basic status dialog for longer messages.
#[derive(Debug, Default, Clone)]
pub struct MsgDialog<'a> {
    style: Style,
    scroll_style: Option<ScrollStyle>,
    button_style: Option<ButtonStyle>,
    block: Option<Block<'a>>,
}

/// Combined style.
#[derive(Debug, Clone)]
pub struct MsgDialogStyle {
    pub style: Style,
    pub scroll: Option<ScrollStyle>,
    pub block: Option<Block<'static>>,
    pub button: Option<ButtonStyle>,

    pub non_exhaustive: NonExhaustive,
}

/// State & event handling.
#[derive(Debug, Clone)]
pub struct MsgDialogState {
    /// Full area.
    /// __readonly__. renewed for each render.
    pub area: Rect,
    /// Area inside the borders.
    /// __readonly__. renewed for each render.
    pub inner: Rect,

    /// Dialog is active.
    /// __read+write__
    pub active: Cell<bool>,
    /// Dialog title
    /// __read+write__
    pub message_title: RefCell<String>,
    /// Dialog text.
    /// __read+write__
    pub message: RefCell<String>,

    /// Ok button
    button: RefCell<ButtonState>,
    /// message-text
    paragraph: RefCell<ParagraphState>,
}

impl<'a> MsgDialog<'a> {
    /// New widget
    pub fn new() -> Self {
        Self {
            block: None,
            style: Default::default(),
            scroll_style: Default::default(),
            button_style: Default::default(),
        }
    }

    /// Block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Combined style
    pub fn styles(mut self, styles: MsgDialogStyle) -> Self {
        self.style = styles.style;
        if styles.scroll.is_some() {
            self.scroll_style = styles.scroll;
        }
        if styles.block.is_some() {
            self.block = styles.block;
        }
        if styles.button.is_some() {
            self.button_style = styles.button;
        }
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Base style
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Scroll style.
    pub fn scroll_style(mut self, style: ScrollStyle) -> Self {
        self.scroll_style = Some(style);
        self
    }

    /// Button style.
    pub fn button_style(mut self, style: ButtonStyle) -> Self {
        self.button_style = Some(style);
        self
    }
}

impl Default for MsgDialogStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            scroll: None,
            block: None,
            button: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl MsgDialogState {
    /// Show the dialog.
    pub fn set_active(&self, active: bool) {
        self.active.set(active);
        self.focus().focus(&*self.button.borrow());
        self.paragraph.borrow_mut().set_line_offset(0);
        self.paragraph.borrow_mut().set_col_offset(0);
    }

    /// Dialog is active.
    pub fn active(&self) -> bool {
        self.active.get()
    }

    /// Clear message text, set active to false.
    pub fn clear(&self) {
        self.active.set(false);
        *self.message.borrow_mut() = Default::default();
    }

    /// Set the title for the message.
    pub fn title(&self, title: impl Into<String>) {
        *self.message_title.borrow_mut() = title.into();
    }

    /// *Append* to the message.
    pub fn append(&self, msg: &str) {
        self.set_active(true);
        let mut message = self.message.borrow_mut();
        if !message.is_empty() {
            message.push('\n');
        }
        message.push_str(msg);
    }
}

impl Default for MsgDialogState {
    fn default() -> Self {
        let s = Self {
            active: Default::default(),
            area: Default::default(),
            inner: Default::default(),
            message: Default::default(),
            button: Default::default(),
            paragraph: Default::default(),
            message_title: Default::default(),
        };
        s.paragraph.borrow().focus.set(true);
        s
    }
}

impl MsgDialogState {
    fn focus(&self) -> Focus {
        let mut fb = FocusBuilder::default();
        fb.widget(&*self.paragraph.borrow())
            .widget(&*self.button.borrow());
        fb.build()
    }
}

#[cfg(feature = "unstable-widget-ref")]
impl<'a> StatefulWidgetRef for MsgDialog<'a> {
    type State = MsgDialogState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

impl<'a> StatefulWidget for MsgDialog<'a> {
    type State = MsgDialogState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

fn render_ref(widget: &MsgDialog<'_>, area: Rect, buf: &mut Buffer, state: &mut MsgDialogState) {
    if state.active.get() {
        let mut block;
        let title = state.message_title.borrow();
        let block = if let Some(b) = &widget.block {
            if !title.is_empty() {
                block = b.clone().title(title.as_str());
                &block
            } else {
                b
            }
        } else {
            block = Block::bordered()
                .style(widget.style)
                .padding(Padding::new(1, 1, 1, 1));
            if !title.is_empty() {
                block = block.title(title.as_str());
            }
            &block
        };

        let l_dlg = layout_dialog(
            area, //
            block_padding2(block),
            [Constraint::Length(10)],
            0,
            Flex::End,
        );
        state.area = l_dlg.area();
        state.inner = l_dlg.widget_for(DialogItem::Inner);

        reset_buf_area(state.area, buf);
        block.render(state.area, buf);

        {
            let scroll = if let Some(style) = &widget.scroll_style {
                Scroll::new().styles(style.clone())
            } else {
                Scroll::new().style(widget.style)
            };

            let message = state.message.borrow();
            let mut lines = Vec::new();
            for t in message.split('\n') {
                lines.push(Line::from(t));
            }
            let text = Text::from(lines).alignment(Alignment::Center);
            Paragraph::new(text).scroll(scroll).render(
                l_dlg.widget_for(DialogItem::Content),
                buf,
                &mut state.paragraph.borrow_mut(),
            );
        }

        Button::new("Ok")
            .styles_opt(widget.button_style.clone())
            .render(
                l_dlg.widget_for(DialogItem::Button(0)),
                buf,
                &mut state.button.borrow_mut(),
            );
    }
}

impl HandleEvent<crossterm::event::Event, Dialog, Outcome> for MsgDialogState {
    fn handle(&mut self, event: &crossterm::event::Event, _: Dialog) -> Outcome {
        if self.active.get() {
            let mut focus = self.focus();
            let f = focus.handle(event, Regular);

            let mut r = match self.button.borrow_mut().handle(event, Regular) {
                ButtonOutcome::Pressed => {
                    self.clear();
                    self.active.set(false);
                    Outcome::Changed
                }
                v => v.into(),
            };
            r = r.or_else(|| self.paragraph.borrow_mut().handle(event, Regular));
            r = r.or_else(|| match event {
                ct_event!(keycode press Esc) => {
                    self.clear();
                    self.active.set(false);
                    Outcome::Changed
                }
                _ => Outcome::Continue,
            });
            // mandatory consume everything else.
            max(max(Outcome::Unchanged, f), r)
        } else {
            Outcome::Continue
        }
    }
}

/// Handle events for the MsgDialog.
pub fn handle_dialog_events(
    state: &mut MsgDialogState,
    event: &crossterm::event::Event,
) -> Outcome {
    state.handle(event, Dialog)
}
