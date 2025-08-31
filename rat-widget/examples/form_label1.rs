#![allow(dead_code)]

use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use rat_event::{ct_event, try_flow, HandleEvent, Popup, Regular};
use rat_focus::{Focus, FocusBuilder, HasFocus};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_text::text_area::{TextArea, TextAreaState};
use rat_text::text_input::{TextInput, TextInputState};
use rat_text::text_input_mask::{MaskedInput, MaskedInputState};
use rat_widget::caption::{CaptionState, CaptionStyle, HotkeyAlignment, HotkeyPolicy, WithFocus};
use rat_widget::choice::{Choice, ChoiceState};
use rat_widget::event::{Outcome, PagerOutcome};
use rat_widget::layout::{FormLabel, FormWidget, LayoutForm};
use rat_widget::pager::{DualPager, DualPagerState};
use rat_widget::paired::{Paired, PairedState, PairedWidget};
use rat_widget::slider::{Slider, SliderState};
use rat_widget::text::HasScreenCursor;
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::Padding;
use ratatui::Frame;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State::default();
    state.edition.set_value(2024);
    state.menu.focus.set(true);

    run_ui(
        "label_and_form",
        init_input,
        handle_input,
        repaint_input,
        &mut data,
        &mut state,
    )
}

struct Data {}

struct State {
    caption_style: CaptionStyle,

    form: DualPagerState<usize>,

    label_name: CaptionState,
    name: TextInputState,
    label_version: CaptionState,
    version: MaskedInputState,
    label_edition: CaptionState,
    edition: SliderState<u16>,
    label_author: CaptionState,
    author: TextInputState,
    label_description: CaptionState,
    description: TextAreaState,
    label_license: CaptionState,
    license: ChoiceState<String>,
    label_repository: CaptionState,
    repository: TextInputState,
    label_readme: CaptionState,
    readme: TextInputState,
    label_keywords: CaptionState,
    keywords: TextInputState,
    label_categories: CaptionState,
    category1: TextInputState,
    category2: TextInputState,
    category3: TextInputState,
    category4: TextInputState,
    category5: TextInputState,
    menu: MenuLineState,
}

impl Default for State {
    fn default() -> Self {
        let mut s = Self {
            caption_style: Default::default(),
            form: Default::default(),
            label_name: Default::default(),
            name: Default::default(),
            label_version: Default::default(),
            version: Default::default(),
            label_edition: Default::default(),
            edition: Default::default(),
            label_author: Default::default(),
            author: Default::default(),
            label_description: Default::default(),
            description: Default::default(),
            label_license: Default::default(),
            license: Default::default(),
            label_repository: Default::default(),
            repository: Default::default(),
            label_readme: Default::default(),
            readme: Default::default(),
            label_keywords: Default::default(),
            keywords: Default::default(),
            label_categories: Default::default(),
            category1: Default::default(),
            category2: Default::default(),
            category3: Default::default(),
            category4: Default::default(),
            category5: Default::default(),
            menu: Default::default(),
        };

        s.version.set_mask("##0\\.##0\\.##0").expect("ok");

        s
    }
}

fn init_input(_data: &mut Data, istate: &mut MiniSalsaState, state: &mut State) {
    state.caption_style = istate.theme.caption_style();
}

fn repaint_input(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l1 = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);

    let l2 = Layout::horizontal([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(3),
    ])
    .split(l1[1]);

    // set up form
    let form = DualPager::new()
        .styles(THEME.pager_style())
        .caption_style(state.caption_style.clone())
        .auto_label(false);

    // maybe rebuild layout
    let layout_size = form.layout_size(l2[1]);
    if !state.form.valid_layout(layout_size) {
        let mut form = LayoutForm::new() //
            .spacing(1)
            .line_spacing(1)
            .flex(Flex::Legacy);

        form.widget(
            state.name.id(),
            FormLabel::Str("_Name|F5"),
            FormWidget::Width(20),
        );
        form.widget(
            state.version.id(),
            FormLabel::Str("_Version"),
            FormWidget::Width(12),
        );
        form.widget(
            state.edition.id(),
            FormLabel::Str("_Edition"),
            FormWidget::Width(32),
        );
        form.widget(
            state.author.id(),
            FormLabel::Str("_Author"),
            FormWidget::Width(20),
        );
        form.widget(
            state.description.id(),
            FormLabel::Str("_Description"),
            FormWidget::StretchXY(30, 4),
        );
        form.widget(
            state.license.id(),
            FormLabel::Str("_License"),
            FormWidget::Width(18),
        );
        form.widget(
            state.repository.id(),
            FormLabel::Str("_Repository"),
            FormWidget::Width(35),
        );
        form.widget(
            state.readme.id(),
            FormLabel::Str("_Readme"),
            FormWidget::Width(20),
        );
        form.widget(
            state.keywords.id(),
            FormLabel::Str("_Keywords"),
            FormWidget::Width(25),
        );
        form.widget(
            state.category1.id(),
            FormLabel::Str("_Category"),
            FormWidget::Width(25),
        );
        form.widget(
            state.category2.id(),
            FormLabel::Str(""),
            FormWidget::Width(25),
        );
        form.widget(
            state.category3.id(),
            FormLabel::Str(""),
            FormWidget::Width(25),
        );
        form.widget(
            state.category4.id(),
            FormLabel::Str(""),
            FormWidget::Width(25),
        );
        form.widget(
            state.category5.id(),
            FormLabel::Str(""),
            FormWidget::Width(25),
        );

        state
            .form
            .set_layout(form.paged(layout_size, Padding::new(2, 2, 1, 1)));
    }

    // set current layout and prepare rendering.
    let mut form = form.into_buffer(l2[1], frame.buffer_mut(), &mut state.form);

    // render the input fields.
    form.render_caption(state.name.id(), &state.name, &mut state.label_name);
    form.render(
        state.name.id(),
        || TextInput::new().styles(istate.theme.input_style()),
        &mut state.name,
    );
    form.render_caption(state.version.id(), &state.version, &mut state.label_version);
    form.render(
        state.version.id(),
        || {
            MaskedInput::new()
                .styles(istate.theme.input_style())
                .compact(true)
        },
        &mut state.version,
    );
    form.render_caption(state.edition.id(), &state.edition, &mut state.label_edition);
    let value = format!("{}", state.edition.value());
    form.render(
        state.edition.id(),
        || {
            Paired::new(
                Slider::new()
                    .styles(istate.theme.slider_style())
                    .range((2015, 2024))
                    .step(3),
                PairedWidget::new(Line::from(value)),
            )
        },
        &mut PairedState::new(&mut state.edition, &mut ()),
    );
    form.render_caption(state.author.id(), &state.author, &mut state.label_author);
    form.render(
        state.author.id(),
        || TextInput::new().styles(istate.theme.input_style()),
        &mut state.author,
    );
    form.render_caption(
        state.description.id(),
        &state.description,
        &mut state.label_description,
    );
    form.render(
        state.description.id(),
        || TextArea::new().styles(istate.theme.textarea_style()),
        &mut state.description,
    );
    form.render_caption(state.license.id(), &state.license, &mut state.label_license);
    let license_popup = form.render2(
        state.license.id(),
        || {
            Choice::new()
                .styles(istate.theme.choice_style())
                .items([(String::from("MIT/Apache-2.0"), "MIT/Apache-2.0")])
                .into_widgets()
        },
        &mut state.license,
    );
    form.render_caption(
        state.repository.id(),
        &state.repository,
        &mut state.label_repository,
    );
    form.render(
        state.repository.id(),
        || TextInput::new().styles(istate.theme.input_style()),
        &mut state.repository,
    );
    form.render_caption(state.readme.id(), &state.readme, &mut state.label_readme);
    form.render(
        state.readme.id(),
        || TextInput::new().styles(istate.theme.input_style()),
        &mut state.readme,
    );
    form.render_caption(
        state.keywords.id(),
        &state.keywords,
        &mut state.label_keywords,
    );
    form.render(
        state.keywords.id(),
        || TextInput::new().styles(istate.theme.input_style()),
        &mut state.keywords,
    );
    form.render_caption(
        state.category1.id(),
        &state.category1,
        &mut state.label_categories,
    );
    form.render(
        state.category1.id(),
        || TextInput::new().styles(istate.theme.input_style()),
        &mut state.category1,
    );
    form.render(
        state.category2.id(),
        || TextInput::new().styles(istate.theme.input_style()),
        &mut state.category2,
    );
    form.render(
        state.category3.id(),
        || TextInput::new().styles(istate.theme.input_style()),
        &mut state.category3,
    );
    form.render(
        state.category4.id(),
        || TextInput::new().styles(istate.theme.input_style()),
        &mut state.category4,
    );
    form.render(
        state.category5.id(),
        || TextInput::new().styles(istate.theme.input_style()),
        &mut state.category5,
    );

    // popups
    if let Some(license_popup) = license_popup {
        form.render(5, || license_popup, &mut state.license);
    }

    if let Some(cursor) = state
        .name
        .screen_cursor()
        .or(state.version.screen_cursor())
        .or(state.author.screen_cursor())
        .or(state.description.screen_cursor())
        .or(state.repository.screen_cursor())
        .or(state.readme.screen_cursor())
        .or(state.keywords.screen_cursor())
        .or(state.category1.screen_cursor())
        .or(state.category2.screen_cursor())
        .or(state.category3.screen_cursor())
        .or(state.category4.screen_cursor())
        .or(state.category5.screen_cursor())
    {
        frame.set_cursor_position(cursor);
    }

    let menu1 = MenuLine::new()
        .title("#.#")
        .item_parsed("_Quit")
        .styles(THEME.menu_style());
    frame.render_stateful_widget(menu1, l1[3], &mut state.menu);

    Ok(())
}

fn focus(state: &State) -> Focus {
    let mut fb = FocusBuilder::default();
    fb.widget(&state.menu);
    fb.widget(&state.name);
    fb.widget(&state.version);
    fb.widget(&state.edition);
    fb.widget(&state.author);
    fb.widget(&state.description);
    fb.widget(&state.license);
    fb.widget(&state.repository);
    fb.widget(&state.readme);
    fb.widget(&state.keywords);
    fb.widget(&state.category1);
    fb.widget(&state.category2);
    fb.widget(&state.category3);
    fb.widget(&state.category4);
    fb.widget(&state.category5);
    fb.build()
}

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let mut focus = focus(state);

    istate.focus_outcome = focus.handle(event, Regular);

    try_flow!(match state.menu.handle(event, Regular) {
        MenuOutcome::Activated(0) => {
            istate.quit = true;
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    try_flow!(match event {
        ct_event!(keycode press F(1)) => {
            state.caption_style.hotkey_policy = match state.caption_style.hotkey_policy {
                Some(HotkeyPolicy::Always) => Some(HotkeyPolicy::OnHover),
                Some(HotkeyPolicy::OnHover) => Some(HotkeyPolicy::WhenFocused),
                Some(HotkeyPolicy::WhenFocused) => Some(HotkeyPolicy::Always),
                None => Some(HotkeyPolicy::Always),
            };
            Outcome::Changed
        }
        ct_event!(keycode press F(2)) => {
            state.caption_style.hotkey_align = match state.caption_style.hotkey_align {
                Some(HotkeyAlignment::LabelHotkey) => Some(HotkeyAlignment::HotkeyLabel),
                Some(HotkeyAlignment::HotkeyLabel) => Some(HotkeyAlignment::LabelHotkey),
                None => Some(HotkeyAlignment::LabelHotkey),
            };
            Outcome::Changed
        }
        ct_event!(keycode press F(3)) => {
            state.caption_style.align = match state.caption_style.align {
                Some(Alignment::Left) => Some(Alignment::Center),
                Some(Alignment::Center) => Some(Alignment::Right),
                Some(Alignment::Right) => Some(Alignment::Left),
                None => Some(Alignment::Left),
            };
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    try_flow!(match state.form.handle(event, Regular) {
        PagerOutcome::Page(n) => {
            if let Some(first) = state.form.first(n) {
                focus.by_widget_id(first);
            } else {
                unreachable!();
            }
            Outcome::Changed
        }
        r => {
            if let Some(focused) = focus.focused() {
                state.form.show(focused.widget_id());
            }
            r.into()
        }
    });

    // popups first
    try_flow!(state.license.handle(event, Popup));

    try_flow!(state.label_name.handle(event, WithFocus(&focus)));
    try_flow!(state.name.handle(event, Regular));
    try_flow!(match event {
        ct_event!(keycode press F(5)) => {
            focus.focus(&state.name);
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });
    try_flow!(state.label_version.handle(event, WithFocus(&focus)));
    try_flow!(state.version.handle(event, Regular));
    try_flow!(state.label_edition.handle(event, WithFocus(&focus)));
    try_flow!(state.edition.handle(event, Regular));
    try_flow!(state.label_author.handle(event, WithFocus(&focus)));
    try_flow!(state.author.handle(event, Regular));
    try_flow!(state.label_description.handle(event, WithFocus(&focus)));
    try_flow!(state.description.handle(event, Regular));
    try_flow!(match event {
        ct_event!(keycode press Esc) => {
            if state.description.is_focused() {
                focus.next_force();
                Outcome::Changed
            } else {
                Outcome::Continue
            }
        }
        _ => Outcome::Continue,
    });
    try_flow!(state.label_license.handle(event, WithFocus(&focus)));
    try_flow!(state.label_repository.handle(event, WithFocus(&focus)));
    try_flow!(state.repository.handle(event, Regular));
    try_flow!(state.label_readme.handle(event, WithFocus(&focus)));
    try_flow!(state.readme.handle(event, Regular));
    try_flow!(state.label_keywords.handle(event, WithFocus(&focus)));
    try_flow!(state.keywords.handle(event, Regular));
    try_flow!(state.label_categories.handle(event, WithFocus(&focus)));
    try_flow!(state.category1.handle(event, Regular));
    try_flow!(state.category2.handle(event, Regular));
    try_flow!(state.category3.handle(event, Regular));
    try_flow!(state.category4.handle(event, Regular));
    try_flow!(state.category5.handle(event, Regular));

    Ok(Outcome::Continue)
}
