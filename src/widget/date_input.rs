use crate::_private::NonExhaustive;
use crate::{
    CanValidate, ControlUI, DefaultKeys, FocusFlag, FrameWidget, HandleCrossterm, HasFocusFlag,
    HasValidFlag, ValidFlag,
};
use chrono::NaiveDate;
use crossterm::event::Event;
#[allow(unused_imports)]
use log::debug;
use rat_input::date_input::{DateInput, DateInputState};
use rat_input::event::HandleEvent;
use rat_input::masked_input::MaskedInputStyle;
use rat_input::Outcome;
use ratatui::layout::{Position, Rect};
use ratatui::prelude::Style;
use ratatui::Frame;
use std::fmt;
use std::fmt::Debug;

#[derive(Debug, Default)]
pub struct DateInputExt<'a> {
    widget: DateInput<'a>,
}

impl<'a> DateInputExt<'a> {
    /// Set the combined style.
    pub fn styles(mut self, style: MaskedInputStyle) -> Self {
        self.widget = self.widget.styles(style);
        self
    }

    /// Base text style.
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.style(style);
        self
    }

    /// Style when focused.
    pub fn focus_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.focus_style(style);
        self
    }

    /// Style for selection
    pub fn select_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.select_style(style);
        self
    }

    /// Style for the invalid indicator.
    pub fn invalid_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.invalid_style(style);
        self
    }
}

impl<'a> FrameWidget for DateInputExt<'a> {
    type State = DateInputStateExt;

    fn render(mut self, frame: &mut Frame<'_>, area: Rect, state: &mut Self::State) {
        state.area = area;

        self.widget = self
            .widget
            .focused(state.is_focused())
            .valid(state.is_valid());

        frame.render_stateful_widget(self.widget, area, &mut state.widget);

        if let Some(Position { x, y }) = state.screen_cursor() {
            frame.set_cursor(x, y);
        }
    }
}

#[derive(Debug)]
pub struct DateInputStateExt {
    pub widget: DateInputState,
    /// Focus
    pub focus: FocusFlag,
    /// Area
    pub area: Rect,
    /// Valid.
    pub valid: ValidFlag,

    pub non_exhaustive: NonExhaustive,
}

impl Default for DateInputStateExt {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            focus: Default::default(),
            area: Default::default(),
            valid: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl DateInputStateExt {
    /// Reset to empty.
    pub fn reset(&mut self) {
        self.widget.reset();
    }

    /// chrono format string.
    ///
    /// generates a mask according to the format and overwrites whatever
    /// set_mask() did.
    pub fn set_format<S: AsRef<str>>(&mut self, pattern: S) -> Result<(), fmt::Error> {
        self.widget.set_format(pattern)
    }

    /// chrono format string.
    ///
    /// generates a mask according to the format and overwrites whatever
    /// set_mask() did.
    pub fn set_formats<S: AsRef<str>>(
        &mut self,
        pattern: S,
        locale: chrono::Locale,
    ) -> Result<(), fmt::Error> {
        self.widget.set_formats(pattern, locale)
    }

    /// Set a display mask overlay.
    pub fn set_display_mask<S: Into<String>>(&mut self, display: S) {
        self.widget.widget.set_display_mask(display);
    }

    pub fn value(&self) -> Result<NaiveDate, chrono::ParseError> {
        self.widget.value()
    }

    pub fn set_value(&mut self, date: NaiveDate) {
        self.widget.set_value(date);
    }

    pub fn select_all(&mut self) {
        self.widget.select_all();
    }

    pub fn screen_cursor(&self) -> Option<Position> {
        self.widget.screen_cursor()
    }
}

/// Add convenience keys:
/// * `h` `t` - today
/// * `a` `b` - January, 1st
/// * `e` - December, 31st
/// * `l` - first of last month
/// * `L` - last of last month
/// * `m` - first of this month
/// * `M` - last of this month
/// * `n` - first of next month
/// * `N` - last of next month
/// * `j` - add month
/// * `k` - subtract month
/// * `J` - add year
/// * `K` - subtract year
pub use rat_input::date_input::ConvenientKeys;

impl<A, E> HandleCrossterm<ControlUI<A, E>, ConvenientKeys> for DateInputStateExt
where
    E: From<fmt::Error>,
{
    fn handle(&mut self, event: &Event, _keymap: ConvenientKeys) -> ControlUI<A, E> {
        match self.widget.handle(event, ConvenientKeys) {
            Ok(Outcome::Changed) => ControlUI::Change,
            Ok(Outcome::NotUsed) => ControlUI::Continue,
            Ok(Outcome::Unchanged) => ControlUI::NoChange,
            Err(e) => ControlUI::Err(e.into()),
        }
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>, DefaultKeys> for DateInputStateExt
where
    E: From<fmt::Error>,
{
    fn handle(&mut self, event: &Event, _keymap: DefaultKeys) -> ControlUI<A, E> {
        let focus = self.is_focused();
        match rat_input::date_input::handle_events(&mut self.widget, focus, event) {
            Ok(Outcome::Changed) => ControlUI::Change,
            Ok(Outcome::Unchanged) => ControlUI::NoChange,
            Ok(Outcome::NotUsed) => ControlUI::Continue,
            Err(e) => ControlUI::Err(e.into()),
        }
    }
}

impl HasFocusFlag for DateInputStateExt {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl HasValidFlag for DateInputStateExt {
    fn valid(&self) -> &ValidFlag {
        &self.valid
    }
}

impl CanValidate for DateInputStateExt {
    fn validate(&mut self) {
        if let Some(d) = self.set_valid_from(self.value()) {
            self.set_value(d);
        }
    }
}
