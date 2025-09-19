#![allow(dead_code)]

use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{MiniSalsaState, run_ui, setup_logging};
use log::warn;
use rat_event::{HandleEvent, Popup, Regular, ct_event, try_flow};
use rat_focus::{Focus, FocusBuilder, HasFocus};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_text::clipboard::{Clipboard, ClipboardError, set_global_clipboard};
use rat_text::text_area::{TextArea, TextAreaState};
use rat_text::text_input::{TextInput, TextInputState};
use rat_text::text_input_mask::{MaskedInput, MaskedInputState};
use rat_widget::choice::{Choice, ChoiceState};
use rat_widget::event::{Outcome, PagerOutcome};
use rat_widget::pager::{DualPager, DualPagerState};
use rat_widget::paired::{Paired, PairedState, PairedWidget};
use rat_widget::slider::{Slider, SliderState};
use rat_widget::text::HasScreenCursor;
use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::Padding;
use std::cell::RefCell;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    set_global_clipboard(CliClipboard::default());

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
    form: DualPagerState<usize>,

    name: TextInputState,
    version: MaskedInputState,
    edition: SliderState<u16>,
    author: TextInputState,
    descr: TextAreaState,
    license: ChoiceState<String>,
    repository: TextInputState,
    readme: TextInputState,
    keywords: TextInputState,
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
            form: Default::default(),
            name: Default::default(),
            version: Default::default(),
            edition: Default::default(),
            author: Default::default(),
            descr: Default::default(),
            license: Default::default(),
            repository: Default::default(),
            readme: Default::default(),
            keywords: Default::default(),
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

fn init_input(_data: &mut Data, _istate: &mut MiniSalsaState, _state: &mut State) {}

fn repaint_input(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l1 = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);
    let l2 = Layout::horizontal([Constraint::Fill(1)]).split(l1[0]);

    // set up form
    let form = DualPager::new()
        .styles(THEME.pager_style())
        .auto_label(false);

    // maybe rebuild layout
    use rat_widget::layout::{FormLabel as L, FormWidget as W, LayoutForm};
    if !state.form.valid_layout(form.layout_size(l2[0])) {
        let mut lf = LayoutForm::new() //
            .spacing(1)
            .line_spacing(1)
            .flex(Flex::Legacy);

        lf.widget(state.name.id(), L::Str("_Name|F5"), W::Width(20));
        lf.widget(state.version.id(), L::Str("_Version"), W::Width(12));
        lf.widget(state.edition.id(), L::Str("_Edition"), W::Width(20));
        lf.widget(state.author.id(), L::Str("_Author"), W::Width(20));
        lf.widget(state.descr.id(), L::Str("_Describe"), W::StretchXY(20, 4));
        lf.widget(state.license.id(), L::Str("_License"), W::Width(18));
        lf.widget(state.repository.id(), L::Str("_Repository"), W::Width(25));
        lf.widget(state.readme.id(), L::Str("Read_me"), W::Width(20));
        lf.widget(state.keywords.id(), L::Str("_Keywords"), W::Width(25));
        lf.page_break();
        lf.widget(state.category1.id(), L::Str("_Category"), W::Width(25));
        lf.widget(state.category2.id(), L::None, W::Width(25));
        lf.widget(state.category3.id(), L::None, W::Width(25));
        lf.widget(state.category4.id(), L::None, W::Width(25));
        lf.widget(state.category5.id(), L::None, W::Width(25));

        state
            .form
            .set_layout(lf.paged(form.layout_size(l2[0]), Padding::new(2, 2, 1, 1)));
    }
    // set current layout and prepare rendering.
    let mut form = form.into_buffer(l2[0], frame.buffer_mut(), &mut state.form);

    // render the input fields.
    form.render(
        state.name.id(),
        || TextInput::new().styles(istate.theme.input_style()),
        &mut state.name,
    );
    form.render(
        state.version.id(),
        || {
            MaskedInput::new()
                .styles(istate.theme.input_style())
                .compact(true)
        },
        &mut state.version,
    );
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
    form.render(
        state.author.id(),
        || TextInput::new().styles(istate.theme.input_style()),
        &mut state.author,
    );
    form.render(
        state.descr.id(),
        || TextArea::new().styles(istate.theme.textarea_style()),
        &mut state.descr,
    );
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
    form.render(
        state.repository.id(),
        || TextInput::new().styles(istate.theme.input_style()),
        &mut state.repository,
    );
    form.render(
        state.readme.id(),
        || TextInput::new().styles(istate.theme.input_style()),
        &mut state.readme,
    );
    form.render(
        state.keywords.id(),
        || TextInput::new().styles(istate.theme.input_style()),
        &mut state.keywords,
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
        form.render(state.license.id(), || license_popup, &mut state.license);
    }

    if let Some(cursor) = state
        .name
        .screen_cursor()
        .or(state.version.screen_cursor())
        .or(state.author.screen_cursor())
        .or(state.descr.screen_cursor())
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
    frame.render_stateful_widget(menu1, l1[1], &mut state.menu);

    Ok(())
}

fn focus(state: &State) -> Focus {
    let mut fb = FocusBuilder::default();
    fb.widget(&state.menu);
    fb.widget(&state.name);
    fb.widget(&state.version);
    fb.widget(&state.edition);
    fb.widget(&state.author);
    fb.widget(&state.descr);
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

    // page navigation for the form.
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

    // regular event-handling
    try_flow!(state.name.handle(event, Regular));
    try_flow!(match event {
        ct_event!(keycode press F(5)) => {
            focus.focus(&state.name);
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });
    try_flow!(state.version.handle(event, Regular));
    try_flow!(state.edition.handle(event, Regular));
    try_flow!(state.author.handle(event, Regular));
    try_flow!(state.descr.handle(event, Regular));
    try_flow!(state.repository.handle(event, Regular));
    try_flow!(state.readme.handle(event, Regular));
    try_flow!(state.keywords.handle(event, Regular));
    try_flow!(state.category1.handle(event, Regular));
    try_flow!(state.category2.handle(event, Regular));
    try_flow!(state.category3.handle(event, Regular));
    try_flow!(state.category4.handle(event, Regular));
    try_flow!(state.category5.handle(event, Regular));

    Ok(Outcome::Continue)
}

#[derive(Debug, Default, Clone)]
struct CliClipboard {
    clip: RefCell<String>,
}

impl Clipboard for CliClipboard {
    fn get_string(&self) -> Result<String, ClipboardError> {
        match cli_clipboard::get_contents() {
            Ok(v) => Ok(v),
            Err(e) => {
                warn!("{:?}", e);
                Ok(self.clip.borrow().clone())
            }
        }
    }

    fn set_string(&self, s: &str) -> Result<(), ClipboardError> {
        let mut clip = self.clip.borrow_mut();
        *clip = s.to_string();

        match cli_clipboard::set_contents(s.to_string()) {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("{:?}", e);
                Err(ClipboardError)
            }
        }
    }
}
