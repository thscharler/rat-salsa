#![allow(unused_variables)]
#![allow(unreachable_pub)]

use crate::facilities::{Facility, FileDlg};
use crate::mdedit::{MDEdit, MDEditState};
use anyhow::Error;
use crossterm::event::Event;
#[allow(unused_imports)]
use log::debug;
use markdown::mdast::Node;
use markdown::{to_mdast, unist, ParseOptions};
use rat_salsa::timer::TimeOut;
use rat_salsa::{run_tui, AppEvents, AppWidget, Control, RunConfig};
use rat_theme::dark_theme::DarkTheme;
use rat_theme::scheme::IMPERIAL;
use rat_widget::event::{
    ct_event, flow_ok, or_else, ConsumedEvent, Dialog, FocusKeys, HandleEvent, Popup,
};
use rat_widget::file_dialog::FileDialog;
use rat_widget::focus::{Focus, HasFocus, HasFocusFlag};
use rat_widget::layout::layout_middle;
use rat_widget::menubar::{MenuBar, MenuBarState, MenuPopup, StaticMenu};
use rat_widget::menuline::MenuOutcome;
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use rat_widget::popup_menu::Placement;
use rat_widget::statusline::{StatusLine, StatusLineState};
use rat_widget::textarea::core::TextRange;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{Block, StatefulWidget};
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

type AppContext<'a> = rat_salsa::AppContext<'a, GlobalState, MDAction, Error>;
type RenderContext<'a> = rat_salsa::RenderContext<'a, GlobalState>;

fn main() -> Result<(), Error> {
    setup_logging()?;

    let config = MDConfig::default();
    let theme = DarkTheme::new("Imperial".into(), IMPERIAL);
    let mut global = GlobalState::new(config, theme);

    let app = MDApp;
    let mut state = MDAppState::default();

    run_tui(
        app,
        &mut global,
        &mut state,
        RunConfig::default()?.threads(1),
    )?;

    Ok(())
}

// -----------------------------------------------------------------------

#[derive(Debug)]
pub struct GlobalState {
    pub cfg: MDConfig,
    pub theme: DarkTheme,
    pub status: RefCell<StatusLineState>,
    pub error_dlg: RefCell<MsgDialogState>,
}

impl GlobalState {
    fn new(cfg: MDConfig, theme: DarkTheme) -> Self {
        Self {
            cfg,
            theme,
            status: Default::default(),
            error_dlg: Default::default(),
        }
    }
}

// -----------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct MDConfig {}

#[derive(Debug)]
pub enum MDAction {
    Message(String),
    Open(PathBuf),
    Show(String, Vec<(TextRange, usize)>),
    Save(PathBuf),
}

// -----------------------------------------------------------------------

#[derive(Debug)]
pub struct MDApp;

#[derive(Debug)]
pub struct MDAppState {
    pub editor: MDEditState,
    pub menu: MenuBarState,

    pub filedlg: FileDlg,
}

impl Default for MDAppState {
    fn default() -> Self {
        let s = Self {
            editor: Default::default(),
            menu: Default::default(),
            filedlg: Default::default(),
        };
        s.menu.focus().set_name("menu");
        s.menu.bar.focus().set_name("menu_bar");
        s.editor.edit.focus.set_name("edit");
        s
    }
}

pub mod facilities {
    use crate::{AppContext, MDAction};
    use anyhow::Error;
    use crossterm::event::Event;
    use log::debug;
    use rat_salsa::event::flow_ok;
    use rat_salsa::Control;
    use rat_widget::event::{Dialog, FileOutcome, HandleEvent};
    use rat_widget::file_dialog::FileDialogState;
    use std::path::PathBuf;

    /// Multi purpose facility.
    pub trait Facility<T, O, A, E> {
        /// Engage with the facility.
        /// Setup its current workings and set a handler for any possible outcomes.
        fn engage(
            &mut self,
            init: impl FnOnce(&mut T) -> Result<Control<A>, E>,
            out: fn(O) -> Result<Control<A>, E>,
        ) -> Result<Control<A>, E>;

        /// Handle crossterm events for the facility.
        fn crossterm(&mut self, event: &Event, ctx: &mut AppContext<'_>) -> Result<Control<A>, E>;
    }

    #[derive(Debug, Default)]
    pub struct FileDlg {
        pub filedlg: FileDialogState,
        pub out: Option<fn(PathBuf) -> Result<Control<MDAction>, Error>>,
    }

    impl Facility<FileDialogState, PathBuf, MDAction, Error> for FileDlg {
        fn engage(
            &mut self,
            init: impl FnOnce(&mut FileDialogState) -> Result<Control<MDAction>, Error>,
            out: fn(PathBuf) -> Result<Control<MDAction>, Error>,
        ) -> Result<Control<MDAction>, Error> {
            let r = init(&mut self.filedlg);
            if r.is_ok() {
                self.out = Some(out);
            }
            r
        }

        fn crossterm(
            &mut self,
            event: &Event,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<MDAction>, Error> {
            flow_ok!(match self.filedlg.handle(event, Dialog)? {
                FileOutcome::Ok(path) => {
                    self.filedlg = Default::default();
                    if let Some(out) = self.out.take() {
                        ctx.queue(Control::Repaint);
                        out(path)?
                    } else {
                        Control::Repaint
                    }
                }
                FileOutcome::Cancel => {
                    self.filedlg = Default::default();
                    Control::Repaint
                }
                r => r.into(),
            });
            Ok(Control::Continue)
        }
    }
}

static MENU: StaticMenu = StaticMenu {
    menu: &[
        ("_File", &["_Open", "_Save"]), //
        ("_Quit", &[]),
    ],
};

impl AppWidget<GlobalState, MDAction, Error> for MDApp {
    type State = MDAppState;

    fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut Self::State,
        ctx: &mut RenderContext<'_>,
    ) -> Result<(), Error> {
        let t0 = SystemTime::now();

        let r = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);
        let s = Layout::horizontal([Constraint::Percentage(61), Constraint::Percentage(39)])
            .split(r[1]);

        MDEdit.render(r[0], buf, &mut state.editor, ctx)?;

        MenuBar::new()
            .styles(ctx.g.theme.menu_style())
            .menu(&MENU)
            .render(s[0], buf, &mut state.menu);

        let l_fd = layout_middle(
            r[0],
            Constraint::Length(state.menu.bar.item_areas[0].x),
            Constraint::Percentage(39),
            Constraint::Percentage(39),
            Constraint::Length(0),
        );
        FileDialog::new()
            .styles(ctx.g.theme.file_dialog_style())
            .render(l_fd, buf, &mut state.filedlg.filedlg);

        MenuPopup::new()
            .styles(ctx.g.theme.menu_style())
            .menu(&MENU)
            .width(20)
            .block(Block::bordered())
            .placement(Placement::Top)
            .render(s[0], buf, &mut state.menu);

        if ctx.g.error_dlg.borrow().active {
            let err = MsgDialog::new().styles(ctx.g.theme.msg_dialog_style());
            err.render(r[0], buf, &mut ctx.g.error_dlg.borrow_mut());
        }

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        debug!("render {:?} {:?}", (), el);
        ctx.g
            .status
            .borrow_mut()
            .status(1, format!("R {:.3?}", el).to_string());

        let status = StatusLine::new()
            .layout([
                Constraint::Fill(1),
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Length(12),
            ])
            .styles(ctx.g.theme.statusline_style());
        status.render(s[1], buf, &mut ctx.g.status.borrow_mut());

        Ok(())
    }
}

impl HasFocus for MDAppState {
    fn focus(&self) -> Focus {
        let mut f = Focus::default();
        f.add(&self.menu);
        f.add_container(&self.editor);
        f
    }
}

impl AppEvents<GlobalState, MDAction, Error> for MDAppState {
    fn init(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
        self.menu.focus().set(true);
        Ok(())
    }

    fn timer(
        &mut self,
        event: &TimeOut,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<MDAction>, Error> {
        let t0 = SystemTime::now();

        let r = self.editor.timer(event, ctx)?;

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        debug!("timer {:?} {:?}", r, el);
        ctx.g
            .status
            .borrow_mut()
            .status(3, format!("T {:.3?}", el).to_string());

        Ok(r)
    }

    fn crossterm(
        &mut self,
        event: &Event,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<MDAction>, Error> {
        let t0 = SystemTime::now();

        let mut r;
        r = match &event {
            Event::Resize(_, _) => Control::Repaint,
            ct_event!(key press CONTROL-'q') => Control::Quit,
            _ => Control::Continue,
        };
        or_else!(r, {
            if ctx.g.error_dlg.borrow().active {
                ctx.g.error_dlg.borrow_mut().handle(event, Dialog).into()
            } else {
                Control::Continue
            }
        });
        or_else!(r, self.filedlg.crossterm(event, ctx)?);

        // focus
        let mut focus = self.focus();
        let f = focus.handle(event, FocusKeys);
        ctx.focus = Some(focus);
        ctx.queue(f);

        or_else!(
            r,
            match self.menu.handle(event, Popup) {
                MenuOutcome::MenuActivated(0, 0) => self.filedlg.engage(
                    |w| {
                        w.open_dialog(".")?;
                        Ok(Control::Repaint)
                    },
                    |p| Ok(Control::Action(MDAction::Open(p))),
                )?,
                MenuOutcome::MenuActivated(0, 1) => self.filedlg.engage(
                    |w| {
                        w.save_dialog(".", "")?;
                        Ok(Control::Repaint)
                    },
                    |p| Ok(Control::Action(MDAction::Save(p))),
                )?,
                r => r.into(),
            }
        );
        or_else!(
            r,
            match self.menu.handle(event, FocusKeys) {
                MenuOutcome::Activated(1) => Control::Quit,
                r => r.into(),
            }
        );

        or_else!(r, self.editor.crossterm(event, ctx)?);

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        debug!("crossterm {:?} {:?}", r, el);
        ctx.g
            .status
            .borrow_mut()
            .status(2, format!("H {:.3?}", el).to_string());

        Ok(r)
    }

    fn action(
        &mut self,
        event: &mut MDAction,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<MDAction>, Error> {
        let t0 = SystemTime::now();

        let mut r;
        r = match event {
            MDAction::Message(s) => {
                ctx.g.status.borrow_mut().status(0, &*s);
                Control::Repaint
            }
            _ => Control::Continue,
        };

        ctx.focus = Some(self.focus());

        // TODO: actions

        or_else!(r, self.editor.action(event, ctx)?);

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        debug!("action {:?} {:?}", r, el);
        ctx.g
            .status
            .borrow_mut()
            .status(3, format!("A {:.3?}", el).to_string());

        Ok(r)
    }

    fn error(&self, event: Error, ctx: &mut AppContext<'_>) -> Result<Control<MDAction>, Error> {
        ctx.g
            .error_dlg
            .borrow_mut()
            .append(format!("{:?}", &*event).as_str());
        Ok(Control::Repaint)
    }
}

mod mdedit {
    use crate::{collect_ast, AppContext, GlobalState, MDAction, RenderContext};
    use anyhow::Error;
    use crossterm::event::Event;
    #[allow(unused_imports)]
    use log::debug;
    use rat_salsa::event::flow_ok;
    use rat_salsa::timer::{TimeOut, TimerDef, TimerHandle};
    use rat_salsa::{AppEvents, AppWidget, Control};
    use rat_widget::event::{FocusKeys, HandleEvent, TextOutcome};
    use rat_widget::focus::{Focus, HasFocus};
    use rat_widget::scrolled::Scroll;
    use rat_widget::textarea::{TextArea, TextAreaState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::{Style, Stylize};
    use ratatui::widgets::StatefulWidget;
    use std::time::{Duration, Instant};
    use std::{fs, mem};

    #[derive(Debug)]
    pub struct MDEdit;

    #[derive(Debug)]
    pub struct MDEditState {
        pub edit: TextAreaState,
        pub parse_timer: Option<TimerHandle>,
    }

    impl Default for MDEditState {
        fn default() -> Self {
            let s = Self {
                edit: Default::default(),
                parse_timer: None,
            };
            s
        }
    }

    impl HasFocus for MDEditState {
        fn focus(&self) -> Focus {
            Focus::new(&[&self.edit])
        }
    }

    impl AppWidget<GlobalState, MDAction, Error> for MDEdit {
        type State = MDEditState;

        fn render(
            &self,
            area: Rect,
            buf: &mut Buffer,
            state: &mut Self::State,
            ctx: &mut RenderContext<'_>,
        ) -> Result<(), Error> {
            TextArea::new()
                .styles(ctx.g.theme.textarea_style())
                .set_horizontal_max_offset(255)
                .vscroll(Scroll::new().styles(ctx.g.theme.scroll_style()))
                .text_style([
                    Style::default().fg(ctx.g.theme.scheme().green[2]), // EMPHASIS
                    Style::default()
                        .fg(ctx.g.theme.scheme().yellow[2])
                        .underlined(), // HEADER
                    Style::default().fg(ctx.g.theme.scheme().bluegreen[2]), // LINK
                    Style::default().fg(ctx.g.theme.scheme().gray[2]),  // CODE
                ])
                .render(area, buf, &mut state.edit);
            ctx.cursor = state.edit.screen_cursor();
            Ok(())
        }
    }

    impl MDEditState {
        pub fn parse_markdown(&mut self) {
            self.edit.clear_styles();
            let styles = collect_ast(self.edit.value().as_str());
            for (r, s) in styles {
                self.edit.add_style(r, s);
            }
        }
    }

    impl AppEvents<GlobalState, MDAction, Error> for MDEditState {
        fn timer(
            &mut self,
            event: &TimeOut,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, MDAction, Error>,
        ) -> Result<Control<MDAction>, Error> {
            if let Some(parse_timer) = &self.parse_timer {
                if event.tag == *parse_timer {
                    self.parse_markdown();
                    Ok(Control::Repaint)
                } else {
                    Ok(Control::Continue)
                }
            } else {
                Ok(Control::Continue)
            }
        }

        fn crossterm(
            &mut self,
            event: &Event,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<MDAction>, Error> {
            flow_ok!(match self.edit.handle(event, FocusKeys) {
                TextOutcome::TextChanged => {
                    // restart timer
                    if let Some(h) = self.parse_timer.take() {
                        ctx.remove_timer(h);
                    }
                    // one-shot timer
                    let h = ctx.add_timer(
                        TimerDef::new().next(Instant::now() + Duration::from_millis(500)),
                    );
                    self.parse_timer = Some(h);

                    Control::Repaint
                }
                r => r.into(),
            });
            Ok(Control::Continue)
        }

        fn action(
            &mut self,
            event: &mut MDAction,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, MDAction, Error>,
        ) -> Result<Control<MDAction>, Error> {
            flow_ok!(match event {
                MDAction::Open(p) => {
                    debug!("load");
                    let t = fs::read_to_string(p)?;
                    let t = t.replace("\r\n", "\n"); // todo: better?!
                    self.edit.set_value(t.as_str());
                    self.parse_timer = Some(ctx.add_timer(
                        TimerDef::new().next(Instant::now() + Duration::from_millis(100)),
                    ));
                    ctx.focus.as_ref().expect("focus").focus_widget(&self.edit);
                    Control::Repaint
                }
                MDAction::Save(_) => {
                    Control::Repaint
                }
                _ => Control::Continue,
            });
            Ok(Control::Continue)
        }
    }
}

fn tr(pos: &unist::Position) -> TextRange {
    TextRange::new(
        (pos.start.column - 1, pos.start.line - 1),
        (pos.end.column - 1, pos.end.line - 1),
    )
}

const EMPHASIS: usize = 0;
const HEADER: usize = 1;
const LINK: usize = 2;
const CODE: usize = 3;

fn collect_ast(t: &str) -> Vec<(TextRange, usize)> {
    let mut styles = Vec::new();

    let ast = to_mdast(t, &ParseOptions::default()).expect("ast");
    collect_ast_rec(&ast, &mut styles);

    styles
}

fn collect_ast_rec(node: &Node, styles: &mut Vec<(TextRange, usize)>) {
    match node {
        Node::Root(_) => {}
        Node::BlockQuote(_) => {}
        Node::FootnoteDefinition(_) => {}
        Node::MdxJsxFlowElement(_) => {}
        Node::List(_) => {}
        Node::MdxjsEsm(_) => {}
        Node::Toml(_) => {}
        Node::Yaml(_) => {}
        Node::Break(_) => {}
        Node::InlineCode(v) => {}
        Node::InlineMath(v) => {}
        Node::Delete(v) => {}
        Node::Emphasis(v) => {
            if let Some(pos) = &v.position {
                styles.push((tr(pos), EMPHASIS));
            }
        }
        Node::MdxTextExpression(_) => {}
        Node::FootnoteReference(_) => {}
        Node::Html(_) => {}
        Node::Image(_) => {}
        Node::ImageReference(_) => {}
        Node::MdxJsxTextElement(_) => {}
        Node::Link(v) => {
            if let Some(pos) = &v.position {
                styles.push((tr(pos), LINK));
            }
        }
        Node::LinkReference(v) => {
            if let Some(pos) = &v.position {
                styles.push((tr(pos), LINK));
            }
        }
        Node::Strong(_) => {}
        Node::Text(_) => {}
        Node::Code(v) => {
            if let Some(pos) = &v.position {
                styles.push((tr(pos), CODE));
            }
        }
        Node::Math(_) => {}
        Node::MdxFlowExpression(_) => {}
        Node::Heading(v) => {
            if let Some(pos) = &v.position {
                styles.push((tr(pos), HEADER));
            }
        }
        Node::Table(_) => {}
        Node::ThematicBreak(_) => {}
        Node::TableRow(_) => {}
        Node::TableCell(_) => {}
        Node::ListItem(_) => {}
        Node::Definition(_) => {}
        Node::Paragraph(_) => {}
    }

    if let Some(children) = node.children() {
        for c in children {
            collect_ast_rec(c, styles);
        }
    }
}

fn setup_logging() -> Result<(), Error> {
    _ = fs::remove_file("log.log");
    fern::Dispatch::new()
        .format(|out, message, _record| out.finish(format_args!("{}", message)))
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("log.log")?)
        .apply()?;
    Ok(())
}
