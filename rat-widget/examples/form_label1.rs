#![allow(dead_code)]

use crate::mini_salsa::{MiniSalsaState, run_ui, setup_logging};
use log::{debug, warn};
use rat_event::{HandleEvent, Popup, Regular, ct_event, try_flow};
use rat_focus::{Focus, FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_text::clipboard::{Clipboard, ClipboardError, set_global_clipboard};
use rat_text::impl_screen_cursor;
use rat_text::text_area::{TextArea, TextAreaState, TextWrap};
use rat_text::text_input::{TextInput, TextInputState};
use rat_text::text_input_mask::{MaskedInput, MaskedInputState};
use rat_widget::choice::{Choice, ChoiceState};
use rat_widget::event::{FormOutcome, Outcome};
use rat_widget::form::{Form, FormState};
use rat_widget::layout::LayoutForm;
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
    state.descr.set_text_wrap(TextWrap::Word(4));
    state.menu.focus.set(true);

    run_ui("label_and_form", init, event, render, &mut data, &mut state)
}

struct Data {}

struct State {
    flex: Flex,
    columns: u8,
    line_spacing: u16,
    form: FormState<usize>,

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
            flex: Default::default(),
            columns: 1,
            line_spacing: 1,
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

fn init(_data: &mut Data, _istate: &mut MiniSalsaState, _state: &mut State) {}

fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l1 = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);
    let l2 = Layout::horizontal([Constraint::Fill(1)]).split(l1[0]);

    // set up form
    let form = Form::new() //
        .styles(istate.theme.form_style());

    // maybe rebuild layout

    if !state.form.valid_layout(form.layout_size(l2[0])) {
        use rat_widget::layout::{FormLabel as L, FormWidget as W};

        let mut lf = LayoutForm::new() //
            .border(Padding::new(2, 2, 1, 1))
            .line_spacing(state.line_spacing)
            .columns(state.columns)
            .flex(state.flex);

        lf.widgets([
            (state.name.id(), L::Str("Name"), W::Width(20)),
            (state.version.id(), L::Str("Version"), W::Width(14)),
            (state.edition.id(), L::Str("Edition"), W::Width(20)),
            (state.author.id(), L::Str("Author"), W::Width(20)),
            (state.descr.id(), L::Str("Describe"), W::StretchXY(20, 4)),
            (state.license.id(), L::Str("License"), W::Width(20)),
            (state.repository.id(), L::Str("Repository"), W::Width(25)),
            (state.readme.id(), L::Str("Readme"), W::Width(20)),
            (state.keywords.id(), L::Str("Keywords"), W::Width(25)),
        ]);
        lf.page_break();
        lf.widgets([
            (state.category1.id(), L::Str("Category"), W::Width(25)),
            (state.category2.id(), L::None, W::Width(25)),
            (state.category3.id(), L::None, W::Width(25)),
            (state.category4.id(), L::None, W::Width(25)),
            (state.category5.id(), L::None, W::Width(25)),
        ]);

        state
            .form
            .set_layout(lf.build_paged(form.layout_size(l2[0])));
    }
    // set current layout and prepare rendering.
    let mut form = form.into_buffer(l2[0], frame.buffer_mut(), &mut state.form);

    // render the input fields.
    form.render(
        state.name.id(),
        || TextInput::new().styles(istate.theme.text_style()),
        &mut state.name,
    );
    form.render(
        state.version.id(),
        || {
            MaskedInput::new()
                .styles(istate.theme.text_style())
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
        || TextInput::new().styles(istate.theme.text_style()),
        &mut state.author,
    );
    form.render(
        state.descr.id(),
        || TextArea::new().styles(istate.theme.text_style()),
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
        || TextInput::new().styles(istate.theme.text_style()),
        &mut state.repository,
    );
    form.render(
        state.readme.id(),
        || TextInput::new().styles(istate.theme.text_style()),
        &mut state.readme,
    );
    form.render(
        state.keywords.id(),
        || TextInput::new().styles(istate.theme.text_style()),
        &mut state.keywords,
    );
    form.render(
        state.category1.id(),
        || TextInput::new().styles(istate.theme.text_style()),
        &mut state.category1,
    );
    form.render(
        state.category2.id(),
        || TextInput::new().styles(istate.theme.text_style()),
        &mut state.category2,
    );
    form.render(
        state.category3.id(),
        || TextInput::new().styles(istate.theme.text_style()),
        &mut state.category3,
    );
    form.render(
        state.category4.id(),
        || TextInput::new().styles(istate.theme.text_style()),
        &mut state.category4,
    );
    form.render(
        state.category5.id(),
        || TextInput::new().styles(istate.theme.text_style()),
        &mut state.category5,
    );

    // popups
    form.render_opt(state.license.id(), || license_popup, &mut state.license);

    if let Some(cursor) = state.screen_cursor() {
        frame.set_cursor_position(cursor);
    }

    let menu1 = MenuLine::new()
        .title("#.#")
        .item_parsed("_Flex|F2")
        .item_parsed("_Spacing|F3")
        .item_parsed("_Columns|F4")
        .item_parsed("_Next|F8")
        .item_parsed("_Prev|F9")
        .item_parsed("_Quit")
        .styles(istate.theme.menu_style());
    frame.render_stateful_widget(menu1, l1[1], &mut state.menu);

    Ok(())
}

impl_screen_cursor!(name, version, author, descr, repository, readme,
    keywords, category1, category2, category3, category4, category5 for State);

impl HasFocus for State {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget_navigate(&self.menu, Navigation::Regular);
        builder.widget(&self.name);
        builder.widget(&self.version);
        builder.widget(&self.edition);
        builder.widget(&self.author);
        builder.widget_navigate(&self.descr, Navigation::Regular);
        builder.widget(&self.license);
        builder.widget(&self.repository);
        builder.widget(&self.readme);
        builder.widget(&self.keywords);
        builder.widget(&self.category1);
        builder.widget(&self.category2);
        builder.widget(&self.category3);
        builder.widget(&self.category4);
        builder.widget(&self.category5);
    }

    fn focus(&self) -> FocusFlag {
        unimplemented!("not defined")
    }

    fn area(&self) -> Rect {
        unimplemented!("not defined")
    }
}

fn focus(state: &State) -> Focus {
    FocusBuilder::build_for(state)
}

fn event(
    event: &crossterm::event::Event,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let mut focus = focus(state);

    istate.focus_outcome = focus.handle(event, Regular);
    if istate.focus_outcome == Outcome::Changed {
        state.form.show_focused(&focus);
    }

    try_flow!(match event {
        ct_event!(keycode press F(1)) => {
            debug!("{:#?}", state.form.layout);
            Outcome::Unchanged
        }
        ct_event!(keycode press F(2)) => flip_flex(state),
        ct_event!(keycode press F(3)) => flip_spacing(state),
        ct_event!(keycode press F(4)) => flip_columns(state),
        ct_event!(keycode press F(8)) => prev_page(state, &focus),
        ct_event!(keycode press F(9)) => next_page(state, &focus),
        _ => Outcome::Continue,
    });

    try_flow!(match state.menu.handle(event, Regular) {
        MenuOutcome::Activated(0) => flip_flex(state),
        MenuOutcome::Activated(1) => flip_spacing(state),
        MenuOutcome::Activated(2) => flip_columns(state),
        MenuOutcome::Activated(3) => next_page(state, &focus),
        MenuOutcome::Activated(4) => prev_page(state, &focus),
        MenuOutcome::Activated(5) => {
            istate.quit = true;
            Outcome::Changed
        }
        r => r.into(),
    });

    // popups first
    try_flow!(state.license.handle(event, Popup));

    // regular event-handling
    try_flow!(state.name.handle(event, Regular));
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

    // page navigation for the form.
    try_flow!(match state.form.handle(event, Regular) {
        FormOutcome::Page => {
            state.form.focus_first(&focus);
            Outcome::Changed
        }
        r => r.into(),
    });

    try_flow!(match event {
        ct_event!(keycode press Esc) => {
            if state.menu.is_focused() {
                state.form.focus_first(&focus);
            } else {
                focus.focus(&state.menu);
            }
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    Ok(Outcome::Continue)
}

fn flip_flex(state: &mut State) -> Outcome {
    state.form.clear();
    state.flex = match state.flex {
        Flex::Legacy => Flex::Start,
        Flex::Start => Flex::End,
        Flex::End => Flex::Center,
        Flex::Center => Flex::SpaceBetween,
        Flex::SpaceBetween => Flex::SpaceAround,
        Flex::SpaceAround => Flex::Legacy,
    };
    Outcome::Changed
}

fn flip_spacing(state: &mut State) -> Outcome {
    state.form.clear();
    state.line_spacing = match state.line_spacing {
        0 => 1,
        1 => 2,
        2 => 3,
        _ => 0,
    };
    Outcome::Changed
}

fn flip_columns(state: &mut State) -> Outcome {
    state.form.clear();
    state.columns = match state.columns {
        1 => 2,
        2 => 3,
        3 => 4,
        4 => 5,
        _ => 1,
    };
    Outcome::Changed
}

fn prev_page(state: &mut State, focus: &Focus) -> Outcome {
    if state.form.prev_page() {
        if let Some(widget) = state.form.first(state.form.page()) {
            focus.by_widget_id(widget);
        }
        Outcome::Changed
    } else {
        Outcome::Unchanged
    }
}

fn next_page(state: &mut State, focus: &Focus) -> Outcome {
    if state.form.next_page() {
        if let Some(widget) = state.form.first(state.form.page()) {
            focus.by_widget_id(widget);
        }
        Outcome::Changed
    } else {
        Outcome::Unchanged
    }
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
