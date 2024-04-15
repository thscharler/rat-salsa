use crate::widget::mask_input::{MaskedInput, MaskedInputState, MaskedInputStyle};
use crate::{
    CanValidate, ControlUI, DefaultKeys, FocusFlag, FrameWidget, HandleCrossterm, HasFocusFlag,
    HasValidFlag, ValidFlag,
};
use chrono::format::{Fixed, Item, Numeric, Pad, StrftimeItems};
use chrono::{Datelike, Days, Local, Months, NaiveDate};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
#[allow(unused_imports)]
use log::debug;
use ratatui::layout::{Margin, Rect};
use ratatui::prelude::Style;
use ratatui::Frame;
use std::fmt;
use std::fmt::Debug;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Default)]
pub struct DateInput {
    pub input: MaskedInput,
}

impl DateInput {
    /// Use extra insets for the text input.
    pub fn insets(mut self, insets: Margin) -> Self {
        self.input = self.input.insets(insets);
        self
    }

    /// Use our own cursor indicator or the terminal cursor.
    pub fn terminal_cursor(mut self, terminal: bool) -> Self {
        self.input = self.input.terminal_cursor(terminal);
        self
    }

    /// Do accept keyboard events event without being focused.
    /// Useful for a catch field, eg "find stuff"
    pub fn without_focus(mut self, without_focus: bool) -> Self {
        self.input = self.input.without_focus(without_focus);
        self
    }

    /// Set the combined style.
    pub fn style(mut self, style: MaskedInputStyle) -> Self {
        self.input = self.input.style(style);
        self
    }

    /// Base text style.
    pub fn base_style(mut self, style: impl Into<Style>) -> Self {
        self.input = self.input.base_style(style);
        self
    }

    /// Style when focused.
    pub fn focus_style(mut self, style: impl Into<Style>) -> Self {
        self.input = self.input.focus_style(style);
        self
    }

    /// Style for selection
    pub fn select_style(mut self, style: impl Into<Style>) -> Self {
        self.input = self.input.select_style(style);
        self
    }

    /// Style for our own cursor.
    pub fn cursor_style(mut self, style: impl Into<Style>) -> Self {
        self.input = self.input.cursor_style(style);
        self
    }

    /// Style for the invalid indicator.
    pub fn invalid_style(mut self, style: impl Into<Style>) -> Self {
        self.input = self.input.invalid_style(style);
        self
    }

    /// Marker character for invalid field.
    pub fn invalid_char(mut self, invalid: char) -> Self {
        self.input = self.input.invalid_char(invalid);
        self
    }
}

impl FrameWidget for DateInput {
    type State = DateInputState;

    fn render(self, frame: &mut Frame<'_>, area: Rect, state: &mut Self::State) {
        if state.lost_focus() {
            state.validate();
        }

        self.input.render(frame, area, &mut state.input);
    }
}

#[derive(Debug, Default)]
pub struct DateInputState {
    pub input: MaskedInputState,
    pub pattern: String,
    pub locale: chrono::Locale,
}

impl DateInputState {
    /// Reset to empty.
    pub fn reset(&mut self) {
        self.input.reset();
    }

    /// chrono format string.
    ///
    /// generates a mask according to the format and overwrites whatever
    /// set_mask() did.
    pub fn set_format<S: AsRef<str>>(&mut self, pattern: S) -> Result<(), fmt::Error> {
        self.set_formats(pattern, chrono::Locale::default())
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
        debug!("pattern {}", pattern.as_ref());
        let mut mask = String::new();
        let items = StrftimeItems::new_with_locale(pattern.as_ref(), locale)
            .parse()
            .map_err(|_| fmt::Error)?;
        for t in &items {
            debug!("{:?}", t);
            match t {
                Item::Literal(s) => {
                    for c in s.graphemes(true) {
                        mask.push('\\');
                        mask.push_str(c);
                    }
                }
                Item::OwnedLiteral(s) => {
                    for c in s.graphemes(true) {
                        mask.push('\\');
                        mask.push_str(c);
                    }
                }
                Item::Space(s) => {
                    for c in s.graphemes(true) {
                        mask.push_str(c);
                    }
                }
                Item::OwnedSpace(s) => {
                    for c in s.graphemes(true) {
                        mask.push_str(c);
                    }
                }
                Item::Numeric(v, Pad::None | Pad::Space) => match v {
                    Numeric::Year | Numeric::IsoYear => mask.push_str("9999"),
                    Numeric::YearDiv100
                    | Numeric::YearMod100
                    | Numeric::IsoYearDiv100
                    | Numeric::IsoYearMod100
                    | Numeric::Month
                    | Numeric::Day
                    | Numeric::WeekFromSun
                    | Numeric::WeekFromMon
                    | Numeric::IsoWeek
                    | Numeric::Hour
                    | Numeric::Hour12
                    | Numeric::Minute
                    | Numeric::Second => mask.push_str("99"),
                    Numeric::NumDaysFromSun | Numeric::WeekdayFromMon => mask.push('9'),
                    Numeric::Ordinal => mask.push_str("999"),
                    Numeric::Nanosecond => mask.push_str("999999999"),
                    Numeric::Timestamp => mask.push_str("###########"),
                    _ => return Err(fmt::Error),
                },
                Item::Numeric(v, Pad::Zero) => match v {
                    Numeric::Year | Numeric::IsoYear => mask.push_str("0000"),
                    Numeric::YearDiv100
                    | Numeric::YearMod100
                    | Numeric::IsoYearDiv100
                    | Numeric::IsoYearMod100
                    | Numeric::Month
                    | Numeric::Day
                    | Numeric::WeekFromSun
                    | Numeric::WeekFromMon
                    | Numeric::IsoWeek
                    | Numeric::Hour
                    | Numeric::Hour12
                    | Numeric::Minute
                    | Numeric::Second => mask.push_str("00"),
                    Numeric::NumDaysFromSun | Numeric::WeekdayFromMon => mask.push('0'),
                    Numeric::Ordinal => mask.push_str("000"),
                    Numeric::Nanosecond => mask.push_str("000000000"),
                    Numeric::Timestamp => mask.push_str("#0000000000"),
                    _ => return Err(fmt::Error),
                },
                Item::Fixed(v) => match v {
                    Fixed::ShortMonthName => mask.push_str("___"),
                    Fixed::LongMonthName => mask.push_str("_________"),
                    Fixed::ShortWeekdayName => mask.push_str("___"),
                    Fixed::LongWeekdayName => mask.push_str("________"),
                    Fixed::LowerAmPm => mask.push_str("__"),
                    Fixed::UpperAmPm => mask.push_str("__"),
                    Fixed::Nanosecond => mask.push_str(".#########"),
                    Fixed::Nanosecond3 => mask.push_str(".###"),
                    Fixed::Nanosecond6 => mask.push_str(".######"),
                    Fixed::Nanosecond9 => mask.push_str(".#########"),
                    Fixed::TimezoneName => mask.push_str("__________"),
                    Fixed::TimezoneOffsetColon | Fixed::TimezoneOffset => mask.push_str("+##:##"),
                    Fixed::TimezoneOffsetDoubleColon => mask.push_str("+##:##:##"),
                    Fixed::TimezoneOffsetTripleColon => mask.push_str("+##"),
                    Fixed::TimezoneOffsetColonZ | Fixed::TimezoneOffsetZ => return Err(fmt::Error),
                    Fixed::RFC2822 => {
                        // 01 Jun 2016 14:31:46 -0700
                        mask.push_str("00 ___ 0000 00:00:00 +0000")
                    }
                    Fixed::RFC3339 => {
                        // not supported, for now
                        return Err(fmt::Error);
                    }
                    _ => return Err(fmt::Error),
                },
                Item::Error => return Err(fmt::Error),
            }
        }

        self.locale = locale;
        self.pattern = pattern.as_ref().to_string();
        self.set_mask(mask)?;
        Ok(())
    }

    /// Overlay for empty fields.
    pub fn set_display_mask<S: Into<String>>(&mut self, s: S) {
        self.input.set_display_mask(s);
    }

    /// Mask as defined by [MaskedInput]
    pub fn set_mask<S: AsRef<str>>(&mut self, s: S) -> Result<(), fmt::Error> {
        self.input.set_mask(s)
    }

    pub fn value(&self) -> Result<NaiveDate, chrono::ParseError> {
        NaiveDate::parse_from_str(self.input.compact_value().as_str(), self.pattern.as_str())
    }

    pub fn set_value(&mut self, date: NaiveDate) {
        let v = date.format(self.pattern.as_str()).to_string();
        self.input.set_value(v);
    }

    pub fn select_all(&mut self) {
        self.input.select_all()
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
#[derive(Debug)]
pub struct ConvenienceKeys;

impl<A, E> HandleCrossterm<ControlUI<A, E>, ConvenienceKeys> for DateInputState
where
    E: From<fmt::Error>,
{
    fn handle(&mut self, event: &Event, _keymap: ConvenienceKeys) -> ControlUI<A, E> {
        let r = {
            match event {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('h') | KeyCode::Char('t'),
                    kind: KeyEventKind::Press,
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => 'f: {
                    if !self.is_focused() {
                        break 'f ControlUI::Continue;
                    }
                    self.set_value(Local::now().date_naive());
                    ControlUI::Change
                }

                Event::Key(KeyEvent {
                    code: KeyCode::Char('l'),
                    kind: KeyEventKind::Press,
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => 'f: {
                    if !self.is_focused() {
                        break 'f ControlUI::Continue;
                    }
                    let date = Local::now()
                        .date_naive()
                        .checked_sub_months(Months::new(1))
                        .expect("month")
                        .with_day(1)
                        .expect("day");
                    self.set_value(date);
                    ControlUI::Change
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('L'),
                    kind: KeyEventKind::Press,
                    modifiers: KeyModifiers::SHIFT,
                    ..
                }) => 'f: {
                    if !self.is_focused() {
                        break 'f ControlUI::Continue;
                    }
                    let date = Local::now()
                        .date_naive()
                        .with_day(1)
                        .expect("month")
                        .checked_sub_days(Days::new(1))
                        .expect("day");
                    self.set_value(date);
                    ControlUI::Change
                }

                Event::Key(KeyEvent {
                    code: KeyCode::Char('m'),
                    kind: KeyEventKind::Press,
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => 'f: {
                    if !self.is_focused() {
                        break 'f ControlUI::Continue;
                    }
                    let date = Local::now().date_naive().with_day(1).expect("day");
                    self.set_value(date);
                    ControlUI::Change
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('M'),
                    kind: KeyEventKind::Press,
                    modifiers: KeyModifiers::SHIFT,
                    ..
                }) => 'f: {
                    if !self.is_focused() {
                        break 'f ControlUI::Continue;
                    }
                    let date = Local::now()
                        .date_naive()
                        .checked_add_months(Months::new(1))
                        .expect("month")
                        .with_day(1)
                        .expect("day")
                        .checked_sub_days(Days::new(1))
                        .expect("day");
                    self.set_value(date);
                    ControlUI::Change
                }

                Event::Key(KeyEvent {
                    code: KeyCode::Char('n'),
                    kind: KeyEventKind::Press,
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => 'f: {
                    if !self.is_focused() {
                        break 'f ControlUI::Continue;
                    }
                    let date = Local::now()
                        .date_naive()
                        .checked_add_months(Months::new(1))
                        .expect("month")
                        .with_day(1)
                        .expect("day");
                    self.set_value(date);
                    ControlUI::Change
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('N'),
                    kind: KeyEventKind::Press,
                    modifiers: KeyModifiers::SHIFT,
                    ..
                }) => 'f: {
                    if !self.is_focused() {
                        break 'f ControlUI::Continue;
                    }
                    let date = Local::now()
                        .date_naive()
                        .checked_add_months(Months::new(2))
                        .expect("month")
                        .with_day(1)
                        .expect("day")
                        .checked_sub_days(Days::new(1))
                        .expect("day");
                    self.set_value(date);
                    ControlUI::Change
                }

                Event::Key(KeyEvent {
                    code: KeyCode::Char('j'),
                    kind: KeyEventKind::Press,
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => 'f: {
                    if !self.is_focused() {
                        break 'f ControlUI::Continue;
                    }
                    if let Ok(date) = self.value() {
                        let date = date.checked_add_months(Months::new(1)).expect("month");
                        self.set_value(date);
                    }
                    ControlUI::Change
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('J'),
                    kind: KeyEventKind::Press,
                    modifiers: KeyModifiers::SHIFT,
                    ..
                }) => 'f: {
                    if !self.is_focused() {
                        break 'f ControlUI::Continue;
                    }
                    if let Ok(date) = self.value() {
                        let date = date.with_year(date.year() + 1).expect("year");
                        self.set_value(date);
                    }
                    ControlUI::Change
                }

                Event::Key(KeyEvent {
                    code: KeyCode::Char('k'),
                    kind: KeyEventKind::Press,
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => 'f: {
                    if !self.is_focused() {
                        break 'f ControlUI::Continue;
                    }
                    if let Ok(date) = self.value() {
                        let date = date.checked_sub_months(Months::new(1)).expect("month");
                        self.set_value(date);
                    }
                    ControlUI::Change
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('K'),
                    kind: KeyEventKind::Press,
                    modifiers: KeyModifiers::SHIFT,
                    ..
                }) => 'f: {
                    if !self.is_focused() {
                        break 'f ControlUI::Continue;
                    }
                    if let Ok(date) = self.value() {
                        let date = date.with_year(date.year() - 1).expect("year");
                        self.set_value(date);
                    }
                    ControlUI::Change
                }

                Event::Key(KeyEvent {
                    code: KeyCode::Char('a') | KeyCode::Char('b'),
                    kind: KeyEventKind::Press,
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => 'f: {
                    if !self.is_focused() {
                        break 'f ControlUI::Continue;
                    }
                    if let Ok(date) = self.value() {
                        let date = date.with_month(1).expect("month").with_day(1).expect("day");
                        self.set_value(date);
                    }
                    ControlUI::Change
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('e'),
                    kind: KeyEventKind::Press,
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => 'f: {
                    if !self.is_focused() {
                        break 'f ControlUI::Continue;
                    }
                    if let Ok(date) = self.value() {
                        let date = date
                            .with_month(12)
                            .expect("month")
                            .with_day(31)
                            .expect("day");
                        self.set_value(date);
                    }
                    ControlUI::Change
                }

                _ => ControlUI::Continue,
            }
        };

        r.or_else(|| {
            let r = self.input.handle(event, DefaultKeys);
            r.on_change_do(|| {
                self.input.set_valid_from(self.value());
            });
            r
        })
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>, DefaultKeys> for DateInputState
where
    E: From<fmt::Error>,
{
    fn handle(&mut self, event: &Event, _keymap: DefaultKeys) -> ControlUI<A, E> {
        let r = self.input.handle(event, DefaultKeys);
        r.on_change_do(|| {
            self.input.set_valid_from(self.value());
        });
        r
    }
}

impl HasFocusFlag for DateInputState {
    fn focus(&self) -> &FocusFlag {
        &self.input.focus
    }

    fn area(&self) -> Rect {
        self.input.area
    }
}

impl HasValidFlag for DateInputState {
    fn valid(&self) -> &ValidFlag {
        &self.input.valid
    }
}

impl CanValidate for DateInputState {
    fn validate(&mut self) {
        if let Some(d) = self.set_valid_from(self.value()) {
            self.set_value(d);
        }
    }
}
