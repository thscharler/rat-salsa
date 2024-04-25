#![allow(unused_variables)]
#![allow(clippy::needless_update)]

use anyhow::anyhow;
use crossbeam::channel::Sender;
use crossterm::event::Event;
#[allow(unused_imports)]
use log::debug;
use rat_salsa::layout::{layout_edit, EditConstraint};
use rat_salsa::number::NumberSymbols;
use rat_salsa::widget::button::ButtonStyle;
use rat_salsa::widget::date_input::{DateInput, DateInputState};
use rat_salsa::widget::input::{TextInput, TextInputState, TextInputStyle};
use rat_salsa::widget::list::{ListExt, ListExtState, ListExtStyle};
use rat_salsa::widget::mask_input::{MaskedInput, MaskedInputState, MaskedInputStyle};
use rat_salsa::widget::menuline::{HotKeyAlt, MenuLine, MenuLineState, MenuStyle};
use rat_salsa::widget::message::{
    StatusDialog, StatusDialogState, StatusDialogStyle, StatusLine, StatusLineState,
};
use rat_salsa::widget::paragraph::{ParagraphExt, ParagraphExtState};
use rat_salsa::widget::scrolled::{Scrolled, ScrolledState};
use rat_salsa::widget::table::{TableExt, TableExtState, TableExtStyle};
use rat_salsa::widget::text_area::{TextAreaExt, TextAreaExtState};
use rat_salsa::widget::tree::{TreeExt, TreeExtState};
use rat_salsa::widget::viewport::{Viewport, ViewportState};
use rat_salsa::{
    check_break, match_focus, on_gained, on_lost, run_tui, tr, validate, ControlUI, DefaultKeys,
    Focus, HandleCrossterm, HasFocusFlag, HasValidFlag, RenderFrameWidget, Repaint, RepaintEvent,
    RunConfig, Timed, Timers, TuiApp,
};
use rat_salsa::{SetSelection, SingleSelection};
use ratatui::layout::{Constraint, Direction, Layout, Rect, Size};
use ratatui::prelude::{Color, Style};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Bar, BarChart, BarGroup, Block, BorderType, Borders, ListItem, Row, Wrap};
use ratatui::Frame;
use std::fs;
use std::iter::repeat_with;
use std::rc::Rc;
use tui_tree_widget::TreeItem;

fn main() -> Result<(), anyhow::Error> {
    _ = fs::remove_file("log.log");

    setup_logging()?;

    let sym = Rc::new(NumberSymbols {
        decimal_sep: ',',
        decimal_grp: Some('.'),
        ..Default::default()
    });

    let mut data = FormOneData::default();
    let mut state = FormOneState::new(&sym);

    run_tui(
        &FormOneApp,
        &mut data,
        &mut state,
        RunConfig {
            n_threats: 1,
            log_timing: false,
            ..RunConfig::default()
        },
    )?;

    Ok(())
}

// -----------------------------------------------------------------------

type Control = ControlUI<FormOneAction, anyhow::Error>;

#[derive(Debug, Default)]
pub struct FormOneData {}

#[derive(Debug)]
pub enum FormOneAction {}

#[derive(Debug)]
pub struct FormOneState {
    pub g: GeneralState,

    pub menu: MenuLineState<MenuItem>,

    pub textinput: FormTextInput,
    pub dateinput: FormDateInput,
    pub scrolled_para: FormScrolledParagraph,
    pub scrolled_table: FormScrolledTable,
    pub scrolled_list: FormScrolledList,
    pub text_area: FormTextArea,
    pub scroll_other: FormOther,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MenuItem {
    Text,
    Date,
    Paragraph,
    Table,
    List,
    TextArea,
    Tree,
    Error,
    Quit,
}

#[derive(Debug)]
pub struct GeneralState {
    pub theme: &'static Theme,
    pub repaint: Repaint,
    pub timers: Timers,

    pub status: StatusLineState,
    pub error_dlg: StatusDialogState,
}

#[derive(Debug)]
pub struct FormTextInput {
    pub text: TextInputState,
    pub decimal: TextInputState,
    pub float: TextInputState,

    pub ipv4: MaskedInputState,
    pub hexcolor: MaskedInputState,
    pub creditcard: MaskedInputState,
    pub date: MaskedInputState,
    pub alpha: MaskedInputState,
    pub dec7_2: MaskedInputState,
    pub euro: MaskedInputState,
    pub exp: MaskedInputState,
}

#[derive(Debug)]
pub struct FormDateInput {
    pub date1: DateInputState,
    pub date2: DateInputState,
    pub date3: DateInputState,
}

#[derive(Debug)]
pub struct FormScrolledParagraph {
    pub para1: ScrolledState<ParagraphExtState>,
    pub para2: ScrolledState<ParagraphExtState>,
    pub para3: ScrolledState<ParagraphExtState>,
    pub para4: ScrolledState<ParagraphExtState>,
}

#[derive(Debug)]
pub struct FormScrolledTable {
    pub table1: ScrolledState<TableExtState<SingleSelection>>,
    pub table2: TableExtState<SetSelection>,
}

#[derive(Debug)]
pub struct FormScrolledList {
    pub list1: ScrolledState<ListExtState<SingleSelection>>,
}

#[derive(Debug)]
pub struct FormTextArea {
    pub text: ScrolledState<TextAreaExtState<'static>>,
}

#[derive(Debug)]
pub struct FormOther {
    pub tree: ScrolledState<TreeExtState<usize>>,
    pub chart: ScrolledState<ViewportState>,
}

impl FormOneState {
    pub fn new(sym: &Rc<NumberSymbols>) -> Self {
        let mut s = Self {
            g: GeneralState::new(),
            menu: Default::default(),
            textinput: FormTextInput::new(sym),
            dateinput: FormDateInput::new(sym),
            scrolled_para: FormScrolledParagraph::new(),
            scrolled_table: FormScrolledTable::new(),
            scrolled_list: FormScrolledList::new(),
            text_area: FormTextArea::new(),
            scroll_other: FormOther::new(),
        };
        s.menu.select(Some(0));
        s
    }
}

impl GeneralState {
    pub fn new() -> Self {
        Self {
            theme: &ONEDARK,
            repaint: Default::default(),
            timers: Default::default(),
            status: Default::default(),
            error_dlg: Default::default(),
        }
    }
}

impl FormTextInput {
    pub fn new(sym: &Rc<NumberSymbols>) -> Self {
        let mut s = Self {
            text: TextInputState::default(),
            decimal: TextInputState::default(),
            float: TextInputState::default(),
            ipv4: MaskedInputState::new_with_symbols(sym),
            hexcolor: MaskedInputState::new_with_symbols(sym),
            creditcard: MaskedInputState::new_with_symbols(sym),
            date: MaskedInputState::new_with_symbols(sym),
            alpha: MaskedInputState::new_with_symbols(sym),
            dec7_2: MaskedInputState::new_with_symbols(sym),
            euro: MaskedInputState::new_with_symbols(sym),
            exp: MaskedInputState::new_with_symbols(sym),
        };

        s.text.focus.set();

        s.ipv4.set_mask("990\\.990\\.990\\.990").expect("mask");
        // s.ipv4.set_display_mask("xxx.xxx.xxx.xxx");
        s.hexcolor.set_mask("HHHHHH").expect("mask");
        s.creditcard.set_mask("dddd dddd dddd dddd").expect("mask");
        // s.creditcard.set_display_mask("dddd dddd dddd dddd");
        s.date.set_mask("99/99/9999").expect("mask");
        s.date.set_display_mask("mm/dd/yyyy");
        s.alpha.set_mask("llllllllll").expect("mask");
        s.dec7_2.set_mask("#,###,##0.00").expect("mask");
        s.euro.set_mask("€ ###,##0.00-").expect("mask");
        s.exp.set_mask("0.#######\\e###").expect("mask");
        s
    }
}

impl FormDateInput {
    pub fn new(sym: &Rc<NumberSymbols>) -> Self {
        let mut s = Self {
            date1: DateInputState::default(),
            date2: DateInputState::default(),
            date3: DateInputState::default(),
        };
        s.date1.set_format("%d.%m.%Y").expect("mask1");
        s.date2.set_format("%x").expect("mask1");
        s.date3.set_format("%c").expect("mask1");
        s
    }
}

impl FormScrolledParagraph {
    pub fn new() -> Self {
        Self {
            para1: Default::default(),
            para2: Default::default(),
            para3: Default::default(),
            para4: Default::default(),
        }
    }
}

impl FormScrolledTable {
    pub fn new() -> Self {
        Self {
            table1: Default::default(),
            table2: Default::default(),
        }
    }
}

impl FormScrolledList {
    pub fn new() -> Self {
        Self {
            list1: Default::default(),
        }
    }
}

impl FormTextArea {
    pub fn new() -> Self {
        Self {
            text: Default::default(),
        }
    }
}

impl FormOther {
    pub fn new() -> Self {
        Self {
            tree: Default::default(),
            chart: Default::default(),
        }
    }
}

// -----------------------------------------------------------------------

#[derive(Debug)]
pub struct FormOneApp;

#[derive(Debug, Clone, Copy)]
pub struct FormOneAppLayout {
    area: Rect,
    menu: Rect,
    status: Rect,
}

impl TuiApp for FormOneApp {
    type Data = FormOneData;
    type State = FormOneState;
    type Action = FormOneAction;
    type Error = anyhow::Error;

    fn get_repaint<'b>(&self, uistate: &'b Self::State) -> Option<&'b Repaint> {
        Some(&uistate.g.repaint)
    }

    fn get_timers<'b>(&self, uistate: &'b Self::State) -> Option<&'b Timers> {
        Some(&uistate.g.timers)
    }

    fn init(
        &self,
        _data: &mut Self::Data,
        _uistate: &mut Self::State,
        _send: &Sender<Self::Action>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn repaint(
        &self,
        event: RepaintEvent,
        frame: &mut Frame<'_>,
        data: &mut Self::Data,
        uistate: &mut Self::State,
    ) -> ControlUI<Self::Action, Self::Error> {
        let area = frame.size();

        let layout = {
            let r = Layout::new(
                Direction::Vertical,
                [
                    Constraint::Fill(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ],
            )
            .split(area);

            FormOneAppLayout {
                area: r[0],
                menu: r[1],
                status: r[2],
            }
        };

        match uistate.menu.action() {
            Some(MenuItem::Text) => {
                tr!(repaint_textinput(&event, frame, layout, data, uistate), _)
            }
            Some(MenuItem::Date) => {
                tr!(repaint_dateinput(&event, frame, layout, data, uistate), _)
            }
            Some(MenuItem::Paragraph) => {
                tr!(
                    repaint_scrolled_paragraph(&event, frame, layout, data, uistate),
                    _
                );
            }
            Some(MenuItem::Table) => {
                tr!(
                    repaint_scrolled_table(&event, frame, layout, data, uistate),
                    _
                )
            }
            Some(MenuItem::List) => {
                tr!(
                    repaint_scrolled_list(&event, frame, layout, data, uistate),
                    _
                )
            }
            Some(MenuItem::TextArea) => {
                tr!(repaint_textarea(&event, frame, layout, data, uistate), _)
            }
            Some(MenuItem::Tree) => {
                tr!(repaint_tree(&event, frame, layout, data, uistate), _)
            }
            _ => {}
        }
        tr!(repaint_menu(&event, frame, layout, data, uistate), _);

        if uistate.g.error_dlg.active {
            let err = StatusDialog::new().style(uistate.g.theme.status_dialog_style());
            frame.render_stateful_widget(err, layout.area, &mut uistate.g.error_dlg);
        }

        let status = StatusLine::new().style(uistate.g.theme.status_style());
        frame.render_stateful_widget(status, layout.status, &mut uistate.g.status);

        Control::Continue
    }

    fn handle_timer(
        &self,
        event: Timed,
        data: &mut Self::Data,
        uistate: &mut Self::State,
    ) -> ControlUI<Self::Action, Self::Error> {
        // TODO: timer
        Control::Continue
    }

    fn handle_event(
        &self,
        event: Event,
        data: &mut Self::Data,
        uistate: &mut Self::State,
    ) -> ControlUI<Self::Action, Self::Error> {
        use crossterm::event::*;

        check_break!(match &event {
            Event::Resize(_, _) => {
                //
                Control::Change
            }
            Event::Key(KeyEvent {
                kind: KeyEventKind::Press,
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => {
                //
                Control::Break
            }
            _ => Control::Continue,
        });

        check_break!({
            if uistate.g.error_dlg.active {
                uistate.g.error_dlg.handle(&event, DefaultKeys)
            } else {
                Control::Continue
            }
        });

        check_break!(match uistate.menu.action() {
            Some(MenuItem::Text) => handle_textinput(&event, data, uistate),
            Some(MenuItem::Date) => handle_dateinput(&event, data, uistate),
            Some(MenuItem::Paragraph) => handle_scrolled_paragraph(&event, data, uistate),
            Some(MenuItem::Table) => handle_scrolled_table(&event, data, uistate),
            Some(MenuItem::List) => handle_scrolled_list(&event, data, uistate),
            Some(MenuItem::TextArea) => handle_textarea(&event, data, uistate),
            Some(MenuItem::Tree) => handle_tree(&event, data, uistate),
            Some(MenuItem::Error) => handle_error(&event, data, uistate),
            _ => Control::Continue,
        });

        check_break!(handle_menu(&event, data, uistate));

        Control::Continue
    }

    fn run_action(
        &self,
        action: Self::Action,
        data: &mut Self::Data,
        uistate: &mut Self::State,
        send: &Sender<Self::Action>,
    ) -> ControlUI<Self::Action, Self::Error> {
        // TODO: actions
        Control::Continue
    }

    fn run_task(&self, task: Self::Action, send: &Sender<Control>) -> Control {
        // TODO: tasks
        Control::Continue
    }

    fn report_error(
        &self,
        error: Self::Error,
        data: &mut Self::Data,
        uistate: &mut Self::State,
    ) -> Control {
        uistate.g.error_dlg.log(format!("{:?}", &*error).as_str());
        Control::Change
    }
}

fn repaint_menu(
    event: &RepaintEvent,
    frame: &mut Frame<'_>,
    layout: FormOneAppLayout,
    data: &mut FormOneData,
    uistate: &mut FormOneState,
) -> Control {
    let menu = MenuLine::new()
        .style(uistate.g.theme.menu_style())
        .title("Select form:")
        .add("TextField", MenuItem::Text)
        .add("DateField", MenuItem::Date)
        .add("Scrolling", MenuItem::Paragraph)
        .add("Table", MenuItem::Table)
        .add("List", MenuItem::List)
        .add("TextArea", MenuItem::TextArea)
        .add("Other", MenuItem::Tree)
        .add("Error", MenuItem::Error)
        .add("_Quit", MenuItem::Quit);
    frame.render_stateful_widget(menu, layout.menu, &mut uistate.menu);

    Control::Continue
}

fn focus_menu(state: &MenuLineState<MenuItem>) -> Focus<'_> {
    Focus::new([(state.focus(), state.area())])
}

fn handle_menu(event: &Event, data: &mut FormOneData, uistate: &mut FormOneState) -> Control {
    check_break!(uistate
        .menu
        .handle(event, DefaultKeys)
        .or_else(|| uistate.menu.handle(event, HotKeyAlt))
        .and_then(|a| match a {
            MenuItem::Quit => {
                Control::Break
            }
            _ => Control::Change,
        }));

    Control::Continue
}

fn repaint_textinput(
    event: &RepaintEvent,
    frame: &mut Frame<'_>,
    layout: FormOneAppLayout,
    data: &mut FormOneData,
    uistate: &mut FormOneState,
) -> Control {
    // TODO: repaint_mask
    let l_columns = Layout::new(
        Direction::Horizontal,
        [
            Constraint::Fill(2),
            Constraint::Fill(1),
            Constraint::Fill(2),
            Constraint::Fill(1),
        ],
    )
    .split(layout.area);

    let l0 = layout_edit(
        l_columns[0],
        &[
            EditConstraint::TitleLabelRows(2),
            EditConstraint::Empty,
            EditConstraint::Label("Text"),
            EditConstraint::Widget(20),
            EditConstraint::Label("Integer"),
            EditConstraint::Widget(12),
            EditConstraint::Label("Float"),
            EditConstraint::Widget(12),
        ],
    );
    let mut l0 = l0.iter();

    let l2 = layout_edit(
        l_columns[2],
        &[
            EditConstraint::TitleLabelRows(2),
            EditConstraint::Empty,
            EditConstraint::Label("IPv4"),
            EditConstraint::Widget(16),
            EditConstraint::Label("Color"),
            EditConstraint::Widget(7),
            EditConstraint::Label("Credit card"),
            EditConstraint::Widget(20),
            EditConstraint::Label("Date"),
            EditConstraint::Widget(11),
            EditConstraint::Label("Name"),
            EditConstraint::Widget(11),
            EditConstraint::Label("Decimal 7.2"),
            EditConstraint::Widget(20),
            EditConstraint::Label("Euro"),
            EditConstraint::Widget(20),
            EditConstraint::Label("Exp"),
            EditConstraint::Widget(20),
        ],
    );
    let mut l2 = l2.iter();

    let w_text = TextInput::default().style(uistate.g.theme.input_style());
    let w_decimal = TextInput::default().style(uistate.g.theme.input_style());
    let w_float = TextInput::default().style(uistate.g.theme.input_style());

    let w_color = MaskedInput::default().style(uistate.g.theme.input_mask_style());
    let w_ipv4 = MaskedInput::default()
        .show_compact(true)
        .style(uistate.g.theme.input_mask_style());
    let w_creditcard = MaskedInput::default().style(uistate.g.theme.input_mask_style());
    let w_date = MaskedInput::default()
        .show_compact(true)
        .style(uistate.g.theme.input_mask_style());
    let w_name = MaskedInput::default().style(uistate.g.theme.input_mask_style());
    let w_dec_7_2 = MaskedInput::default().style(uistate.g.theme.input_mask_style());
    let w_euro = MaskedInput::default().style(uistate.g.theme.input_mask_style());
    let w_exp = MaskedInput::default()
        .show_compact(true)
        .style(uistate.g.theme.input_mask_style());

    frame.render_widget(Span::from("Plain text input").underlined(), l0.label());
    frame.render_widget(Span::from("Text"), l0.label());
    frame.render_frame_widget(w_text, l0.widget(), &mut uistate.textinput.text);
    frame.render_widget(Span::from("Integer"), l0.label());
    frame.render_frame_widget(w_decimal, l0.widget(), &mut uistate.textinput.decimal);
    frame.render_widget(Span::from("Float"), l0.label());
    frame.render_frame_widget(w_float, l0.widget(), &mut uistate.textinput.float);

    frame.render_widget(Span::from("Masked text input").underlined(), l2.label());
    frame.render_widget(Span::from("IPv4"), l2.label());
    frame.render_frame_widget(w_ipv4, l2.widget(), &mut uistate.textinput.ipv4);
    frame.render_widget(Span::from("Color"), l2.label());
    frame.render_frame_widget(w_color, l2.widget(), &mut uistate.textinput.hexcolor);
    frame.render_widget(Span::from("Credit card"), l2.label());
    frame.render_frame_widget(w_creditcard, l2.widget(), &mut uistate.textinput.creditcard);
    frame.render_widget(Span::from("Date"), l2.label());
    frame.render_frame_widget(w_date, l2.widget(), &mut uistate.textinput.date);
    frame.render_widget(Span::from("Name"), l2.label());
    frame.render_frame_widget(w_name, l2.widget(), &mut uistate.textinput.alpha);
    frame.render_widget(Span::from("Decimal 7.2"), l2.label());
    frame.render_frame_widget(w_dec_7_2, l2.widget(), &mut uistate.textinput.dec7_2);
    frame.render_widget(Span::from("Euro"), l2.label());
    frame.render_frame_widget(w_euro, l2.widget(), &mut uistate.textinput.euro);
    frame.render_widget(Span::from("Exp"), l2.label());
    frame.render_frame_widget(w_exp, l2.widget(), &mut uistate.textinput.exp);

    let r = match_focus!(
        uistate.textinput.text => Some(&uistate.textinput.text),
        uistate.textinput.decimal => Some(&uistate.textinput.decimal),
        uistate.textinput.float => Some(&uistate.textinput.float),
        _ => None
    );
    if let Some(r) = r {
        let mut ec = Vec::new();
        ec.push(EditConstraint::EmptyRows(2));
        ec.push(EditConstraint::Empty);
        ec.push(EditConstraint::TitleLabel);
        ec.push(EditConstraint::TitleLabel);

        let l1 = layout_edit(l_columns[1], &ec);
        let mut l1 = l1.iter();

        frame.render_widget(Span::from(format!("value={}", r.value())), l1.label());
        frame.render_widget(
            Span::from(format!(
                "o={} w={} c={} s={}:{}",
                r.offset(),
                r.width(),
                r.cursor(),
                r.selection().start,
                r.selection().end
            )),
            l1.label(),
        );
    }

    let r = match_focus!(
        uistate.textinput.ipv4 => Some(&uistate.textinput.ipv4),
        uistate.textinput.hexcolor => Some(&uistate.textinput.hexcolor),
        uistate.textinput.creditcard => Some(&uistate.textinput.creditcard),
        uistate.textinput.date => Some(&uistate.textinput.date),
        uistate.textinput.alpha =>Some( &uistate.textinput.alpha),
        uistate.textinput.dec7_2 => Some(&uistate.textinput.dec7_2),
        uistate.textinput.euro => Some(&uistate.textinput.euro),
        uistate.textinput.exp => Some(&uistate.textinput.exp),
        _ => None
    );
    if let Some(r) = r {
        let mut ec = Vec::new();
        ec.push(EditConstraint::EmptyRows(2));
        ec.push(EditConstraint::Empty);
        for _ in 0..r.value.tokens().len() {
            ec.push(EditConstraint::TitleLabel);
        }
        ec.push(EditConstraint::TitleLabel);
        ec.push(EditConstraint::TitleLabel);
        ec.push(EditConstraint::TitleLabel);
        ec.push(EditConstraint::TitleLabel);
        ec.push(EditConstraint::TitleLabel);
        ec.push(EditConstraint::TitleLabel);

        let l3 = layout_edit(l_columns[3], &ec);
        let mut l3 = l3.iter();

        for (i, t) in r.value.tokens().iter().enumerate() {
            let mut w_info = Span::from(format!(
                "#{}:{}:{}-{}   {:?} | {:?}",
                t.nr_id, t.sec_id, t.sec_start, t.sec_end, t.peek_left, t.right
            ));
            if i == r.cursor() {
                w_info = w_info.on_cyan();
            }
            frame.render_widget(w_info, l3.label());
        }
        frame.render_widget(Span::from(format!("value={}", r.value())), l3.label());
        frame.render_widget(
            Span::from(format!("compact={}", r.compact_value())),
            l3.label(),
        );
        frame.render_widget(
            Span::from(format!(
                "parse={:?}",
                r.compact_value().as_str().parse::<f64>()
            )),
            l3.label(),
        );
        frame.render_widget(Span::from(format!("mask={}", r.mask())), l3.label());
        frame.render_widget(
            Span::from(format!("display={}", r.display_mask())),
            l3.label(),
        );
        frame.render_widget(
            Span::from(format!(
                "o={} w={} c={} s={}:{}",
                r.offset(),
                r.width(),
                r.cursor(),
                r.selection().start,
                r.selection().end
            )),
            l3.label(),
        );
    }

    Control::Continue
}

fn focus_textinput(mask0: &FormTextInput) -> Focus<'_> {
    Focus::new([
        (mask0.text.focus(), mask0.text.area()),
        (mask0.decimal.focus(), mask0.decimal.area()),
        (mask0.float.focus(), mask0.float.area()),
        (mask0.ipv4.focus(), mask0.ipv4.area()),
        (mask0.hexcolor.focus(), mask0.hexcolor.area()),
        (mask0.creditcard.focus(), mask0.creditcard.area()),
        (mask0.date.focus(), mask0.date.area()),
        (mask0.alpha.focus(), mask0.alpha.area()),
        (mask0.dec7_2.focus(), mask0.dec7_2.area()),
        (mask0.euro.focus(), mask0.euro.area()),
        (mask0.exp.focus(), mask0.exp.area()),
    ])
}

fn handle_textinput(event: &Event, data: &mut FormOneData, uistate: &mut FormOneState) -> Control {
    let mask0 = &mut uistate.textinput;

    focus_textinput(mask0)
        .append(focus_menu(&uistate.menu))
        .handle(event, DefaultKeys)
        .and_do(|_| uistate.g.repaint.set());

    on_lost!(
        mask0.decimal => {
            let v = mask0.decimal.value().parse::<i64>();
            if let Some(v) = mask0.decimal.set_valid_from(v) {
                mask0.decimal.set_value(format!("{}", v));
            }
        },
        mask0.float => {
            let v = mask0.float.value().parse::<f64>();
            if let Some(v) = mask0.float.set_valid_from(v) {
                mask0.float.set_value(format!("{}", v));
            }
        },
        mask0.ipv4 => {
            // mask0.ipv4.value()

        }
    );
    on_gained!(
        mask0.decimal => {
            mask0.decimal.select_all();
        },
        mask0.float => {
            mask0.float.select_all();
        }
    );

    // TODO: handle_mask
    check_break!(mask0.text.handle(event, DefaultKeys));
    check_break!(mask0.decimal.handle(event, DefaultKeys));
    check_break!(mask0.float.handle(event, DefaultKeys));
    check_break!(mask0.ipv4.handle(event, DefaultKeys));
    check_break!(mask0.hexcolor.handle(event, DefaultKeys));
    check_break!(mask0.creditcard.handle(event, DefaultKeys));
    check_break!(mask0.date.handle(event, DefaultKeys));
    check_break!(mask0.alpha.handle(event, DefaultKeys));
    check_break!(mask0.dec7_2.handle(event, DefaultKeys));
    check_break!(mask0.euro.handle(event, DefaultKeys));
    check_break!(mask0.exp.handle(event, DefaultKeys));

    Control::Continue
}

fn repaint_dateinput(
    event: &RepaintEvent,
    frame: &mut Frame<'_>,
    layout: FormOneAppLayout,
    data: &mut FormOneData,
    uistate: &mut FormOneState,
) -> Control {
    let l_columns = Layout::new(
        Direction::Horizontal,
        [
            Constraint::Fill(2),
            Constraint::Fill(1),
            Constraint::Fill(2),
            Constraint::Fill(1),
        ],
    )
    .split(layout.area);

    let l0 = layout_edit(
        l_columns[0],
        &[
            EditConstraint::TitleLabelRows(2),
            EditConstraint::Empty,
            EditConstraint::Label("Date 1"),
            EditConstraint::Widget(20),
            EditConstraint::Label("Date 2"),
            EditConstraint::Widget(20),
            EditConstraint::Label("Date 3"),
            EditConstraint::Widget(20),
        ],
    );
    let mut l0 = l0.iter();

    let w_date1 = DateInput::default().style(uistate.g.theme.input_mask_style());
    let w_date2 = DateInput::default().style(uistate.g.theme.input_mask_style());
    let w_date3 = DateInput::default().style(uistate.g.theme.input_mask_style());

    frame.render_widget(Span::from("Date input").underlined(), l0.label());
    frame.render_widget(Span::from("Date 1"), l0.label());
    frame.render_frame_widget(w_date1, l0.widget(), &mut uistate.dateinput.date1);
    frame.render_widget(Span::from("Date 2"), l0.label());
    frame.render_frame_widget(w_date2, l0.widget(), &mut uistate.dateinput.date2);
    frame.render_widget(Span::from("Date 3"), l0.label());
    frame.render_frame_widget(w_date3, l0.widget(), &mut uistate.dateinput.date3);

    let r = match_focus!(
        uistate.dateinput.date1 => Some(&uistate.dateinput.date1),
        uistate.dateinput.date2 => Some(&uistate.dateinput.date2),
        uistate.dateinput.date3 => Some(&uistate.dateinput.date3),
        _ => None
    );
    if let Some(r) = r {
        let mut ec = Vec::new();
        ec.push(EditConstraint::EmptyRows(2));
        ec.push(EditConstraint::Empty);
        for _ in 0..r.input.value.tokens().len() {
            ec.push(EditConstraint::TitleLabel);
        }

        let l1 = layout_edit(l_columns[1], &ec);
        let mut l1 = l1.iter();

        for (i, t) in r.input.value.tokens().iter().enumerate() {
            let mut w_info = Span::from(format!(
                "#{}:{}:{}-{}   {:?} | {:?}",
                t.nr_id, t.sec_id, t.sec_start, t.sec_end, t.peek_left, t.right
            ));
            if i == r.input.cursor() {
                w_info = w_info.on_cyan();
            }
            frame.render_widget(w_info, l1.label());
        }

        let mut ec = Vec::new();
        ec.push(EditConstraint::EmptyRows(2));
        ec.push(EditConstraint::Empty);
        ec.push(EditConstraint::TitleLabel);
        ec.push(EditConstraint::TitleLabel);
        ec.push(EditConstraint::TitleLabel);
        ec.push(EditConstraint::TitleLabel);
        ec.push(EditConstraint::TitleLabel);
        ec.push(EditConstraint::TitleLabel);
        ec.push(EditConstraint::TitleLabel);

        let l2 = layout_edit(l_columns[2], &ec);
        let mut l2 = l2.iter();

        frame.render_widget(Span::from(format!("value={}", r.input.value())), l2.label());
        frame.render_widget(
            Span::from(format!("compact={}", r.input.compact_value())),
            l2.label(),
        );
        frame.render_widget(Span::from(format!("parse={:?}", r.value())), l2.label());
        frame.render_widget(Span::from(format!("pattern={}", r.pattern)), l2.label());
        frame.render_widget(Span::from(format!("mask={}", r.input.mask())), l2.label());
        frame.render_widget(
            Span::from(format!("display={}", r.input.display_mask())),
            l2.label(),
        );
        frame.render_widget(
            Span::from(format!(
                "o={} w={} c={} s={}:{}",
                r.input.offset(),
                r.input.width(),
                r.input.cursor(),
                r.input.selection().start,
                r.input.selection().end
            )),
            l2.label(),
        );
    }

    Control::Continue
}

fn focus_dateinput(mask1: &FormDateInput) -> Focus<'_> {
    Focus::new([
        (mask1.date1.focus(), mask1.date1.area()),
        (mask1.date2.focus(), mask1.date2.area()),
        (mask1.date3.focus(), mask1.date3.area()),
    ])
}

fn handle_dateinput(event: &Event, data: &mut FormOneData, uistate: &mut FormOneState) -> Control {
    let mask1 = &mut uistate.dateinput;

    focus_dateinput(mask1)
        .append(focus_menu(&uistate.menu))
        .handle(event, DefaultKeys)
        .and_do(|_| uistate.g.repaint.set());

    validate!(mask1.date1, mask1.date2, mask1.date3);

    check_break!(mask1.date1.handle(event, DefaultKeys));
    check_break!(mask1.date2.handle(event, DefaultKeys));
    check_break!(mask1.date3.handle(event, DefaultKeys));

    Control::Continue
}

fn repaint_scrolled_paragraph(
    event: &RepaintEvent,
    frame: &mut Frame<'_>,
    layout: FormOneAppLayout,
    data: &mut FormOneData,
    uistate: &mut FormOneState,
) -> Control {
    let l_columns = Layout::new(
        Direction::Horizontal,
        [
            Constraint::Fill(1),
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(2),
            Constraint::Fill(1),
        ],
    )
    .split(layout.area);

    let w_para = create_para();
    let w_para = Scrolled::new(w_para)
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title("no overscroll")
                .title_style(Style::default().underlined()),
        )
        .v_overscroll(0);
    frame.render_stateful_widget(w_para, l_columns[0], &mut uistate.scrolled_para.para1);

    let w_para = create_para();
    let w_para = Scrolled::new(w_para)
        .block(
            Block::default()
                .borders(Borders::RIGHT | Borders::BOTTOM)
                .title("overscroll")
                .title_style(Style::default().underlined()),
        )
        .v_overscroll(5);
    frame.render_stateful_widget(w_para, l_columns[2], &mut uistate.scrolled_para.para2);

    let w_para = create_para().block(
        Block::default()
            .title("inner border")
            .borders(Borders::ALL)
            .border_type(BorderType::Plain),
    );
    let w_para = Scrolled::new(w_para);
    frame.render_stateful_widget(w_para, l_columns[4], &mut uistate.scrolled_para.para3);

    let w_para = create_para().wrap(Wrap { trim: true });
    let w_para = Scrolled::new(w_para);
    frame.render_stateful_widget(w_para, l_columns[6], &mut uistate.scrolled_para.para4);

    Control::Continue
}

fn create_para() -> ParagraphExt<'static> {
    ParagraphExt::new(
        [
            "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, ",
            "sed diam nonumy eirmod tempor invidunt ut labore et dolore",
            "magna aliquyam erat, sed diam voluptua. At vero eos et ",
            "accusam et justo duo dolores et ea rebum. Stet clita kasd ",
            "gubergren, no sea takimata sanctus est Lorem ipsum dolor ",
            "sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing ",
            "elitr, sed diam nonumy eirmod tempor invidunt ut labore et",
            "dolore magna aliquyam erat, sed diam voluptua. At vero ",
            "eos et accusam et justo duo dolores et ea rebum. Stet",
            "clita kasd gubergren, no sea takimata sanctus est Lorem ",
            "ipsum dolor sit amet. Lorem ipsum dolor sit amet, ",
            "consetetur sadipscing elitr, sed diam nonumy eirmod tempor ",
            "invidunt ut labore et dolore magna aliquyam erat, sed diam",
            "voluptua. At vero eos et accusam et justo duo dolores et ",
            "ea rebum. Stet clita kasd gubergren, no sea takimata sanctus ",
            "est Lorem ipsum dolor sit amet.   ",
            "",
            "Duis autem vel eum iriure dolor in hendrerit in vulputate ",
            "velit esse molestie consequat, vel illum dolore eu feugiat ",
            "nulla facilisis at vero eros et accumsan et iusto odio",
            " dignissim qui blandit praesent luptatum zzril delenit ",
            "augue duis dolore te feugait nulla facilisi. Lorem ipsum ",
            "dolor sit amet, consectetuer adipiscing elit, sed diam ",
            "nonummy nibh euismod tincidunt ut laoreet dolore magna ",
            "aliquam erat volutpat.   ",
            "",
            "Ut wisi enim ad minim veniam, quis nostrud exerci tation ",
            "ullamcorper suscipit lobortis nisl ut aliquip ex ea commodo ",
            "consequat. Duis autem vel eum iriure dolor in hendrerit in ",
            "vulputate velit esse molestie consequat, vel illum dolore ",
            "eu feugiat nulla facilisis at vero eros et accumsan et ",
            "iusto odio dignissim qui blandit praesent luptatum zzril ",
            "delenit augue duis dolore te feugait nulla facilisi.   ",
            "",
            "Nam liber tempor cum soluta nobis eleifend option congue ",
            "nihil imperdiet doming id quod mazim placerat facer possim ",
            "assum. Lorem ipsum dolor sit amet, consectetuer adipiscing ",
            "elit, sed diam nonummy nibh euismod tincidunt ut laoreet ",
            "dolore magna aliquam erat volutpat. Ut wisi enim ad minim ",
            "veniam, quis nostrud exerci tation ullamcorper suscipit ",
            "lobortis nisl ut aliquip ex ea commodo consequat.   ",
            "",
            "Duis autem vel eum iriure dolor in hendrerit in vulputate ",
            "velit esse molestie consequat, vel illum dolore eu feugiat ",
            "nulla facilisis.   ",
            "",
            "At vero eos et accusam et justo duo dolores et ea rebum. ",
            "Stet clita kasd gubergren, no sea takimata sanctus est ",
            "Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, ",
            "consetetur sadipscing elitr, sed diam nonumy eirmod tempor ",
            "invidunt ut labore et dolore magna aliquyam erat, sed diam ",
            "voluptua. At vero eos et accusam et justo duo dolores et ",
            "ea rebum. Stet clita kasd gubergren, no sea takimata sanctus ",
            "est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit ",
            "amet, consetetur sadipscing elitr, At accusam aliquyam diam ",
            "diam dolore dolores duo eirmod eos erat, et nonumy sed ",
            "tempor et et invidunt justo labore Stet clita ea et gubergren, ",
            "kasd magna no rebum. sanctus sea sed takimata ut vero ",
            "voluptua. est Lorem ipsum dolor sit amet. Lorem ipsum dolor ",
            "sit amet, consetetur",
        ]
        .iter()
        .map(|v| Line::from(*v))
        .collect::<Vec<_>>(),
    )
}

fn handle_scrolled_paragraph(
    event: &Event,
    data: &mut FormOneData,
    uistate: &mut FormOneState,
) -> Control {
    let mask2 = &mut uistate.scrolled_para;

    Focus::new([])
        .append(focus_menu(&uistate.menu))
        .handle(event, DefaultKeys)
        .and_do(|_| uistate.g.repaint.set());

    check_break!(mask2.para1.handle(event, DefaultKeys));
    check_break!(mask2.para2.handle(event, DefaultKeys));
    check_break!(mask2.para3.handle(event, DefaultKeys));
    check_break!(mask2.para4.handle(event, DefaultKeys));

    Control::Continue
}

fn handle_error(event: &Event, data: &mut FormOneData, uistate: &mut FormOneState) -> Control {
    for s in [
        "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, ",
        "sed diam nonumy eirmod tempor invidunt ut labore et dolore",
        "magna aliquyam erat, sed diam voluptua. At vero eos et ",
        "accusam et justo duo dolores et ea rebum. Stet clita kasd ",
        "gubergren, no sea takimata sanctus est Lorem ipsum dolor ",
        "sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing ",
        "elitr, sed diam nonumy eirmod tempor invidunt ut labore et",
        "dolore magna aliquyam erat, sed diam voluptua. At vero ",
        "eos et accusam et justo duo dolores et ea rebum. Stet",
        "clita kasd gubergren, no sea takimata sanctus est Lorem ",
        "ipsum dolor sit amet. Lorem ipsum dolor sit amet, ",
        "consetetur sadipscing elitr, sed diam nonumy eirmod tempor ",
        "invidunt ut labore et dolore magna aliquyam erat, sed diam",
        "voluptua. At vero eos et accusam et justo duo dolores et ",
        "ea rebum. Stet clita kasd gubergren, no sea takimata sanctus ",
        "est Lorem ipsum dolor sit amet.   ",
        "",
        "Duis autem vel eum iriure dolor in hendrerit in vulputate ",
        "velit esse molestie consequat, vel illum dolore eu feugiat ",
        "nulla facilisis at vero eros et accumsan et iusto odio",
        " dignissim qui blandit praesent luptatum zzril delenit ",
        "augue duis dolore te feugait nulla facilisi. Lorem ipsum ",
        "dolor sit amet, consectetuer adipiscing elit, sed diam ",
        "nonummy nibh euismod tincidunt ut laoreet dolore magna ",
        "aliquam erat volutpat.   ",
        "",
        "Ut wisi enim ad minim veniam, quis nostrud exerci tation ",
        "ullamcorper suscipit lobortis nisl ut aliquip ex ea commodo ",
        "consequat. Duis autem vel eum iriure dolor in hendrerit in ",
        "vulputate velit esse molestie consequat, vel illum dolore ",
        "eu feugiat nulla facilisis at vero eros et accumsan et ",
        "iusto odio dignissim qui blandit praesent luptatum zzril ",
        "delenit augue duis dolore te feugait nulla facilisi.   ",
        "",
        "Nam liber tempor cum soluta nobis eleifend option congue ",
        "nihil imperdiet doming id quod mazim placerat facer possim ",
        "assum. Lorem ipsum dolor sit amet, consectetuer adipiscing ",
        "elit, sed diam nonummy nibh euismod tincidunt ut laoreet ",
        "dolore magna aliquam erat volutpat. Ut wisi enim ad minim ",
        "veniam, quis nostrud exerci tation ullamcorper suscipit ",
        "lobortis nisl ut aliquip ex ea commodo consequat.   ",
        "",
        "Duis autem vel eum iriure dolor in hendrerit in vulputate ",
        "velit esse molestie consequat, vel illum dolore eu feugiat ",
        "nulla facilisis.   ",
        "",
        "At vero eos et accusam et justo duo dolores et ea rebum. ",
        "Stet clita kasd gubergren, no sea takimata sanctus est ",
        "Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, ",
        "consetetur sadipscing elitr, sed diam nonumy eirmod tempor ",
        "invidunt ut labore et dolore magna aliquyam erat, sed diam ",
        "voluptua. At vero eos et accusam et justo duo dolores et ",
        "ea rebum. Stet clita kasd gubergren, no sea takimata sanctus ",
        "est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit ",
        "amet, consetetur sadipscing elitr, At accusam aliquyam diam ",
        "diam dolore dolores duo eirmod eos erat, et nonumy sed ",
        "tempor et et invidunt justo labore Stet clita ea et gubergren, ",
        "kasd magna no rebum. sanctus sea sed takimata ut vero ",
        "voluptua. est Lorem ipsum dolor sit amet. Lorem ipsum dolor ",
        "sit amet, consetetur",
    ] {
        uistate.g.error_dlg.log(s);
    }
    uistate.menu.select(Some(0));

    Control::Change
}

fn repaint_scrolled_table(
    event: &RepaintEvent,
    frame: &mut Frame<'_>,
    layout: FormOneAppLayout,
    data: &mut FormOneData,
    uistate: &mut FormOneState,
) -> Control {
    let l_title = Layout::new(
        Direction::Horizontal,
        [
            Constraint::Fill(2),
            Constraint::Fill(1),
            Constraint::Fill(2),
            Constraint::Fill(1),
        ],
    )
    .split(Rect::new(
        layout.area.x,
        layout.area.y,
        layout.area.width,
        1,
    ));

    let l_columns = Layout::new(
        Direction::Horizontal,
        [
            Constraint::Fill(2),
            Constraint::Fill(1),
            Constraint::Fill(2),
            Constraint::Fill(1),
        ],
    )
    .split(Rect::new(
        layout.area.x,
        layout.area.y + 1,
        layout.area.width,
        layout.area.height - 1,
    ));

    let l_table1 = Span::from("Single selection, external scroll").underlined();
    let w_table1 = create_sample_table().styles(uistate.g.theme.table_style());
    let w_table1 = Scrolled::new(w_table1);

    let l_table2 = Span::from("Multiple selection, internal scroll").underlined();
    let w_table2 = create_sample_table().styles(uistate.g.theme.table_style());

    frame.render_widget(l_table1, l_title[0]);
    frame.render_stateful_widget(w_table1, l_columns[0], &mut uistate.scrolled_table.table1);
    frame.render_widget(l_table2, l_title[2]);
    frame.render_stateful_widget(w_table2, l_columns[2], &mut uistate.scrolled_table.table2);

    ControlUI::Continue
}

fn create_sample_table<'a, SEL>() -> TableExt<'a, SEL> {
    TableExt::from_iter(
        [
            ["1", "2", "3", "4"],
            ["2", "3", "4", "5"],
            ["3", "4", "5", "6"],
            ["4", "5", "6", "7"],
            ["5", "6", "7", "8"],
            ["6", "7", "8", "9"],
            ["7", "8", "9", "10"],
            ["8", "9", "10", "11"],
            ["9", "10", "11", "12"],
            ["10", "11", "12", "13"],
            ["11", "12", "13", "14"],
            ["12", "13", "14", "15"],
            ["13", "14", "15", "16"],
            ["14", "15", "16", "17"],
            ["15", "16", "17", "18"],
            ["16", "17", "18", "19"],
            ["17", "18", "19", "20"],
            ["18", "19", "20", "21"],
            ["19", "20", "21", "22"],
            ["20", "21", "22", "23"],
            ["21", "22", "23", "24"],
            ["22", "23", "24", "25"],
            ["23", "24", "25", "26"],
            ["24", "25", "26", "27"],
            ["25", "26", "27", "28"],
            ["26", "27", "28", "29"],
            ["27", "28", "29", "30"],
            ["28", "29", "30", "31"],
            ["29", "30", "31", "32"],
            ["30", "31", "32", "33"],
            ["31", "32", "33", "34"],
            ["32", "33", "34", "35"],
            ["33", "34", "35", "36"],
            ["34", "35", "36", "37"],
            ["35", "36", "37", "38"],
            ["36", "37", "38", "39"],
            ["37", "38", "39", "40"],
            ["38", "39", "40", "41"],
            ["39", "40", "41", "42"],
            ["40", "41", "42", "43"],
            ["41", "42", "43", "44"],
            ["42", "43", "44", "45"],
            ["43", "44", "45", "46"],
            ["44", "45", "46", "47"],
            ["45", "46", "47", "48"],
            ["46", "47", "48", "49"],
            ["47", "48", "49", "50"],
            ["48", "49", "50", "51"],
            ["49", "50", "51", "52"],
            ["50", "51", "52", "53"],
            ["51", "52", "53", "54"],
            ["52", "53", "54", "55"],
            ["53", "54", "55", "56"],
            ["54", "55", "56", "57"],
            ["55", "56", "57", "58"],
            ["56", "57", "58", "59"],
            ["57", "58", "59", "60"],
            ["58", "59", "60", "61"],
            ["59", "60", "61", "62"],
            ["60", "61", "62", "63"],
            ["61", "62", "63", "64"],
            ["62", "63", "64", "65"],
            ["63", "64", "65", "66"],
            ["64", "65", "66", "67"],
            ["65", "66", "67", "68"],
            ["66", "67", "68", "69"],
            ["67", "68", "69", "70"],
            ["68", "69", "70", "71"],
            ["69", "70", "71", "72"],
            ["70", "71", "72", "73"],
            ["71", "72", "73", "74"],
            ["72", "73", "74", "75"],
            ["73", "74", "75", "76"],
            ["74", "75", "76", "77"],
            ["75", "76", "77", "78"],
            ["76", "77", "78", "79"],
            ["77", "78", "79", "80"],
            ["78", "79", "80", "81"],
            ["79", "80", "81", "82"],
            ["80", "81", "82", "83"],
            ["81", "82", "83", "84"],
            ["82", "83", "84", "85"],
            ["83", "84", "85", "86"],
            ["84", "85", "86", "87"],
            ["85", "86", "87", "88"],
            ["86", "87", "88", "89"],
            ["87", "88", "89", "90"],
            ["88", "89", "90", "91"],
            ["89", "90", "91", "92"],
            ["90", "91", "92", "93"],
            ["91", "92", "93", "94"],
            ["92", "93", "94", "95"],
            ["93", "94", "95", "96"],
            ["94", "95", "96", "97"],
            ["95", "96", "97", "98"],
            ["96", "97", "98", "99"],
            ["97", "98", "99", "100"],
            ["98", "99", "100", "101"],
            ["99", "100", "101", "102"],
            ["100", "101", "102", "103"],
            ["101", "102", "103", "104"],
            ["102", "103", "104", "105"],
            ["103", "104", "105", "106"],
            ["104", "105", "106", "107"],
            ["105", "106", "107", "108"],
            ["106", "107", "108", "109"],
            ["107", "108", "109", "110"],
            ["108", "109", "110", "111"],
            ["109", "110", "111", "112"],
            ["110", "111", "112", "113"],
            ["111", "112", "113", "114"],
            ["112", "113", "114", "115"],
            ["113", "114", "115", "116"],
            ["114", "115", "116", "117"],
            ["115", "116", "117", "118"],
            ["116", "117", "118", "119"],
            ["117", "118", "119", "120"],
            ["118", "119", "120", "121"],
            ["119", "120", "121", "122"],
            ["120", "121", "122", "123"],
        ]
        .into_iter()
        .map(|v| {
            Row::new([
                Span::from(v[0]),
                Span::from(v[1]),
                Span::from(v[2]),
                Span::from(v[3]),
            ])
        }),
    )
    .widths([
        Constraint::Length(5),
        Constraint::Length(5),
        Constraint::Length(5),
        Constraint::Length(5),
    ])
}

fn focus_table(state: &FormScrolledTable) -> Focus<'_> {
    Focus::new([
        (state.table1.focus(), state.table1.area()),
        (state.table2.focus(), state.table2.area()),
    ])
}

fn handle_scrolled_table(
    event: &Event,
    data: &mut FormOneData,
    uistate: &mut FormOneState,
) -> Control {
    let state = &mut uistate.scrolled_table;

    focus_table(state)
        .append(focus_menu(&uistate.menu))
        .handle(event, DefaultKeys)
        .and_do(|_| uistate.g.repaint.set());

    check_break!(uistate.scrolled_table.table1.handle(event, DefaultKeys));
    check_break!(uistate.scrolled_table.table2.handle(event, DefaultKeys));

    Control::Continue
}

fn repaint_scrolled_list(
    event: &RepaintEvent,
    frame: &mut Frame<'_>,
    layout: FormOneAppLayout,
    data: &mut FormOneData,
    uistate: &mut FormOneState,
) -> Control {
    let l_columns = Layout::new(
        Direction::Horizontal,
        [
            Constraint::Fill(2),
            Constraint::Fill(1),
            Constraint::Fill(2),
            Constraint::Fill(1),
        ],
    )
    .split(layout.area);

    let mut nn = 0;
    let w_list1 = ListExt::default()
        .items(
            repeat_with(|| {
                nn += 1;
                nn
            })
            .take(5000)
            .map(|v| {
                ListItem::new(Text::from_iter([
                    Line::from(format!("item {}", v).to_string()),
                    Line::from("this is a second line for the item".to_string()),
                ]))
            }),
        )
        .styles(uistate.g.theme.list_style());
    let w_list1 = Scrolled::new(w_list1);

    frame.render_stateful_widget(w_list1, l_columns[0], &mut uistate.scrolled_list.list1);

    ControlUI::Continue
}

fn focus_list(state: &FormScrolledList) -> Focus<'_> {
    Focus::new([(state.list1.focus(), state.list1.area())])
}

fn handle_scrolled_list(
    event: &Event,
    data: &mut FormOneData,
    uistate: &mut FormOneState,
) -> Control {
    focus_list(&uistate.scrolled_list)
        .append(focus_menu(&uistate.menu))
        .handle(event, DefaultKeys)
        .and_do(|_| uistate.g.repaint.set());

    uistate.scrolled_list.list1.handle(event, DefaultKeys)
}

fn repaint_textarea(
    event: &RepaintEvent,
    frame: &mut Frame<'_>,
    layout: FormOneAppLayout,
    data: &mut FormOneData,
    uistate: &mut FormOneState,
) -> Control {
    let l_columns = Layout::new(
        Direction::Horizontal,
        [Constraint::Fill(5), Constraint::Fill(1)],
    )
    .split(Rect::new(
        layout.area.x,
        layout.area.y + 1,
        layout.area.width,
        layout.area.height - 1,
    ));

    let text_area = Scrolled::new(TextAreaExt::default());
    frame.render_stateful_widget(text_area, l_columns[0], &mut uistate.text_area.text);

    let l1 = layout_edit(
        l_columns[1],
        &[
            EditConstraint::EmptyRows(2),
            EditConstraint::Label("cur"),
            EditConstraint::Widget(20),
            EditConstraint::Label("rec"),
            EditConstraint::Widget(20),
            EditConstraint::Label("dim"),
            EditConstraint::Widget(20),
        ],
    );
    let mut l1 = l1.iter();

    frame.render_widget(Span::from("cur"), l1.label());
    frame.render_widget(
        Span::from(format!("{:?}", uistate.text_area.text.widget.widget.cursor()).to_string()),
        l1.widget(),
    );
    // TODO: rect() and dimensions() not implemented in the baseline
    // frame.render_widget(Span::from("rec"), l1.label());
    // frame.render_widget(
    //     Span::from(format!("{:?}", uistate.text_area.text.widget.widget.rect()).to_string()),
    //     l1.widget(),
    // );
    // frame.render_widget(Span::from("dim"), l1.label());
    // frame.render_widget(
    //     Span::from(format!("{:?}", uistate.text_area.text.widget.widget.dimensions()).to_string()),
    //     l1.widget(),
    // );

    ControlUI::Continue
}

fn focus_textarea(state: &FormTextArea) -> Focus<'_> {
    Focus::new([(state.text.focus(), state.text.area())])
}

fn handle_textarea(
    event: &Event,
    data: &mut FormOneData,
    uistate: &mut FormOneState,
) -> ControlUI<FormOneAction, anyhow::Error> {
    check_break!(focus_textarea(&uistate.text_area)
        .append(focus_menu(&uistate.menu))
        .handle(event, DefaultKeys)
        .and_then(|_| { ControlUI::Change })
        .map_err(|_| anyhow!("wtf")));

    uistate.text_area.text.handle(event, DefaultKeys)
}

fn repaint_tree(
    event: &RepaintEvent,
    frame: &mut Frame<'_>,
    layout: FormOneAppLayout,
    data: &mut FormOneData,
    uistate: &mut FormOneState,
) -> Control {
    let l_columns = Layout::new(
        Direction::Horizontal,
        [
            Constraint::Fill(2),
            Constraint::Fill(1),
            Constraint::Fill(2),
            Constraint::Fill(1),
        ],
    )
    .split(Rect::new(
        layout.area.x,
        layout.area.y + 1,
        layout.area.width,
        layout.area.height - 1,
    ));

    let tree = Scrolled::new(TreeExt::new(vec![
        tr!(TreeItem::new(
            1,
            "1",
            vec![
                TreeItem::new_leaf(10, "10"),
                TreeItem::new_leaf(11, "11"),
                TreeItem::new_leaf(12, "12"),
                TreeItem::new_leaf(13, "13"),
                TreeItem::new_leaf(14, "14"),
                TreeItem::new_leaf(15, "15"),
                TreeItem::new_leaf(16, "16"),
                TreeItem::new_leaf(17, "17"),
                TreeItem::new_leaf(18, "18"),
                TreeItem::new_leaf(19, "19"),
            ],
        )),
        tr!(TreeItem::new(
            2,
            "2",
            vec![
                TreeItem::new_leaf(20, "20"),
                TreeItem::new_leaf(21, "21"),
                TreeItem::new_leaf(22, "22"),
                TreeItem::new_leaf(23, "23"),
                TreeItem::new_leaf(24, "24"),
                TreeItem::new_leaf(25, "25"),
                TreeItem::new_leaf(26, "26"),
                TreeItem::new_leaf(27, "27"),
                TreeItem::new_leaf(28, "28"),
                TreeItem::new_leaf(29, "29"),
            ],
        )),
        tr!(TreeItem::new(
            3,
            "3",
            vec![
                TreeItem::new_leaf(30, "30"),
                TreeItem::new_leaf(31, "31"),
                TreeItem::new_leaf(32, "32"),
                TreeItem::new_leaf(33, "33"),
                TreeItem::new_leaf(34, "34"),
                TreeItem::new_leaf(35, "35"),
                TreeItem::new_leaf(36, "36"),
                TreeItem::new_leaf(37, "37"),
                TreeItem::new_leaf(38, "38"),
                TreeItem::new_leaf(39, "39"),
            ],
        )),
        tr!(TreeItem::new(
            4,
            "4",
            vec![
                TreeItem::new_leaf(40, "40"),
                TreeItem::new_leaf(41, "41"),
                TreeItem::new_leaf(42, "42"),
                TreeItem::new_leaf(43, "43"),
                TreeItem::new_leaf(44, "44"),
                TreeItem::new_leaf(45, "45"),
                TreeItem::new_leaf(46, "46"),
                TreeItem::new_leaf(47, "47"),
                TreeItem::new_leaf(48, "48"),
                TreeItem::new_leaf(49, "49"),
            ],
        )),
        tr!(TreeItem::new(
            5,
            "5",
            vec![
                TreeItem::new_leaf(50, "50"),
                TreeItem::new_leaf(51, "51"),
                TreeItem::new_leaf(52, "52"),
                TreeItem::new_leaf(53, "53"),
                TreeItem::new_leaf(54, "54"),
                TreeItem::new_leaf(55, "55"),
                TreeItem::new_leaf(56, "56"),
                TreeItem::new_leaf(57, "57"),
                TreeItem::new_leaf(58, "58"),
                TreeItem::new_leaf(59, "59"),
            ],
        )),
    ]));

    for i in 0..60 {
        uistate.scroll_other.tree.widget.widget.open(vec![i / 10]);
        uistate
            .scroll_other
            .tree
            .widget
            .widget
            .open(vec![i / 10, i]);
    }
    frame.render_stateful_widget(tree, l_columns[0], &mut uistate.scroll_other.tree);

    let w_chart = BarChart::default()
        .block(Block::default().title("BarChart").borders(Borders::ALL))
        .bar_width(3)
        .bar_gap(1)
        .group_gap(3)
        .bar_style(Style::new().yellow().on_red())
        .value_style(Style::new().red().bold())
        .label_style(Style::new().white())
        .data(&[("B0", 0), ("B1", 2), ("B2", 4), ("B3", 3)])
        .data(BarGroup::default().bars(&[Bar::default().value(10), Bar::default().value(20)]))
        .max(4);

    let v = Viewport {
        viewport_size: Default::default(),
        style: Default::default(),
        fill_char: ' ',
        widget: (),
        ..Default::default()
    };

    let vv = ViewportState {
        area: Default::default(),
        viewport_area: Default::default(),
        h_offset: 0,
        v_offset: 0,
        ..Default::default()
    };

    let w_chart = Scrolled::new(
        Viewport::new(w_chart)
            .viewport_size(Size::new(60, 60))
            .style(Style::default().fg(uistate.g.theme.red))
            .fill_char('∞'),
    )
    .h_overscroll(10)
    .v_overscroll(10);
    frame.render_stateful_widget(w_chart, l_columns[2], &mut uistate.scroll_other.chart);

    ControlUI::Continue
}

fn focus_tree(state: &FormOther) -> Focus<'_> {
    Focus::new([(state.tree.focus(), state.tree.area())])
}

fn handle_tree(
    event: &Event,
    data: &mut FormOneData,
    uistate: &mut FormOneState,
) -> ControlUI<FormOneAction, anyhow::Error> {
    check_break!(focus_tree(&uistate.scroll_other)
        .append(focus_menu(&uistate.menu))
        .handle(event, DefaultKeys)
        .and_then(|_| { ControlUI::Change })
        .map_err(|_| anyhow!("wtf")));

    check_break!(uistate.scroll_other.tree.handle(event, DefaultKeys));
    check_break!(uistate.scroll_other.chart.handle(event, DefaultKeys));

    ControlUI::Continue
}

// -----------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct Theme {
    pub name: &'static str,
    pub dark_theme: bool,

    pub white: Color,
    pub darker_black: Color,
    pub black: Color,
    pub black2: Color,
    pub one_bg: Color,
    pub one_bg2: Color,
    pub one_bg3: Color,
    pub grey: Color,
    pub grey_fg: Color,
    pub grey_fg2: Color,
    pub light_grey: Color,
    pub red: Color,
    pub baby_pink: Color,
    pub pink: Color,
    pub line: Color,
    pub green: Color,
    pub vibrant_green: Color,
    pub nord_blue: Color,
    pub blue: Color,
    pub yellow: Color,
    pub sun: Color,
    pub purple: Color,
    pub dark_purple: Color,
    pub teal: Color,
    pub orange: Color,
    pub cyan: Color,
    pub statusline_bg: Color,
    pub lightbg: Color,
    pub pmenu_bg: Color,
    pub folder_bg: Color,

    pub base00: Color,
    pub base01: Color,
    pub base02: Color,
    pub base03: Color,
    pub base04: Color,
    pub base05: Color,
    pub base06: Color,
    pub base07: Color,
    pub base08: Color,
    pub base09: Color,
    pub base0a: Color,
    pub base0b: Color,
    pub base0c: Color,
    pub base0d: Color,
    pub base0e: Color,
    pub base0f: Color,
}

impl Theme {
    pub fn status_style(&self) -> Style {
        Style::default().fg(self.white).bg(self.one_bg3)
    }

    pub fn list_style(&self) -> ListExtStyle {
        ListExtStyle {
            style: Default::default(),
            select_style: Style::default().fg(self.black).bg(self.purple).bold(),
            focus_style: Style::default().fg(self.black).bg(self.green).bold(),
            ..ListExtStyle::default()
        }
    }

    pub fn table_style(&self) -> TableExtStyle {
        TableExtStyle {
            style: Default::default(),
            select_style: Style::default().fg(self.black).bg(self.purple).bold(),
            focus_style: Style::default().fg(self.black).bg(self.green).bold(),
            ..TableExtStyle::default()
        }
    }

    pub fn input_style(&self) -> TextInputStyle {
        TextInputStyle {
            style: Style::default().fg(self.black).bg(self.base05),
            focus: Style::default().fg(self.black).bg(self.green),
            select: Style::default().fg(self.black).bg(self.base0e),
            invalid: Style::default().red().underlined(),
            ..TextInputStyle::default()
        }
    }

    pub fn input_mask_style(&self) -> MaskedInputStyle {
        MaskedInputStyle {
            style: Style::default().fg(self.black).bg(self.base05),
            focus: Style::default().fg(self.black).bg(self.green),
            select: Style::default().fg(self.black).bg(self.base0e),
            invalid: Style::default().red().underlined(),
            ..MaskedInputStyle::default()
        }
    }

    pub fn button_style(&self) -> ButtonStyle {
        ButtonStyle {
            style: Style::default().fg(self.black).bg(self.purple).bold(),
            focus: Style::default().fg(self.black).bg(self.green).bold(),
            armed: Style::default().fg(self.black).bg(self.orange).bold(),
        }
    }

    pub fn status_dialog_style(&self) -> StatusDialogStyle {
        StatusDialogStyle {
            style: self.status_style(),
            button: self.button_style(),
        }
    }

    pub fn menu_style(&self) -> MenuStyle {
        MenuStyle {
            style: Style::default().fg(self.white).bg(self.one_bg3).bold(),
            title: Style::default().fg(self.black).bg(self.base0a).bold(),
            select: Style::default().fg(self.black).bg(self.base0e).bold(),
            focus: Style::default().fg(self.black).bg(self.green).bold(),
        }
    }
}

pub static ONEDARK: Theme = Theme {
    name: "onedark",
    dark_theme: false,

    white: Color::from_u32(0xabb2bf),
    darker_black: Color::from_u32(0x1b1f27),
    black: Color::from_u32(0x1e222a), //  nvim bg
    black2: Color::from_u32(0x252931),
    one_bg: Color::from_u32(0x282c34), // real bg of onedark
    one_bg2: Color::from_u32(0x353b45),
    one_bg3: Color::from_u32(0x373b43),
    grey: Color::from_u32(0x42464e),
    grey_fg: Color::from_u32(0x565c64),
    grey_fg2: Color::from_u32(0x6f737b),
    light_grey: Color::from_u32(0x6f737b),
    red: Color::from_u32(0xe06c75),
    baby_pink: Color::from_u32(0xDE8C92),
    pink: Color::from_u32(0xff75a0),
    line: Color::from_u32(0x31353d), // for lines like vertsplit
    green: Color::from_u32(0x98c379),
    vibrant_green: Color::from_u32(0x7eca9c),
    nord_blue: Color::from_u32(0x81A1C1),
    blue: Color::from_u32(0x61afef),
    yellow: Color::from_u32(0xe7c787),
    sun: Color::from_u32(0xEBCB8B),
    purple: Color::from_u32(0xde98fd),
    dark_purple: Color::from_u32(0xc882e7),
    teal: Color::from_u32(0x519ABA),
    orange: Color::from_u32(0xfca2aa),
    cyan: Color::from_u32(0xa3b8ef),
    statusline_bg: Color::from_u32(0x22262e),
    lightbg: Color::from_u32(0x2d3139),
    pmenu_bg: Color::from_u32(0x61afef),
    folder_bg: Color::from_u32(0x61afef),

    base00: Color::from_u32(0x1e222a),
    base01: Color::from_u32(0x353b45),
    base02: Color::from_u32(0x3e4451),
    base03: Color::from_u32(0x545862),
    base04: Color::from_u32(0x565c64),
    base05: Color::from_u32(0xabb2bf),
    base06: Color::from_u32(0xb6bdca),
    base07: Color::from_u32(0xc8ccd4),
    base08: Color::from_u32(0xe06c75),
    base09: Color::from_u32(0xd19a66),
    base0a: Color::from_u32(0xe5c07b),
    base0b: Color::from_u32(0x98c379),
    base0c: Color::from_u32(0x56b6c2),
    base0d: Color::from_u32(0x61afef),
    base0e: Color::from_u32(0xc678dd),
    base0f: Color::from_u32(0xbe5046),
};

fn setup_logging() -> Result<(), anyhow::Error> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                //"[{} {} {}]\n        {}",
                "{}",
                // humantime::format_rfc3339_seconds(SystemTime::now()),
                // record.level(),
                // record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("log.log")?)
        .apply()?;
    Ok(())
}
