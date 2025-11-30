//!
//! Input field for IBAN bank account numbers.
//!
//! Verifies the checksum.
//!
//! Shows the bank account grouped by 4 chars and
//! takes account of the different lengths per country.
//!
use crate::_private::NonExhaustive;
use rat_event::{HandleEvent, MouseOnly, Regular};
use rat_text::date_input::DateInputState;
use rat_text::event::{ReadOnly, TextOutcome};
use rat_text::text_input_mask::{MaskedInput, MaskedInputState};
use rat_text::{
    TextError, TextFocusGained, TextFocusLost, TextStyle, TextTab, derive_text_widget,
    derive_text_widget_state,
};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Style;
use ratatui_core::widgets::StatefulWidget;
use ratatui_crossterm::crossterm::event::Event;
use std::cmp::min;
use std::str::FromStr;

/// Widget for IBAN.
#[derive(Debug, Default, Clone)]
pub struct IBANInput<'a> {
    widget: MaskedInput<'a>,
}

/// Widget state.
#[derive(Debug, Clone)]
pub struct IBANInputState {
    /// __read only__ renewed with each render.
    pub area: Rect,
    /// __read only__ renewed with each render.
    pub inner: Rect,
    /// __read+write__  
    pub widget: MaskedInputState,

    /// __read only__ Current country mask.
    pub country: String,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> IBANInput<'a> {
    pub fn new() -> Self {
        Self::default()
    }
}

derive_text_widget!(IBANInput<'a>);

impl<'a> StatefulWidget for &IBANInput<'a> {
    type State = IBANInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        (&self.widget).render(area, buf, &mut state.widget);

        state.area = state.widget.area;
        state.inner = state.widget.inner;
    }
}

impl StatefulWidget for IBANInput<'_> {
    type State = IBANInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget.render(area, buf, &mut state.widget);

        state.area = state.widget.area;
        state.inner = state.widget.inner;
    }
}

impl Default for IBANInputState {
    fn default() -> Self {
        let mut z = Self {
            area: Default::default(),
            inner: Default::default(),
            widget: Default::default(),
            country: Default::default(),
            non_exhaustive: NonExhaustive,
        };
        _ = z.widget.set_mask("ll");
        z
    }
}

derive_text_widget_state!(IBANInputState);

impl IBANInputState {
    /// New state.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn named(name: &str) -> Self {
        let mut z = Self::default();
        z.widget.focus = z.widget.focus.with_name(name);
        z
    }

    /// Reset to empty.
    #[inline]
    pub fn clear(&mut self) {
        self.widget.clear();
    }

    /// Validate the IBAN.
    /// Sets the invalid flag.
    /// Returns true if the IBAN is valid.
    pub fn validate(&mut self) -> Result<bool, TextError> {
        let iban = from_display_format(&self.country, self.widget.text());

        if self.country.as_str() != country_code(&iban) {
            let cursor = self.widget.cursor();
            let anchor = self.widget.anchor();
            self.widget.set_mask(pattern(&iban)).expect("valid_mask");
            self.country = country_code(&iban).to_string();
            self.widget.set_text(to_display_format(&iban));
            self.widget.set_selection(anchor, cursor);
        }

        let r = if !iban.trim().is_empty() {
            valid_iban_country(&iban) && valid_check_sum(&iban)
        } else {
            true
        };

        self.widget.set_invalid(!r);
        Ok(r)
    }

    /// Set the IBAN.
    ///
    /// Invalid IBANs are acceptable, but they will
    /// be truncated to 35 characters. If an invalid IBAN
    /// is set the widget will immediately set the invalid
    /// flag too.
    ///
    pub fn set_value(&mut self, iban: impl AsRef<str>) {
        let iban = iban.as_ref();
        self.country = country_code(iban).to_string();
        self.widget.set_mask(pattern(iban)).expect("valid mask");
        self.widget.set_text(to_display_format(iban));
        let valid = if !iban.trim().is_empty() {
            valid_iban_country(&iban) && valid_check_sum(&iban)
        } else {
            true
        };
        self.widget.set_invalid(!valid);
    }

    /// Get the IBAN.
    ///
    /// This will return invalid IBANs too.
    pub fn value(&self) -> String {
        let txt = self.widget.text();
        from_display_format(&self.country, txt)
    }

    /// Get the IBAN.
    ///
    /// This will only return valid IBANs or empty strings.
    pub fn valid_value(&self) -> Result<String, TextError> {
        let txt = self.widget.text();
        let iban = from_display_format(&self.country, txt);
        if iban.trim().is_empty() {
            Ok(Default::default())
        } else if is_valid_iban(&iban) {
            Ok(iban)
        } else {
            Err(TextError::InvalidValue)
        }
    }
}

impl HandleEvent<Event, Regular, TextOutcome> for IBANInputState {
    fn handle(&mut self, event: &Event, _keymap: Regular) -> TextOutcome {
        match self.widget.handle(event, Regular) {
            TextOutcome::TextChanged => {
                if let Err(_) = self.validate() {
                    self.set_invalid(true);
                }
                TextOutcome::TextChanged
            }
            r => r,
        }
    }
}

impl HandleEvent<Event, ReadOnly, TextOutcome> for IBANInputState {
    fn handle(&mut self, event: &Event, _keymap: ReadOnly) -> TextOutcome {
        self.widget.handle(event, ReadOnly)
    }
}

impl HandleEvent<Event, MouseOnly, TextOutcome> for IBANInputState {
    fn handle(&mut self, event: &Event, _keymap: MouseOnly) -> TextOutcome {
        self.widget.handle(event, MouseOnly)
    }
}

/// Handle all events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_events(state: &mut IBANInputState, focus: bool, event: &Event) -> TextOutcome {
    state.widget.focus.set(focus);
    state.handle(event, Regular)
}

/// Handle only navigation events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_readonly_events(
    state: &mut IBANInputState,
    focus: bool,
    event: &Event,
) -> TextOutcome {
    state.widget.focus.set(focus);
    state.handle(event, ReadOnly)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(state: &mut DateInputState, event: &Event) -> TextOutcome {
    state.handle(event, MouseOnly)
}

/// Is the IBAN correct.
/// - valid country code
/// - valid length
/// - valid checksum
pub fn is_valid_iban(iban: &str) -> bool {
    if !valid_iban_country(iban) {
        return false;
    }
    if iban_len(iban) != Some(iban.len() as u8) {
        return false;
    }
    if !valid_check_sum(iban) {
        return false;
    }
    true
}

static IBAN: &'static [(&'static str, u8)] = &[
    ("AD", 24),
    ("AE", 23),
    ("AL", 28),
    ("AT", 20),
    ("AZ", 28),
    ("BA", 20),
    ("BE", 16),
    ("BG", 22),
    ("BH", 22),
    ("BR", 29),
    ("BY", 28),
    ("CH", 21),
    ("CR", 22),
    ("CY", 28),
    ("CZ", 24),
    ("DE", 22),
    ("DK", 18),
    ("DO", 28),
    ("EE", 20),
    ("EG", 29),
    ("ES", 24),
    ("FI", 18),
    ("FO", 18),
    ("FR", 27),
    ("GB", 22),
    ("GE", 22),
    ("GI", 23),
    ("GL", 18),
    ("GR", 27),
    ("GT", 28),
    ("HR", 21),
    ("HU", 28),
    ("IE", 22),
    ("IL", 23),
    ("IQ", 23),
    ("IS", 26),
    ("IT", 27),
    ("JO", 30),
    ("KW", 30),
    ("KZ", 20),
    ("LB", 28),
    ("LC", 32),
    ("LI", 21),
    ("LT", 20),
    ("LU", 20),
    ("LV", 21),
    ("MC", 27),
    ("MD", 24),
    ("ME", 22),
    ("MK", 19),
    ("MR", 27),
    ("MT", 31),
    ("MU", 30),
    ("NL", 18),
    ("NO", 15),
    ("PK", 24),
    ("PL", 28),
    ("PS", 29),
    ("PT", 25),
    ("QA", 29),
    ("RO", 24),
    ("RS", 22),
    ("SA", 24),
    ("SC", 31),
    ("SE", 24),
    ("SI", 19),
    ("SK", 24),
    ("SM", 27),
    ("ST", 25),
    ("SV", 28),
    ("TL", 23),
    ("TN", 24),
    ("TR", 26),
    ("UA", 29),
    ("VG", 24),
    ("XK", 20),
];

fn country_code(iban: &str) -> &str {
    let mut cit = iban.char_indices();
    let mut c_end = 0;
    cit.next();
    if let Some(c) = cit.next() {
        c_end = c.0 + c.1.len_utf8();
    }
    &iban[0..c_end]
}

fn enc(c: char, buf: &mut String) -> bool {
    match c {
        '0'..='9' => buf.push(c),
        'a'..='z' => buf.push_str(format!("{}", (c as u32 - 'a' as u32) + 10).as_str()),
        'A'..='Z' => buf.push_str(format!("{}", (c as u32 - 'A' as u32) + 10).as_str()),
        ' ' => { /* noop */ }
        _ => return false,
    }
    true
}

fn valid_check_sum(iban: &str) -> bool {
    let mut cit = iban.chars();
    let Some(c0) = cit.next() else {
        return false;
    };
    let Some(c1) = cit.next() else {
        return false;
    };
    let Some(c2) = cit.next() else {
        return false;
    };
    let Some(c3) = cit.next() else {
        return false;
    };

    let mut buf = String::new();
    for c in cit {
        if !enc(c, &mut buf) {
            return false;
        }
    }
    if !enc(c0, &mut buf) {
        return false;
    }
    if !enc(c1, &mut buf) {
        return false;
    }
    if !enc(c2, &mut buf) {
        return false;
    }
    if !enc(c3, &mut buf) {
        return false;
    }
    let buf = buf.as_str();

    let mut c0 = 0;
    let mut c1 = min(buf.len(), 18);
    let mut r = 0;
    loop {
        let mut v = u64::from_str(&buf[c0..c1]).expect("integer");

        v += r * 10u64.pow(v.ilog10() + 1);
        r = v % 97;

        c0 = c1;
        c1 = min(buf.len(), c1 + 18);

        if c0 == c1 {
            break;
        }
    }

    r == 1
}

fn valid_iban_country(iban: &str) -> bool {
    let cc = country_code(iban);
    for v in IBAN {
        if v.0 == cc {
            return true;
        }
    }
    false
}

/// IBAN length derived from the country-code in
/// the first two chars of the string.
fn iban_len(iban: &str) -> Option<u8> {
    let cc = country_code(iban);
    for v in IBAN {
        if v.0 == cc {
            return Some(v.1);
        }
    }
    None
}

fn pattern(iban: &str) -> String {
    if let Some(len) = iban_len(iban) {
        let mut buf = String::new();
        buf.push_str("lldd ");
        for i in 0..len - 4 {
            if i > 0 && i % 4 == 0 {
                buf.push(' ');
            }
            buf.push('a');
        }
        buf
    } else {
        "___________________________________".to_string()
    }
}

fn to_display_format(iban: &str) -> String {
    if valid_iban_country(iban) {
        let mut buf = String::new();
        for (i, c) in iban.chars().enumerate() {
            if i > 0 && i % 4 == 0 {
                buf.push(' ');
            }
            buf.push(c);
        }
        buf
    } else {
        iban.to_string()
    }
}

fn from_display_format(cc: &str, iban: &str) -> String {
    if valid_iban_country(cc) {
        let mut buf = String::new();
        for (i, c) in iban.chars().enumerate() {
            if (i + 1) % 5 != 0 {
                buf.push(c);
            }
        }
        buf
    } else {
        iban.to_string()
    }
}
