//!
//! A message dialog.
//!
//! ```
//! use ratatui_core::buffer::Buffer;
//! use ratatui_core::layout::Rect;
//! use ratatui_core::widgets::{StatefulWidget};
//! use ratatui_widgets::block::{Block};
//! use rat_event::{Dialog, HandleEvent, Outcome};
//! use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
//!
//! let mut state = MsgDialogState::new_active(
//!     "Warning",
//!     "This is some warning etc etc");
//!
//! # let area = Rect::new(5,5,60,15);
//! # let mut buf = Buffer::empty(area);
//! # let buf = &mut buf;
//!
//! MsgDialog::new()
//!     .block(Block::bordered())
//!     .render(area, buf, &mut state);
//!
//!
//! // ...
//!
//! # use ratatui_crossterm::crossterm::event::Event;
//! # let event = Event::FocusGained;//dummy
//! # let event = &event;
//! match state.handle(event, Dialog) {
//!     Outcome::Continue => {}
//!     Outcome::Unchanged | Outcome::Changed => { return; }
//! };
//!
//! ```
//!
//! The trick to make this work like a dialog is to render it
//! as the last thing during your rendering and to let it
//! handle events before any other widgets.
//!
//! Then it will be rendered on top of everything else and will
//! react to events first if it is `active`.
//!

use crate::_private::NonExhaustive;
use crate::button::{Button, ButtonState, ButtonStyle};
use crate::event::ButtonOutcome;
use crate::layout::{DialogItem, LayoutOuter, layout_dialog};
use crate::paragraph::{Paragraph, ParagraphState};
use crate::text::HasScreenCursor;
use crate::util::{block_padding2, reset_buf_area};
use rat_event::{ConsumedEvent, Dialog, HandleEvent, Outcome, Regular, ct_event};
use rat_focus::{Focus, FocusBuilder, FocusFlag, HasFocus};
use rat_reloc::RelocatableState;
use rat_scrolled::{Scroll, ScrollStyle};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Alignment, Constraint, Flex, Position, Rect, Size};
use ratatui_core::style::Style;
use ratatui_core::text::{Line, Text};
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_crossterm::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui_widgets::block::{Block, Padding};
use std::cell::{Cell, RefCell};
use std::cmp::max;
use std::fmt::Debug;

/// Basic status dialog for longer messages.
#[derive(Debug, Default, Clone)]
pub struct MsgDialog<'a> {
    style: Style,
    block: Option<Block<'a>>,
    scroll_style: Option<ScrollStyle>,
    button_style: Option<ButtonStyle>,

    layout: LayoutOuter,
}

/// Combined style.
#[derive(Debug, Clone)]
pub struct MsgDialogStyle {
    pub style: Style,
    pub block: Option<Block<'static>>,
    pub border_style: Option<Style>,
    pub title_style: Option<Style>,
    pub scroll: Option<ScrollStyle>,

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
        Self::default()
    }

    /// Block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Margin constraint for the left side.
    pub fn left(mut self, left: Constraint) -> Self {
        self.layout = self.layout.left(left);
        self
    }

    /// Margin constraint for the top side.
    pub fn top(mut self, top: Constraint) -> Self {
        self.layout = self.layout.top(top);
        self
    }

    /// Margin constraint for the right side.
    pub fn right(mut self, right: Constraint) -> Self {
        self.layout = self.layout.right(right);
        self
    }

    /// Margin constraint for the bottom side.
    pub fn bottom(mut self, bottom: Constraint) -> Self {
        self.layout = self.layout.bottom(bottom);
        self
    }

    /// Put at a fixed position.
    pub fn position(mut self, pos: Position) -> Self {
        self.layout = self.layout.position(pos);
        self
    }

    /// Constraint for the width.
    pub fn width(mut self, width: Constraint) -> Self {
        self.layout = self.layout.width(width);
        self
    }

    /// Constraint for the height.
    pub fn height(mut self, height: Constraint) -> Self {
        self.layout = self.layout.height(height);
        self
    }

    /// Set at a fixed size.
    pub fn size(mut self, size: Size) -> Self {
        self.layout = self.layout.size(size);
        self
    }

    /// Combined style
    pub fn styles(mut self, styles: MsgDialogStyle) -> Self {
        self.style = styles.style;
        if styles.block.is_some() {
            self.block = styles.block;
        }
        if let Some(border_style) = styles.border_style {
            self.block = self.block.map(|v| v.border_style(border_style));
        }
        if let Some(title_style) = styles.title_style {
            self.block = self.block.map(|v| v.title_style(title_style));
        }
        self.block = self.block.map(|v| v.style(self.style));

        if styles.scroll.is_some() {
            self.scroll_style = styles.scroll;
        }

        if styles.button.is_some() {
            self.button_style = styles.button;
        }

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
            block: Default::default(),
            border_style: Default::default(),
            title_style: Default::default(),
            scroll: Default::default(),
            button: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl HasFocus for MsgDialogState {
    fn build(&self, _builder: &mut FocusBuilder) {
        // don't expose inner workings.
    }

    fn focus(&self) -> FocusFlag {
        unimplemented!("not available")
    }

    fn area(&self) -> Rect {
        unimplemented!("not available")
    }
}

impl RelocatableState for MsgDialogState {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.area.relocate(shift, clip);
        self.inner.relocate(shift, clip);
        self.button.borrow_mut().relocate(shift, clip);
        self.paragraph.borrow_mut().relocate(shift, clip);
    }
}

impl HasScreenCursor for MsgDialogState {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        None
    }
}

impl MsgDialogState {
    pub fn new() -> Self {
        Self::default()
    }

    /// New dialog with active-flag set.
    pub fn new_active(title: impl Into<String>, msg: impl AsRef<str>) -> Self {
        let zelf = Self::default();
        zelf.set_active(true);
        zelf.title(title);
        zelf.append(msg.as_ref());
        zelf
    }

    /// Show the dialog.
    pub fn set_active(&self, active: bool) {
        self.active.set(active);
        self.build_focus().focus(&*self.paragraph.borrow());
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
    fn build_focus(&self) -> Focus {
        let mut fb = FocusBuilder::default();
        fb.widget(&*self.paragraph.borrow())
            .widget(&*self.button.borrow());
        fb.build()
    }
}

impl<'a> StatefulWidget for &MsgDialog<'a> {
    type State = MsgDialogState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

impl StatefulWidget for MsgDialog<'_> {
    type State = MsgDialogState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

fn render_ref(widget: &MsgDialog<'_>, area: Rect, buf: &mut Buffer, state: &mut MsgDialogState) {
    state.area = area;

    if !state.active.get() {
        return;
    }

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

impl HandleEvent<Event, Dialog, Outcome> for MsgDialogState {
    fn handle(&mut self, event: &Event, _: Dialog) -> Outcome {
        if self.active.get() {
            let mut focus = self.build_focus();
            let f = focus.handle(event, Regular);

            let mut r = match self
                .button
                .borrow_mut()
                .handle(event, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
            {
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
pub fn handle_dialog_events(state: &mut MsgDialogState, event: &Event) -> Outcome {
    state.handle(event, Dialog)
}
