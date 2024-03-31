#![allow(unused_variables)]
#![allow(clippy::needless_update)]

use crossterm::event::Event;
use rat_salsa::layout::{layout_edit, EditConstraint};
use rat_salsa::number::NumberSymbols;
use rat_salsa::widget::button::ButtonStyle;
use rat_salsa::widget::input::{TextInput, TextInputState, TextInputStyle};
use rat_salsa::widget::mask_input::{MaskedInput, MaskedInputState, MaskedInputStyle};
use rat_salsa::widget::menuline::{MenuLine, MenuLineState, MenuStyle};
use rat_salsa::widget::message::{
    StatusDialog, StatusDialogState, StatusDialogStyle, StatusLine, StatusLineState,
};
use rat_salsa::{
    check_break, for_focus, run_tui, try_ui, ControlUI, DefaultKeys, Focus, HandleCrossterm,
    HasFocusFlag, RenderFrameWidget, Repaint, RepaintEvent, RunConfig, TaskSender, ThreadPool,
    TimerEvent, Timers, TuiApp,
};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Style};
use ratatui::style::Stylize;
use ratatui::text::Span;
use ratatui::Frame;
use std::fs;
use std::rc::Rc;

fn main() -> Result<(), anyhow::Error> {
    _ = fs::remove_file("log.log");

    setup_logging()?;

    let sym = Rc::new(NumberSymbols {
        decimal_sep: ",".to_string(),
        decimal_grp: ".".to_string(),
        exponent_upper_sym: "E".to_string(),
        exponent_lower_sym: "e".to_string(),
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
    pub mask0: Mask0,
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
pub struct Mask0 {
    pub menu: MenuLineState<u16>,

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

impl FormOneState {
    pub fn new(sym: &Rc<NumberSymbols>) -> Self {
        Self {
            g: GeneralState::new(),
            mask0: Mask0::new(sym),
        }
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

impl Mask0 {
    pub fn new(sym: &Rc<NumberSymbols>) -> Self {
        let mut s = Self {
            menu: MenuLineState::default(),
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
        s.menu.focus.set();
        s.text.focus.set();
        s.ipv4.set_mask("999\\.999\\.999\\.999");
        // s.ipv4.set_display_mask("xxx.xxx.xxx.xxx");
        s.hexcolor.set_mask("HHHHHH");
        s.creditcard.set_mask("dddd dddd dddd dddd");
        // s.creditcard.set_display_mask("dddd dddd dddd dddd");
        s.date.set_mask("99/99/9999");
        s.date.set_display_mask("mm/dd/yyyy");
        s.alpha.set_mask("llllllllll");
        s.dec7_2.set_mask("#,###,##0.00");
        s.euro.set_mask("€ ###,##0.00-");
        s.exp.set_mask("#.#######e###");
        s
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

        try_ui!(repaint_mask0(&event, frame, layout, data, uistate), _);

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
        event: TimerEvent,
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

        check_break!(handle_mask0(&event, data, uistate));

        Control::Continue
    }

    fn run_action(
        &self,
        action: Self::Action,
        data: &mut Self::Data,
        uistate: &mut Self::State,
        worker: &ThreadPool<Self>,
    ) -> ControlUI<Self::Action, Self::Error> {
        // TODO: actions
        Control::Continue
    }

    fn run_task(
        &self,
        task: Self::Action,
        send: &TaskSender<Self>,
    ) -> ControlUI<Self::Action, Self::Error> {
        // TODO: tasks
        Control::Continue
    }

    fn report_error(
        &self,
        error: Self::Error,
        data: &mut Self::Data,
        uistate: &mut Self::State,
    ) -> ControlUI<Self::Action, Self::Error> {
        uistate.g.error_dlg.log(format!("{:?}", &*error).as_str());
        Control::Change
    }
}

fn repaint_mask0(
    event: &RepaintEvent,
    frame: &mut Frame<'_>,
    layout: FormOneAppLayout,
    data: &mut FormOneData,
    uistate: &mut FormOneState,
) -> Control {
    // TODO: repaint_mask
    let l_columns = Layout::new(
        Direction::Horizontal,
        [Constraint::Length(33), Constraint::Length(33)],
    )
    .split(layout.area);

    let l = layout_edit(
        l_columns[0],
        [
            EditConstraint::Label("Text"),
            EditConstraint::Widget(20),
            EditConstraint::Label("Integer"),
            EditConstraint::Widget(12),
            EditConstraint::Label("Float"),
            EditConstraint::Widget(12),
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

    let w_text = TextInput::default().style(uistate.g.theme.input_style());
    let w_decimal = TextInput::default().style(uistate.g.theme.input_style());
    let w_float = TextInput::default().style(uistate.g.theme.input_style());

    let w_color = MaskedInput::default().style(uistate.g.theme.input_mask_style());
    let w_ipv4 = MaskedInput::default().style(uistate.g.theme.input_mask_style());
    let w_creditcard = MaskedInput::default().style(uistate.g.theme.input_mask_style());
    let w_date = MaskedInput::default().style(uistate.g.theme.input_mask_style());
    let w_name = MaskedInput::default().style(uistate.g.theme.input_mask_style());
    let w_dec_7_2 = MaskedInput::default().style(uistate.g.theme.input_mask_style());
    let w_euro = MaskedInput::default().style(uistate.g.theme.input_mask_style());
    let w_exp = MaskedInput::default().style(uistate.g.theme.input_mask_style());

    frame.render_widget(Span::from("Text"), l.label[0]);
    frame.render_frame_widget(w_text, l.widget[0], &mut uistate.mask0.text);
    frame.render_widget(Span::from("Integer"), l.label[1]);
    frame.render_frame_widget(w_decimal, l.widget[1], &mut uistate.mask0.decimal);
    frame.render_widget(Span::from("Float"), l.label[2]);
    frame.render_frame_widget(w_float, l.widget[2], &mut uistate.mask0.float);
    frame.render_widget(Span::from("IPv4"), l.label[3]);
    frame.render_frame_widget(w_ipv4, l.widget[3], &mut uistate.mask0.ipv4);
    frame.render_widget(Span::from("Color"), l.label[4]);
    frame.render_frame_widget(w_color, l.widget[4], &mut uistate.mask0.hexcolor);
    frame.render_widget(Span::from("Credit card"), l.label[5]);
    frame.render_frame_widget(w_creditcard, l.widget[5], &mut uistate.mask0.creditcard);
    frame.render_widget(Span::from("Date"), l.label[6]);
    frame.render_frame_widget(w_date, l.widget[6], &mut uistate.mask0.date);
    frame.render_widget(Span::from("Name"), l.label[7]);
    frame.render_frame_widget(w_name, l.widget[7], &mut uistate.mask0.alpha);
    frame.render_widget(Span::from("Decimal 7.2"), l.label[8]);
    frame.render_frame_widget(w_dec_7_2, l.widget[8], &mut uistate.mask0.dec7_2);
    frame.render_widget(Span::from("Euro"), l.label[9]);
    frame.render_frame_widget(w_euro, l.widget[9], &mut uistate.mask0.euro);
    frame.render_widget(Span::from("Exp"), l.label[10]);
    frame.render_frame_widget(w_exp, l.widget[10], &mut uistate.mask0.exp);

    let r = for_focus!(
        uistate.mask0.ipv4 => &uistate.mask0.ipv4,
        uistate.mask0.hexcolor => &uistate.mask0.hexcolor,
        uistate.mask0.creditcard => &uistate.mask0.creditcard,
        uistate.mask0.date => &uistate.mask0.date,
        uistate.mask0.alpha => &uistate.mask0.alpha,
        uistate.mask0.dec7_2 => &uistate.mask0.dec7_2,
        uistate.mask0.euro => &uistate.mask0.euro,
        uistate.mask0.exp => &uistate.mask0.exp
    );
    if let Some(r) = r {
        let mut area = l_columns[1];
        area.height = 1;

        for (i, t) in r.value.tokens().iter().enumerate() {
            let mut w_info = Span::from(format!(
                "#{}:{}:{}-{}   {} | {}",
                t.nr_id, t.sec_id, t.sec_start, t.sec_end, t.peek_left, t.right
            ));
            if i == r.cursor() {
                w_info = w_info.on_cyan();
            }
            frame.render_widget(w_info, area);
            area.y += 1;
        }
        frame.render_widget(Span::from(format!("value={}", r.value())), area);
        area.y += 1;
        frame.render_widget(Span::from(format!("compact={}", r.compact_value())), area);
        area.y += 1;
        frame.render_widget(
            Span::from(format!(
                "parse={:?}",
                r.compact_value().as_str().parse::<f64>()
            )),
            area,
        );
        area.y += 1;
        frame.render_widget(Span::from(format!("mask={}", r.mask())), area);
        area.y += 1;
        frame.render_widget(Span::from(format!("display={}", r.display_mask())), area);
        area.y += 1;
        frame.render_widget(
            Span::from(format!(
                "{}:{} {} {}:{}",
                r.offset(),
                r.width(),
                r.cursor(),
                r.selection().start,
                r.selection().end
            )),
            area,
        );
    }

    let menu = MenuLine::new()
        .style(uistate.g.theme.menu_style())
        .add("_Quit", 0u16);
    frame.render_stateful_widget(menu, layout.menu, &mut uistate.mask0.menu);

    Control::Continue
}

fn focus0(mask0: &Mask0) -> Focus<'_> {
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

fn handle_mask0(event: &Event, data: &mut FormOneData, uistate: &mut FormOneState) -> Control {
    let mask0 = &mut uistate.mask0;

    focus0(mask0)
        .handle(event, DefaultKeys)
        .and_do(|_| uistate.g.repaint.set());

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

    check_break!(mask0.menu.handle(event, DefaultKeys).and_then(|a| match a {
        0 => {
            Control::Break
        }
        _ => Control::Continue,
    }));

    Control::Continue
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

    pub fn input_style(&self) -> TextInputStyle {
        TextInputStyle {
            style: Style::default().fg(self.black).bg(self.base05),
            focus: Style::default().fg(self.black).bg(self.green),
            select: Style::default().fg(self.black).bg(self.base0e),
            cursor: None,
            ..TextInputStyle::default()
        }
    }

    pub fn input_mask_style(&self) -> MaskedInputStyle {
        MaskedInputStyle {
            style: Style::default().fg(self.black).bg(self.base05),
            focus: Style::default().fg(self.black).bg(self.green),
            select: Style::default().fg(self.black).bg(self.base0e),
            cursor: None,
            invalid: Some(Style::default().fg(Color::White).bg(Color::Red)),
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
