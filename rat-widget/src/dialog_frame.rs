//!
//! A simple dialog frame and buttons.
//!
use crate::_private::NonExhaustive;
use crate::button::{Button, ButtonState, ButtonStyle};
use crate::event::{
    ButtonOutcome, ConsumedEvent, Dialog, HandleEvent, Outcome, Regular, ct_event, flow,
};
use crate::focus::{FocusBuilder, FocusFlag, HasFocus};
use crate::layout::{DialogItem, LayoutOuter, layout_dialog};
use crate::text::HasScreenCursor;
use crate::util::{block_padding2, fill_buf_area};
use crossterm::event::Event;
use rat_event::MouseOnly;
use rat_reloc::RelocatableState;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Position, Rect, Size};
use ratatui::style::Style;
use ratatui::widgets::{Block, BorderType, StatefulWidget, Widget};

/// Renders the frame and the Ok/Cancel buttons for a dialog window.
///
/// After rendering BaseDialogState::widget_area is available
/// to render any content.
#[derive(Debug, Clone, Default)]
pub struct DialogFrame<'a> {
    style: Style,
    block: Block<'a>,
    button_style: ButtonStyle,
    layout: LayoutOuter,
    ok_text: &'a str,
    no_cancel: bool,
    cancel_text: &'a str,
}

/// Combined styles.
#[derive(Debug, Clone)]
pub struct DialogFrameStyle {
    pub style: Style,
    pub block: Option<Block<'static>>,
    pub border_style: Option<Style>,
    pub title_style: Option<Style>,
    pub button_style: Option<ButtonStyle>,
    pub layout: Option<LayoutOuter>,
    pub ok_text: Option<&'static str>,
    pub no_cancel: Option<bool>,
    pub cancel_text: Option<&'static str>,
    pub non_exhaustive: NonExhaustive,
}

impl Default for DialogFrameStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            block: Default::default(),
            border_style: Default::default(),
            title_style: Default::default(),
            button_style: Default::default(),
            layout: Default::default(),
            ok_text: Default::default(),
            no_cancel: Default::default(),
            cancel_text: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DialogFrameState {
    /// Area for the dialog.
    /// __read only__ set with each render.
    pub area: Rect,
    /// Area for the dialog-content.
    /// __read only__ set with each render.
    pub widget_area: Rect,

    /// ok-button
    pub ok: ButtonState,
    /// no cancel button
    pub no_cancel: bool,
    /// cancel-button
    pub cancel: ButtonState,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> DialogFrame<'a> {
    pub fn new() -> Self {
        Self {
            style: Default::default(),
            block: Block::bordered().border_type(BorderType::Plain),
            button_style: Default::default(),
            layout: LayoutOuter::new()
                .left(Constraint::Percentage(19))
                .top(Constraint::Length(3))
                .right(Constraint::Percentage(19))
                .bottom(Constraint::Length(3)),
            ok_text: "Ok",
            no_cancel: false,
            cancel_text: "Cancel",
        }
    }

    pub fn styles(mut self, styles: DialogFrameStyle) -> Self {
        self.style = styles.style;
        self.block = self.block.style(self.style);
        if let Some(border_style) = styles.border_style {
            self.block = self.block.border_style(border_style);
        }
        if let Some(title_style) = styles.title_style {
            self.block = self.block.title_style(title_style);
        }
        if let Some(block) = styles.block {
            self.block = block;
        }
        if let Some(button_style) = styles.button_style {
            self.button_style = button_style;
        }
        if let Some(layout) = styles.layout {
            self.layout = layout;
        }
        if let Some(ok_text) = styles.ok_text {
            self.ok_text = ok_text;
        }
        if let Some(no_cancel) = styles.no_cancel {
            self.no_cancel = no_cancel;
        }
        if let Some(cancel_text) = styles.cancel_text {
            self.cancel_text = cancel_text;
        }
        self
    }

    /// Base style for the dialog.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Block for the dialog.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = block;
        self
    }

    /// Sets the border-style for the Block, if any.
    pub fn border_style(mut self, style: Style) -> Self {
        self.block = self.block.border_style(style);
        self
    }

    /// Sets the title-style for the Block, if any.
    pub fn title_style(mut self, style: Style) -> Self {
        self.block = self.block.title_style(style);
        self
    }

    /// Button style.
    pub fn button_style(mut self, style: ButtonStyle) -> Self {
        self.button_style = style;
        self
    }

    /// Ok text.
    pub fn ok_text(mut self, str: &'a str) -> Self {
        self.ok_text = str;
        self
    }

    /// No cancel button.
    pub fn no_cancel(mut self) -> Self {
        self.no_cancel = true;
        self
    }

    /// Cancel text.
    pub fn cancel_text(mut self, str: &'a str) -> Self {
        self.cancel_text = str;
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

    /// Calculate the resulting dialog area.
    /// Returns the inner area that is available for drawing widgets.
    pub fn layout_size(&self, area: Rect) -> Rect {
        let area = self.layout.layout(area);
        let l_dlg = if self.no_cancel {
            layout_dialog(
                area,
                block_padding2(&self.block),
                [Constraint::Length(10)],
                1,
                Flex::End,
            )
        } else {
            layout_dialog(
                area,
                block_padding2(&self.block),
                [Constraint::Length(12), Constraint::Length(10)],
                1,
                Flex::End,
            )
        };
        l_dlg.widget_for(DialogItem::Content)
    }
}

impl<'a> StatefulWidget for DialogFrame<'a> {
    type State = DialogFrameState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = self.layout.layout(area);
        state.no_cancel = self.no_cancel;

        let l_dlg = if self.no_cancel {
            layout_dialog(
                state.area,
                block_padding2(&self.block),
                [Constraint::Length(10)],
                1,
                Flex::End,
            )
        } else {
            layout_dialog(
                state.area,
                block_padding2(&self.block),
                [Constraint::Length(12), Constraint::Length(10)],
                1,
                Flex::End,
            )
        };
        state.widget_area = l_dlg.widget_for(DialogItem::Content);

        fill_buf_area(buf, l_dlg.area(), " ", self.style);
        self.block.render(state.area, buf);

        if self.no_cancel {
            Button::new(self.ok_text).styles(self.button_style).render(
                l_dlg.widget_for(DialogItem::Button(0)),
                buf,
                &mut state.ok,
            );
        } else {
            Button::new(self.cancel_text)
                .styles(self.button_style.clone())
                .render(
                    l_dlg.widget_for(DialogItem::Button(0)),
                    buf,
                    &mut state.cancel,
                );
            Button::new(self.ok_text).styles(self.button_style).render(
                l_dlg.widget_for(DialogItem::Button(1)),
                buf,
                &mut state.ok,
            );
        }
    }
}

impl Default for DialogFrameState {
    fn default() -> Self {
        let z = Self {
            area: Default::default(),
            widget_area: Default::default(),
            ok: Default::default(),
            no_cancel: Default::default(),
            cancel: Default::default(),
            non_exhaustive: NonExhaustive,
        };
        z.ok.focus.set(true);
        z
    }
}

impl HasFocus for DialogFrameState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget(&self.ok);
        if !self.no_cancel {
            builder.widget(&self.cancel);
        }
    }

    fn focus(&self) -> FocusFlag {
        unimplemented!()
    }

    fn area(&self) -> Rect {
        unimplemented!()
    }
}

impl HasScreenCursor for DialogFrameState {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        None
    }
}

impl RelocatableState for DialogFrameState {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.area.relocate(shift, clip);
        self.widget_area.relocate(shift, clip);
        self.ok.relocate(shift, clip);
        self.cancel.relocate(shift, clip);
    }
}

impl DialogFrameState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn named(_name: &str) -> Self {
        Self::default()
    }
}

/// Result type for event-handling.
pub enum DialogOutcome {
    /// Continue with event-handling.
    /// In the event-loop this waits for the next event.
    Continue,
    /// Break event-handling without repaint.
    /// In the event-loop this waits for the next event.
    Unchanged,
    /// Break event-handling and repaints/renders the application.
    /// In the event-loop this calls `render`.
    Changed,
    /// Ok pressed
    Ok,
    /// Cancel pressed
    Cancel,
}

impl ConsumedEvent for DialogOutcome {
    fn is_consumed(&self) -> bool {
        !matches!(self, DialogOutcome::Continue)
    }
}

impl From<DialogOutcome> for Outcome {
    fn from(value: DialogOutcome) -> Self {
        match value {
            DialogOutcome::Continue => Outcome::Continue,
            DialogOutcome::Unchanged => Outcome::Unchanged,
            DialogOutcome::Changed => Outcome::Changed,
            DialogOutcome::Ok => Outcome::Changed,
            DialogOutcome::Cancel => Outcome::Changed,
        }
    }
}

impl From<Outcome> for DialogOutcome {
    fn from(value: Outcome) -> Self {
        match value {
            Outcome::Continue => DialogOutcome::Continue,
            Outcome::Unchanged => DialogOutcome::Unchanged,
            Outcome::Changed => DialogOutcome::Changed,
        }
    }
}

impl<'a> HandleEvent<Event, Dialog, DialogOutcome> for DialogFrameState {
    fn handle(&mut self, event: &Event, _: Dialog) -> DialogOutcome {
        flow!({
            if !self.no_cancel {
                match self.cancel.handle(event, Regular) {
                    ButtonOutcome::Pressed => DialogOutcome::Cancel,
                    r => Outcome::from(r).into(),
                }
            } else {
                DialogOutcome::Continue
            }
        });
        flow!(match self.ok.handle(event, Regular) {
            ButtonOutcome::Pressed => {
                DialogOutcome::Ok
            }
            r => Outcome::from(r).into(),
        });

        flow!(match event {
            ct_event!(keycode press Esc) if !self.no_cancel => {
                DialogOutcome::Cancel
            }
            ct_event!(keycode press Enter) | ct_event!(keycode press F(12)) => {
                DialogOutcome::Ok
            }
            _ => DialogOutcome::Unchanged,
        });

        DialogOutcome::Unchanged
    }
}

impl<'a> HandleEvent<Event, MouseOnly, DialogOutcome> for DialogFrameState {
    fn handle(&mut self, event: &Event, _: MouseOnly) -> DialogOutcome {
        flow!({
            if !self.no_cancel {
                match self.cancel.handle(event, MouseOnly) {
                    ButtonOutcome::Pressed => DialogOutcome::Cancel,
                    r => Outcome::from(r).into(),
                }
            } else {
                DialogOutcome::Continue
            }
        });
        flow!(match self.ok.handle(event, MouseOnly) {
            ButtonOutcome::Pressed => {
                DialogOutcome::Ok
            }
            r => Outcome::from(r).into(),
        });

        DialogOutcome::Unchanged
    }
}

/// Handle events for the popup.
/// Call before other handlers to deal with intersections
/// with other widgets.
pub fn handle_events(state: &mut DialogFrameState, _focus: bool, event: &Event) -> DialogOutcome {
    HandleEvent::handle(state, event, Dialog)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(state: &mut DialogFrameState, event: &Event) -> DialogOutcome {
    HandleEvent::handle(state, event, MouseOnly)
}
