use crate::Global;
use anyhow::Error;
use pure_rust_locales::{Locale, locale_match};
use rat_theme4::{StyleName, WidgetStyle};
use rat_widget::button::{Button, ButtonState};
use rat_widget::calendar::selection::SingleSelection;
use rat_widget::calendar::{CalendarState, Month};
use rat_widget::checkbox::{Checkbox, CheckboxState};
use rat_widget::choice::{Choice, ChoiceState};
use rat_widget::clipper::{Clipper, ClipperState};
use rat_widget::combobox::{Combobox, ComboboxState};
use rat_widget::date_input::{DateInput, DateInputState};
use rat_widget::event::{
    ButtonOutcome, ChoiceOutcome, HandleEvent, Outcome, Popup, Regular, event_flow,
};
use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_widget::layout::LayoutForm;
use rat_widget::number_input::{NumberInput, NumberInputState};
use rat_widget::paired::{PairSplit, Paired, PairedState, PairedWidget};
use rat_widget::radio::{Radio, RadioLayout, RadioState};
use rat_widget::scrolled::Scroll;
use rat_widget::slider::{Slider, SliderState};
use rat_widget::text::{HasScreenCursor, TextFocusLost};
use rat_widget::text_input::{TextInput, TextInputState};
use rat_widget::textarea::{TextArea, TextAreaState};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Direction, Flex, Rect};
use ratatui_core::style::Style;
use ratatui_core::symbols::border;
use ratatui_core::text::Line;
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::block::{Block, Padding};
use ratatui_widgets::borders::Borders;

// mark
#[derive(Debug)]
pub struct SampleDataInput {
    pub form: ClipperState,

    pub disabled: ButtonState,
    pub button: ButtonState,
    pub checkbox: CheckboxState,
    pub choice: ChoiceState,
    pub combobox: ComboboxState,
    pub date_input: DateInputState,
    pub number_input: NumberInputState,
    pub number_invalid: NumberInputState,
    pub radio: RadioState,
    pub slider: SliderState<usize>,
    pub text: TextInputState,
    pub textarea: TextAreaState,
    pub calendar: CalendarState<1, SingleSelection>,
}

impl HasFocus for SampleDataInput {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget(&self.disabled);
        builder.widget(&self.button);
        builder.widget(&self.checkbox);
        builder.widget(&self.choice);
        builder.widget(&self.combobox);
        builder.widget(&self.date_input);
        builder.widget(&self.number_input);
        builder.widget(&self.number_invalid);
        builder.widget(&self.radio);
        builder.widget(&self.slider);
        builder.widget(&self.text);
        builder.widget_navigate(&self.textarea, Navigation::Regular);
        builder.widget(&self.calendar);
    }

    fn focus(&self) -> FocusFlag {
        unimplemented!("no available")
    }

    fn area(&self) -> Rect {
        unimplemented!("no available")
    }
}

impl HasScreenCursor for SampleDataInput {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        self.combobox
            .screen_cursor()
            .or(self.date_input.screen_cursor())
            .or(self.number_input.screen_cursor())
            .or(self.number_invalid.screen_cursor())
            .or(self.text.screen_cursor())
            .or(self.textarea.screen_cursor())
            .or(self.calendar.screen_cursor())
    }
}

impl SampleDataInput {
    pub fn new(loc: Locale) -> Self {
        let mut z = Self {
            form: ClipperState::named("show"),
            disabled: ButtonState::named("disabled"),
            button: ButtonState::named("button"),
            checkbox: CheckboxState::named("checkbox"),
            choice: ChoiceState::named("choice"),
            combobox: ComboboxState::named("combobox"),
            date_input: DateInputState::named("date_input"),
            number_input: NumberInputState::named("number_input"),
            number_invalid: NumberInputState::named("number_invalid"),
            radio: RadioState::named("radio"),
            slider: SliderState::<usize>::named("slider"),
            text: TextInputState::named("text"),
            textarea: TextAreaState::named("textarea"),
            calendar: CalendarState::named("calendar"),
        };

        let fmt = locale_match!(loc => LC_TIME::D_FMT);
        z.date_input.set_format_loc(fmt, loc).expect("date_format");
        z.number_input
            .set_format_loc("###,##0.00#", loc)
            .expect("number_format");
        z.number_invalid
            .set_format_loc("###,##0.00#", loc)
            .expect("number_format");
        z.number_invalid.set_invalid(true);
        z.calendar.move_to_today();
        z.text.set_text("text text text");
        z.text.set_selection(0, 4);
        z.textarea.set_text("sample sample\nsample\nsample sample");
        z.textarea.set_selection((0, 1), (6, 1));
        z
    }
}

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut SampleDataInput,
    ctx: &mut Global,
) -> Result<(), Error> {
    let mut form = Clipper::new() //
        .vscroll(Scroll::new())
        .buffer_uses_view_size()
        .styles(ctx.show_theme.style(WidgetStyle::CLIPPER));

    let layout_size = form.layout_size(area, &mut state.form);

    if !state.form.valid_layout(layout_size) {
        use rat_widget::layout::{FormLabel as L, FormWidget as W};
        let mut layout = LayoutForm::<usize>::new()
            .spacing(1)
            .line_spacing(1)
            .padding(Padding::new(1, 1, 1, 1))
            .flex(Flex::Start);
        layout.widget(state.disabled.id(), L::Str("Disabled"), W::Width(11));
        layout.widget(state.button.id(), L::Str("Button"), W::Width(11));
        layout.widget(state.checkbox.id(), L::Str("Checkbox"), W::Width(14));
        layout.widget(state.choice.id(), L::Str("Choice"), W::Width(14));
        layout.widget(state.combobox.id(), L::Str("Combobox"), W::Width(14));
        layout.widget(state.date_input.id(), L::Str("DateInput"), W::Width(11));
        layout.widget(state.number_input.id(), L::Str("NumberInput"), W::Width(11));
        layout.widget(state.number_invalid.id(), L::Str("Invalid"), W::Width(11));
        layout.widget(state.radio.id(), L::Str("Radio"), W::Width(25));
        layout.widget(state.slider.id(), L::Str("Slider"), W::Width(15));
        layout.widget(state.text.id(), L::Str("TextInput"), W::Width(20));
        layout.widget(state.textarea.id(), L::Str("TextArea"), W::Size(25, 5));
        layout.widget(state.calendar.id(), L::Str("Calendar"), W::Size(25, 8));
        form = form.layout(layout.build_endless(layout_size.width));
    }
    let mut form = form.into_buffer(area, &mut state.form);

    form.render(
        state.disabled.id(),
        || {
            Button::new("Disabled")
                .styles(ctx.show_theme.style(WidgetStyle::BUTTON))
                .style(ctx.show_theme.style_style(Style::DISABLED))
        },
        &mut state.disabled,
    );
    form.render(
        state.button.id(),
        || Button::new("Ok").styles(ctx.show_theme.style(WidgetStyle::BUTTON)),
        &mut state.button,
    );
    form.render(
        state.checkbox.id(),
        || {
            Checkbox::new()
                .text("rat-salsa")
                .styles(ctx.show_theme.style(WidgetStyle::CHECKBOX))
        },
        &mut state.checkbox,
    );
    let choice_popup = form.render2(
        state.choice.id(),
        || {
            Choice::new()
                .items([
                    (0, "Zero"),
                    (1, "One"),
                    (2, "Two"),
                    (3, "Three"),
                    (4, "Four"),
                ])
                // .popup_placement(Placement::Right)
                .styles(ctx.show_theme.style(WidgetStyle::CHOICE))
                .into_widgets()
        },
        &mut state.choice,
    );
    let combo_popup = form.render2(
        state.combobox.id(),
        || {
            Combobox::new()
                .items([
                    ("Α".to_string(), "Alpha"),
                    ("Β".to_string(), "Beta"),
                    ("Γ".to_string(), "Gamma"),
                    ("Δ".to_string(), "Delta"),
                    ("Ε".to_string(), "Epsilon"),
                    ("Η".to_string(), "Eta"),
                    ("Θ".to_string(), "Theta"),
                    ("Ι".to_string(), "Iota"),
                    ("Κ".to_string(), "Kappa"),
                    ("Λ".to_string(), "Lambda"),
                    ("Μ".to_string(), "My"),
                    ("Ν".to_string(), "Ny"),
                    ("Ξ".to_string(), "Xi"),
                    ("Ο".to_string(), "Omikron"),
                    ("Π".to_string(), "Pi"),
                    ("Χ".to_string(), "Chi"),
                    ("Ψ".to_string(), "Psi"),
                    ("Ω".to_string(), "Omega"),
                ])
                .popup_scroll(Scroll::new())
                .popup_len(7)
                .styles(ctx.show_theme.style(WidgetStyle::COMBOBOX))
                .into_widgets()
        },
        &mut state.combobox,
    );
    form.render(
        state.date_input.id(),
        || {
            DateInput::new()
                .on_focus_lost(TextFocusLost::Position0)
                .styles(ctx.show_theme.style(WidgetStyle::TEXT))
        },
        &mut state.date_input,
    );
    form.render(
        state.number_input.id(),
        || NumberInput::new().styles(ctx.show_theme.style(WidgetStyle::TEXT)),
        &mut state.number_input,
    );
    form.render(
        state.number_invalid.id(),
        || NumberInput::new().styles(ctx.show_theme.style(WidgetStyle::TEXT)),
        &mut state.number_invalid,
    );
    form.render(
        state.radio.id(),
        || {
            Radio::new()
                .direction(Direction::Horizontal)
                .layout(RadioLayout::Stacked)
                .items([(0, "abc"), (1, "def"), (2, "ghi"), (3, "jkl")])
                .styles(ctx.show_theme.style(WidgetStyle::RADIO))
        },
        &mut state.radio,
    );

    let val = format!("{}", state.slider.value());
    form.render(
        state.slider.id(),
        || {
            Paired::new(
                Slider::new()
                    .range((0, 25))
                    .long_step(4)
                    .styles(ctx.show_theme.style(WidgetStyle::SLIDER)),
                PairedWidget::new(Line::from(val)),
            )
            .split(PairSplit::Constrain(
                Constraint::Fill(1),
                Constraint::Length(3),
            ))
        },
        &mut PairedState::new(&mut state.slider, &mut ()),
    );
    form.render(
        state.text.id(),
        || TextInput::new().styles(ctx.show_theme.style(WidgetStyle::TEXT)),
        &mut state.text,
    );
    let text_area_focused = state.textarea.is_focused();
    form.render(
        state.textarea.id(),
        || {
            TextArea::new()
                .vscroll(Scroll::new())
                .styles(ctx.show_theme.style(WidgetStyle::TEXTAREA))
                .block(if text_area_focused {
                    Block::new()
                        .style(ctx.show_theme.style_style(Style::INPUT))
                        .border_style(ctx.show_theme.style_style(Style::FOCUS))
                        .borders(Borders::LEFT)
                        .border_set(border::EMPTY)
                } else {
                    Block::default().style(ctx.show_theme.style_style(Style::INPUT))
                })
        },
        &mut state.textarea,
    );
    form.render(
        state.calendar.id(),
        || {
            Month::new()
                .locale(ctx.cfg.loc)
                .styles(ctx.show_theme.style(WidgetStyle::MONTH))
        },
        &mut state.calendar.months[0],
    );

    form.render_popup(state.choice.id(), || choice_popup, &mut state.choice);
    form.render_popup(state.combobox.id(), || combo_popup, &mut state.combobox);
    form.finish(buf, &mut state.form);
    Ok(())
}

pub fn event(
    event: &Event,
    state: &mut SampleDataInput,
    ctx: &mut Global,
) -> Result<Outcome, Error> {
    event_flow!(match state.choice.handle(event, Popup) {
        ChoiceOutcome::Changed => {
            ChoiceOutcome::Changed
        }
        ChoiceOutcome::Value => {
            ChoiceOutcome::Value
        }
        r => r,
    });
    event_flow!(state.combobox.handle(event, Popup));

    event_flow!(match state.button.handle(event, Regular) {
        ButtonOutcome::Pressed => {
            ctx.status = "!!OK!!".to_string();
            Outcome::Changed
        }
        r => r.into(),
    });
    event_flow!(state.checkbox.handle(event, Regular));
    event_flow!(state.date_input.handle(event, Regular));
    event_flow!(state.number_input.handle(event, Regular));
    event_flow!(state.number_invalid.handle(event, Regular));
    event_flow!(state.radio.handle(event, Regular));
    event_flow!(state.slider.handle(event, Regular));
    event_flow!(state.text.handle(event, Regular));
    event_flow!(state.textarea.handle(event, Regular));
    event_flow!(state.calendar.handle(event, Regular));

    event_flow!(state.form.handle(event, Regular));

    Ok(Outcome::Continue)
}
