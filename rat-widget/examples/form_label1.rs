#![allow(dead_code)]

use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use rat_event::{try_flow, HandleEvent, Regular};
use rat_focus::{Focus, FocusBuilder};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_text::text_area::TextAreaState;
use rat_text::text_input::TextInputState;
use rat_text::text_input_mask::MaskedInputState;
use rat_widget::choice::ChoiceState;
use rat_widget::event::Outcome;
use rat_widget::label::{Label, LabelState};
use rat_widget::layout::{FormLabel, FormWidget, LayoutForm};
use rat_widget::pager::{Form, FormState};
use rat_widget::slider::SliderState;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::widgets::{Padding, StatefulWidget};
use ratatui::Frame;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        form: FormState::default(),
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
    state.menu.focus.set(true);

    run_ui(
        "label_and_form",
        |_| {},
        handle_input,
        repaint_input,
        &mut data,
        &mut state,
    )
}

struct Data {}

struct State {
    form: FormState<u16>,

    label_name: LabelState,
    name: TextInputState,
    label_version: LabelState,
    version: MaskedInputState,
    label_edition: LabelState,
    edition: SliderState<u16>,
    label_author: LabelState,
    author: TextInputState,
    label_description: LabelState,
    description: TextAreaState,
    label_license: LabelState,
    license: ChoiceState,
    label_repository: LabelState,
    repository: TextInputState,
    label_readme: LabelState,
    readme: TextInputState,
    label_keywords: LabelState,
    keywords: TextInputState,
    label_categories: LabelState,
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
        s.edition.set_value(2024);
        s.edition.set_range((2015, 2024));
        s.edition.set_step(3);
        s
    }
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
    let form = Form::new().styles(THEME.pager_style());

    // maybe rebuild layout
    let layout_size = form.layout_size(l2[1]);
    if !state.form.valid_layout(layout_size) {
        let mut form = LayoutForm::new() //
            .spacing(1)
            .line_spacing(1)
            .flex(Flex::Legacy);

        form.widget(0, FormLabel::Str("_Name"), FormWidget::Width(20));
        form.widget(1, FormLabel::Str("_Version"), FormWidget::Width(12));
        form.widget(2, FormLabel::Str("_Edition"), FormWidget::Width(8));
        form.widget(3, FormLabel::Str("_Author"), FormWidget::Width(20));
        form.widget(
            4,
            FormLabel::Str("_Description"),
            FormWidget::StretchXY(30, 4),
        );
        form.widget(5, FormLabel::Str("_License"), FormWidget::Width(15));
        form.widget(6, FormLabel::Str("_Repository"), FormWidget::Width(35));
        form.widget(7, FormLabel::Str("_Readme"), FormWidget::Width(20));
        form.widget(8, FormLabel::Str("_Keywords"), FormWidget::Width(25));
        form.widget(9, FormLabel::Str("_Category"), FormWidget::Width(25));
        form.widget(10, FormLabel::Str(""), FormWidget::Width(25));
        form.widget(11, FormLabel::Str(""), FormWidget::Width(25));
        form.widget(12, FormLabel::Str(""), FormWidget::Width(25));
        form.widget(13, FormLabel::Str(""), FormWidget::Width(25));

        state
            .form
            .set_layout(form.paged(layout_size, Padding::new(2, 2, 1, 1)));
    }

    // set current layout and prepare rendering.
    let mut form = form.into_buffer(l2[1], frame.buffer_mut(), &mut state.form);

    // render the input fields.
    form.render_label(0, |s, a, b| {
        Label::new_parsed(s.as_ref())
            .styles(istate.theme.label_style())
            .link(&state.name)
            .render(a, b, &mut state.label_name)
    });

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
    // TODO
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

    Ok(Outcome::Continue)
}
