//!
//! A standard dialog frame and buttons.
//!

use crate::_private::NonExhaustive;
use crossterm::event::Event;
use rat_widget::button::{Button, ButtonState, ButtonStyle};
use rat_widget::event::{
    ButtonOutcome, ConsumedEvent, Dialog, HandleEvent, Outcome, Regular, ct_event, flow,
};
use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus};
use rat_widget::layout::{DialogItem, layout_dialog};
use rat_widget::util::{block_padding2, fill_buf_area};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Position, Rect, Size};
use ratatui::style::Style;
use ratatui::widgets::{Block, BorderType, StatefulWidget, Widget};

/// Renders the frame and the Ok/Cancel buttons for a dialog window.
///
/// After rendering BaseDialogState::widget_area is available
/// to render any content.
#[derive(Debug, Default)]
pub struct BaseDialog<'a> {
    style: Style,
    block: Block<'a>,
    button_style: ButtonStyle,
    constrain_position: Option<[Constraint; 2]>,
    constrain_size: Option<[Constraint; 2]>,
    position: Option<Position>,
    size: Option<Size>,
    ok_text: &'a str,
    cancel_text: &'a str,
}

/// Combined styles.
#[derive(Debug, Clone)]
pub struct BaseDialogStyle {
    pub style: Style,
    pub block: Option<Block<'static>>,
    pub button_style: Option<ButtonStyle>,
    pub constrain_position: Option<[Constraint; 2]>,
    pub constrain_size: Option<[Constraint; 2]>,
    pub position: Option<Position>,
    pub size: Option<Size>,
    pub ok_text: Option<&'static str>,
    pub cancel_text: Option<&'static str>,
    pub non_exhaustive: NonExhaustive,
}

impl Default for BaseDialogStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            block: Default::default(),
            button_style: Default::default(),
            constrain_position: Default::default(),
            constrain_size: Default::default(),
            position: Default::default(),
            size: Default::default(),
            ok_text: Default::default(),
            cancel_text: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BaseDialogState {
    /// Area for the dialog.
    /// __read only__ set with each render.
    pub area: Rect,
    /// Area for the dialog-content.
    /// __read only__ set with each render.
    pub widget_area: Rect,

    /// ok-button
    pub ok: ButtonState,
    /// cancel-button
    pub cancel: ButtonState,
}

impl<'a> BaseDialog<'a> {
    pub fn new() -> Self {
        Self {
            style: Default::default(),
            block: Block::bordered().border_type(BorderType::Plain),
            button_style: Default::default(),
            constrain_position: Some([Constraint::Percentage(19), Constraint::Length(3)]),
            constrain_size: Some([Constraint::Percentage(19), Constraint::Length(3)]),
            position: None,
            size: None,
            ok_text: "Ok",
            cancel_text: "Cancel",
        }
    }

    pub fn styles(mut self, styles: BaseDialogStyle) -> Self {
        self.style = styles.style;
        if let Some(block) = styles.block {
            self.block = block;
        }
        if let Some(button_style) = styles.button_style {
            self.button_style = button_style;
        }
        if let Some(constraints) = styles.constrain_position {
            self.constrain_position = Some(constraints);
        }
        if let Some(constraints) = styles.constrain_size {
            self.constrain_size = Some(constraints);
        }
        if let Some(constraints) = styles.position {
            self.position = Some(constraints);
        }
        if let Some(constraints) = styles.size {
            self.size = Some(constraints);
        }
        if let Some(ok_text) = styles.ok_text {
            self.ok_text = ok_text;
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

    /// Button style.
    pub fn button_style(mut self, style: ButtonStyle) -> Self {
        self.button_style = style;
        self
    }

    /// Margin constraints for the top-left corner.
    pub fn constrain_position(mut self, constraints: [Constraint; 2]) -> Self {
        self.constrain_position = Some(constraints);
        self
    }

    /// Margin constraints for the bottom-right corner.
    pub fn constrain_size(mut self, constraints: [Constraint; 2]) -> Self {
        self.constrain_position = Some(constraints);
        self
    }

    /// Put at a fixed position.
    pub fn position(mut self, pos: Position) -> Self {
        self.position = Some(pos);
        self
    }

    /// Set at a fixed size.
    pub fn size(mut self, size: Size) -> Self {
        self.size = Some(size);
        self
    }
}

impl<'a> StatefulWidget for BaseDialog<'a> {
    type State = BaseDialogState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mut hor = [Constraint::Fill(1); 3];
        let mut ver = [Constraint::Fill(1); 3];

        if let Some(constraints) = self.constrain_position {
            ver[0] = constraints[0];
            hor[0] = constraints[1];
        }
        if let Some(constraint) = self.constrain_size {
            ver[2] = constraint[0];
            hor[2] = constraint[1];
        }
        if let Some(pos) = self.position {
            ver[0] = Constraint::Length(pos.y);
            hor[0] = Constraint::Length(pos.x);
        }
        if let Some(size) = self.size {
            ver[1] = Constraint::Length(size.height);
            hor[1] = Constraint::Length(size.width);
        }

        let h_layout = Layout::horizontal(hor).split(area);
        let v_layout = Layout::vertical(ver).split(h_layout[1]);
        state.area = v_layout[1];

        let l_dlg = layout_dialog(
            state.area,
            block_padding2(&self.block),
            [Constraint::Length(12), Constraint::Length(10)],
            1,
            Flex::End,
        );
        state.widget_area = l_dlg.widget_for(DialogItem::Content);

        fill_buf_area(buf, l_dlg.area(), " ", self.style);
        self.block.render(state.area, buf);

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

impl Default for BaseDialogState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            widget_area: Default::default(),
            ok: Default::default(),
            cancel: Default::default(),
        }
    }
}

impl HasFocus for BaseDialogState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget(&self.ok);
        builder.widget(&self.cancel);
    }

    fn focus(&self) -> FocusFlag {
        unimplemented!()
    }

    fn area(&self) -> Rect {
        unimplemented!()
    }
}

impl BaseDialogState {
    pub fn new() -> Self {
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

impl<'a> HandleEvent<Event, Dialog, DialogOutcome> for BaseDialogState {
    fn handle(&mut self, event: &Event, _: Dialog) -> DialogOutcome {
        flow!(match self.cancel.handle(event, Regular) {
            ButtonOutcome::Pressed => {
                DialogOutcome::Cancel
            }
            r => Outcome::from(r).into(),
        });
        flow!(match self.ok.handle(event, Regular) {
            ButtonOutcome::Pressed => {
                DialogOutcome::Ok
            }
            r => Outcome::from(r).into(),
        });

        flow!(match event {
            ct_event!(keycode press Esc) => {
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
