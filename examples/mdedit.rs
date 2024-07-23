#![allow(unused_variables)]
#![allow(unreachable_pub)]

use crate::facilities::{Facility, MDFileDialog, MDFileDialogState};
use crate::mdedit::{MDEdit, MDEditState};
use anyhow::Error;
#[allow(unused_imports)]
use log::debug;
use pulldown_cmark::{Event, Options, Parser, Tag};
use rat_salsa::timer::TimeOut;
use rat_salsa::{run_tui, AppEvents, AppWidget, Control, RunConfig};
use rat_theme::dark_theme::DarkTheme;
use rat_theme::scheme::IMPERIAL;
use rat_theme::{dark_themes, Scheme};
use rat_widget::event::{ct_event, or_else, Dialog, HandleEvent, Popup, Regular};
use rat_widget::focus::{Focus, HasFocus, HasFocusFlag};
use rat_widget::layout::layout_middle;
use rat_widget::menubar::{MenuBarState, MenuStructure, Menubar, StaticMenu};
use rat_widget::menuline::MenuOutcome;
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use rat_widget::popup_menu::Placement;
use rat_widget::statusline::{StatusLine, StatusLineState};
use rat_widget::text::textarea_core::TextRange;
use rat_widget::textarea::TextAreaState;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::Line;
use ratatui::widgets::{Block, StatefulWidget};
use std::fs;
use std::ops::Range;
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

    pub status: StatusLineState,
    pub error_dlg: MsgDialogState,
    pub file_dlg: MDFileDialogState,
}

impl GlobalState {
    fn new(cfg: MDConfig, theme: DarkTheme) -> Self {
        Self {
            cfg,
            theme,
            status: Default::default(),
            error_dlg: Default::default(),
            file_dlg: Default::default(),
        }
    }

    fn theme(&self) -> &DarkTheme {
        &self.theme
    }

    fn scheme(&self) -> &Scheme {
        self.theme.scheme()
    }
}

// -----------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct MDConfig {
    // ...
}

#[derive(Debug)]
pub enum MDAction {
    Message(String),
    MenuOpen,
    MenuSave,
    Open(PathBuf),
    Show(String, Vec<(TextRange, usize)>),
    SaveAs(PathBuf),
    Save(),
}

// -----------------------------------------------------------------------

#[derive(Debug)]
pub struct MDApp;

#[derive(Debug)]
pub struct MDAppState {
    pub editor: MDEditState,
    pub menu: MenuBarState,
}

impl Default for MDAppState {
    fn default() -> Self {
        let s = Self {
            editor: Default::default(),
            menu: Default::default(),
        };
        s.menu.focus().set_name("menu");
        s.menu.bar.focus().set_name("menu_bar");
        s.editor.edit.focus.set_name("edit");
        s
    }
}

pub mod facilities {
    use crate::MDAction;
    use anyhow::Error;
    use crossterm::event::Event;
    use rat_salsa::event::flow_ok;
    use rat_salsa::Control;
    use rat_widget::event::{Dialog, FileOutcome, HandleEvent};
    use rat_widget::file_dialog::{FileDialog, FileDialogState, FileDialogStyle};
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::prelude::StatefulWidget;
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
        fn handle(&mut self, event: &Event) -> Result<Control<A>, E>;
    }

    #[derive(Debug, Default)]
    pub struct MDFileDialog {
        style: FileDialogStyle,
    }

    impl MDFileDialog {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn style(mut self, style: FileDialogStyle) -> Self {
            self.style = style;
            self
        }
    }

    impl StatefulWidget for MDFileDialog {
        type State = MDFileDialogState;

        fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
            FileDialog::new()
                .styles(self.style)
                .render(area, buf, &mut state.file_dlg);
        }
    }

    #[derive(Debug, Default)]
    pub struct MDFileDialogState {
        pub file_dlg: FileDialogState,
        pub handle: Option<fn(PathBuf) -> Result<Control<MDAction>, Error>>,
    }

    impl Facility<FileDialogState, PathBuf, MDAction, Error> for MDFileDialogState {
        fn engage(
            &mut self,
            prepare: impl FnOnce(&mut FileDialogState) -> Result<Control<MDAction>, Error>,
            handle: fn(PathBuf) -> Result<Control<MDAction>, Error>,
        ) -> Result<Control<MDAction>, Error> {
            let r = prepare(&mut self.file_dlg);
            if r.is_ok() {
                self.handle = Some(handle);
            }
            r
        }

        fn handle(&mut self, event: &Event) -> Result<Control<MDAction>, Error> {
            flow_ok!(match self.file_dlg.handle(event, Dialog)? {
                FileOutcome::Ok(path) => {
                    if let Some(handle) = self.handle.take() {
                        handle(path)?
                    } else {
                        Control::Repaint
                    }
                }
                FileOutcome::Cancel => {
                    Control::Repaint
                }
                r => r.into(),
            });
            Ok(Control::Continue)
        }
    }

    impl MDFileDialogState {
        pub fn screen_cursor(&self) -> Option<(u16, u16)> {
            self.file_dlg.screen_cursor()
        }
    }
}

static MENU: StaticMenu = StaticMenu {
    menu: &[
        ("_File", &["_Open", "Save _as", "_Save"]), //
        ("_View", &[/*dynamic*/]),
        ("_Theme", &[/*dynamic*/]),
        ("_Quit", &[]),
    ],
};

#[derive(Debug)]
struct Menu {
    show_ctrl: bool,
    use_crlf: bool,
}

impl<'a> MenuStructure<'a> for Menu {
    fn menus(&'a self) -> Vec<(Line<'a>, Option<char>)> {
        MENU.menus()
    }

    fn submenu(&'a self, n: usize) -> Vec<(Line<'a>, Option<char>)> {
        match n {
            1 => {
                vec![
                    if self.show_ctrl {
                        ("\u{2611} Control chars".into(), None)
                    } else {
                        ("\u{2610} Control chars".into(), None)
                    },
                    if self.use_crlf {
                        ("\u{2611} Use CR+LF".into(), None)
                    } else {
                        ("\u{2611} Use CR+LF".into(), None)
                    },
                ]
            }
            2 => rat_theme::dark_themes()
                .iter()
                .map(|v| (v.name().to_string().into(), None))
                .collect(),
            _ => MENU.submenu(n),
        }
    }
}

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

        let menu_struct = Menu {
            show_ctrl: state.editor.show_ctrl,
            use_crlf: state.editor.edit.newline() == "\r\n",
        };
        let (menu, menu_popup) = Menubar::new(&menu_struct)
            .title("^^°n°^^")
            .popup_width(20)
            .popup_block(Block::bordered())
            .popup_placement(Placement::Top)
            .styles(ctx.g.theme.menu_style())
            .into_widgets();
        menu.render(s[0], buf, &mut state.menu);

        let l_fd = layout_middle(
            r[0],
            Constraint::Length(state.menu.bar.item_areas[0].x),
            Constraint::Percentage(39),
            Constraint::Percentage(39),
            Constraint::Length(0),
        );
        MDFileDialog::new()
            .style(ctx.g.theme.file_dialog_style())
            .render(l_fd, buf, &mut ctx.g.file_dlg);
        ctx.set_screen_cursor(ctx.g.file_dlg.screen_cursor());

        menu_popup.render(s[0], buf, &mut state.menu);

        if ctx.g.error_dlg.active() {
            let err = MsgDialog::new().styles(ctx.g.theme.msg_dialog_style());
            err.render(r[0], buf, &mut ctx.g.error_dlg);
        }

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        // debug!("render {:?} {:?}", (), el);
        ctx.g.status.status(2, format!("R {:.3?}", el).to_string());

        let status = StatusLine::new()
            .layout([
                Constraint::Fill(1),
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Length(12),
            ])
            .styles(ctx.g.theme.statusline_style());
        status.render(s[1], buf, &mut ctx.g.status);

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
        // debug!("timer {:?} {:?}", r, el);
        ctx.g.status.status(3, format!("T {:.3?}", el).to_string());

        Ok(r)
    }

    fn crossterm(
        &mut self,
        event: &crossterm::event::Event,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<MDAction>, Error> {
        let t0 = SystemTime::now();

        let mut r;
        r = match &event {
            ct_event!(resized) => Control::Repaint,
            ct_event!(key press CONTROL-'q') => Control::Quit,
            _ => Control::Continue,
        };
        or_else!(r, ctx.g.error_dlg.handle(event, Dialog).into());
        or_else!(r, ctx.g.file_dlg.handle(event)?);

        // focus
        let mut focus = self.focus();
        let f = focus.handle(event, Regular);
        ctx.focus = Some(focus);
        ctx.queue(f);

        or_else!(
            r,
            match self.menu.handle(event, Popup) {
                MenuOutcome::MenuActivated(0, 0) => Control::Action(MDAction::MenuOpen),
                MenuOutcome::MenuActivated(0, 1) => Control::Action(MDAction::MenuSave),
                MenuOutcome::MenuActivated(1, 0) => {
                    self.editor.show_ctrl = !self.editor.show_ctrl;
                    Control::Repaint
                }
                MenuOutcome::MenuActivated(1, 1) => {
                    if self.editor.edit.newline() == "\r\n" {
                        self.editor.edit.set_newline("\n");
                    } else {
                        self.editor.edit.set_newline("\r\n");
                    }
                    Control::Repaint
                }
                MenuOutcome::MenuSelected(2, n) => {
                    ctx.g.theme = dark_themes()[n].clone();
                    Control::Repaint
                }
                r => r.into(),
            }
        );

        or_else!(
            r,
            match self.menu.handle(event, Regular) {
                MenuOutcome::Activated(3) => Control::Quit,
                r => r.into(),
            }
        );
        or_else!(r, self.editor.crossterm(event, ctx)?);

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        // debug!("crossterm {:?} {:?}", r, el);
        ctx.g.status.status(3, format!("H {:.3?}", el).to_string());

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
                ctx.g.status.status(0, &*s);
                Control::Repaint
            }
            _ => Control::Continue,
        };

        ctx.focus = Some(self.focus());

        // TODO: actions
        or_else!(r, self.editor.action(event, ctx)?);

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        // debug!("action {:?} {:?}", r, el);
        ctx.g.status.status(3, format!("A {:.3?}", el).to_string());

        Ok(r)
    }

    fn error(&self, event: Error, ctx: &mut AppContext<'_>) -> Result<Control<MDAction>, Error> {
        ctx.g.error_dlg.append(format!("{:?}", &*event).as_str());
        Ok(Control::Repaint)
    }
}

pub mod mdedit {
    use crate::facilities::Facility;
    use crate::{collect_ast, AppContext, GlobalState, MDAction, RenderContext};
    use anyhow::{anyhow, Error};
    use crossterm::event::Event;
    #[allow(unused_imports)]
    use log::debug;
    use rat_salsa::event::{ct_event, flow_ok};
    use rat_salsa::timer::{TimeOut, TimerDef, TimerHandle};
    use rat_salsa::{AppEvents, AppWidget, Control};
    use rat_widget::event::{HandleEvent, Regular, TextOutcome};
    use rat_widget::focus::{Focus, HasFocus};
    use rat_widget::scrolled::Scroll;
    use rat_widget::textarea::{TextArea, TextAreaState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::{Modifier, Style, Stylize};
    use ratatui::widgets::StatefulWidget;
    use std::fs;
    use std::fs::File;
    use std::io::{BufWriter, Write};
    use std::path::PathBuf;
    use std::time::{Duration, Instant};

    #[derive(Debug)]
    pub struct MDEdit;

    #[derive(Debug)]
    pub struct MDEditState {
        pub show_ctrl: bool,
        pub path: Option<PathBuf>,
        pub edit: TextAreaState,
        pub parse_timer: Option<TimerHandle>,
    }

    impl Default for MDEditState {
        fn default() -> Self {
            let mut s = Self {
                show_ctrl: false,
                path: None,
                edit: Default::default(),
                parse_timer: None,
            };
            // s.edit.set_expand_tabs(false);
            s
        }
    }

    impl HasFocus for MDEditState {
        fn focus(&self) -> Focus {
            Focus::new(&[&self.edit])
        }
    }

    impl MDEdit {
        fn text_style(&self, ctx: &mut RenderContext<'_>) -> [Style; 17] {
            [
                Style::default().fg(ctx.g.scheme().yellow[2]).underlined(), // Heading,
                Style::default().fg(ctx.g.scheme().yellow[1]),              // BlockQuote,
                Style::default().fg(ctx.g.scheme().redpink[2]),             // CodeBlock,
                Style::default().fg(ctx.g.scheme().bluegreen[3]),           // FootnodeDefinition
                Style::default().fg(ctx.g.scheme().bluegreen[2]),           // FootnodeReference
                Style::default().fg(ctx.g.scheme().orange[2]),              // Item
                Style::default()
                    .fg(ctx.g.scheme().white[3])
                    .add_modifier(Modifier::ITALIC), // Emphasis
                Style::default().fg(ctx.g.scheme().white[3]),               // Strong
                Style::default().fg(ctx.g.scheme().gray[2]),                // Strikethrough
                Style::default().fg(ctx.g.scheme().bluegreen[2]),           // Link
                Style::default().fg(ctx.g.scheme().bluegreen[2]),           // Image
                Style::default().fg(ctx.g.scheme().orange[1]),              // MetadataBlock
                Style::default().fg(ctx.g.scheme().redpink[2]),             // CodeInline
                Style::default().fg(ctx.g.scheme().redpink[2]),             // MathInline
                Style::default().fg(ctx.g.scheme().redpink[2]),             // MathDisplay
                Style::default().fg(ctx.g.scheme().white[3]),               // Rule
                Style::default().fg(ctx.g.scheme().orange[2]),              // TaskListMarker
            ]
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
                // .set_horizontal_max_offset(255)
                // .set_horizontal_overscroll(16384)
                .vscroll(Scroll::new().styles(ctx.g.theme.scroll_style()))
                .show_ctrl(state.show_ctrl)
                .text_style(self.text_style(ctx))
                .render(area, buf, &mut state.edit);
            ctx.set_screen_cursor(state.edit.screen_cursor());

            let cursor = state.edit.cursor();
            ctx.g.status.status(
                1,
                format!(
                    "{}|{}+{}",
                    cursor.0,
                    cursor.1,
                    state.edit.line_width(cursor.1).unwrap_or_default()
                ),
            );

            Ok(())
        }
    }

    impl MDEditState {
        pub fn parse_markdown(&mut self) {
            let styles = collect_ast(&self.edit);
            self.edit.clear_styles();
            for (r, s) in styles {
                self.edit.add_style(r, s);
            }
        }

        pub fn load(&mut self) -> Result<(), Error> {
            let Some(path) = &self.path else {
                return Err(anyhow!("No file."));
            };

            let t = fs::read_to_string(path)?;
            self.edit.set_value(t.as_str());

            Ok(())
        }

        pub fn save(&mut self) -> Result<(), Error> {
            let Some(path) = &self.path else {
                return Err(anyhow!("No file."));
            };

            let mut f = BufWriter::new(File::create(path)?);
            let mut buf = Vec::new();
            for line in self.edit.lines_at(0) {
                buf.clear();
                buf.extend(line.bytes().filter(|v| *v != b'\n' && *v != b'\r'));
                buf.extend_from_slice(self.edit.newline().as_bytes());
                f.write_all(&buf)?;
            }
            Ok(())
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
            flow_ok!(match event {
                ct_event!(key press CONTROL-'s') => {
                    self.save()?;
                    Control::Repaint
                }
                ct_event!(key press CONTROL-'c') => {
                    use cli_clipboard;
                    if let Some(v) = self.edit.selected_value() {
                        let r = cli_clipboard::set_contents(v.to_string());
                        debug!("{:?}", r);
                    }
                    Control::Repaint
                }
                ct_event!(key press CONTROL-'x') => {
                    use cli_clipboard;
                    if let Some(v) = self.edit.selected_value() {
                        _ = cli_clipboard::set_contents(v.to_string());
                    }
                    self.edit.delete_range(self.edit.selection());
                    Control::Repaint
                }
                ct_event!(key press CONTROL-'v') => {
                    // todo: might do the insert two times depending on the terminal.
                    use cli_clipboard;
                    if let Ok(v) = cli_clipboard::get_contents() {
                        self.edit.insert_str(&v);
                    }
                    Control::Repaint
                }
                _ => Control::Continue,
            });

            flow_ok!(match self.edit.handle(event, Regular) {
                TextOutcome::TextChanged => {
                    // restart timer
                    self.parse_timer = Some(ctx.replace_timer(
                        self.parse_timer,
                        TimerDef::new().next(Instant::now() + Duration::from_millis(500)),
                    ));
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
                MDAction::MenuOpen => ctx.g.file_dlg.engage(
                    |w| {
                        w.open_dialog(".")?;
                        Ok(Control::Repaint)
                    },
                    |p| Ok(Control::Action(MDAction::Open(p))),
                )?,
                MDAction::MenuSave => ctx.g.file_dlg.engage(
                    |w| {
                        w.save_dialog(".", "")?;
                        Ok(Control::Repaint)
                    },
                    |p| Ok(Control::Action(MDAction::SaveAs(p))),
                )?,

                MDAction::Open(p) => {
                    self.path = Some(p.clone());
                    self.load()?;
                    self.parse_timer = Some(ctx.add_timer(
                        TimerDef::new().next(Instant::now() + Duration::from_millis(100)),
                    ));
                    ctx.focus.as_ref().expect("focus").focus(&self.edit);
                    Control::Repaint
                }

                MDAction::Save() => {
                    self.save()?;
                    Control::Repaint
                }

                MDAction::SaveAs(p) => {
                    self.path = Some(p.clone());
                    self.save()?;
                    Control::Repaint
                }

                _ => Control::Continue,
            });
            Ok(Control::Continue)
        }
    }
}

enum MDStyle {
    Heading = 0,
    BlockQuote,
    CodeBlock,
    FootnoteDefinition,
    FootnoteReference,
    Item,
    Emphasis,
    Strong,
    Strikethrough,
    Link,
    Image,
    MetadataBlock,
    CodeInline,
    MathInline,
    MathDisplay,
    Rule,
    TaskListMarker,
}

fn collect_ast(state: &TextAreaState) -> Vec<(TextRange, usize)> {
    let mut styles = Vec::new();

    let txt = state.value();

    let range = |r: Range<usize>| {
        TextRange::new(
            state.byte_pos(r.start).expect("pos"),
            state.byte_pos(r.end).expect("pos"),
        )
    };

    let p = Parser::new_ext(
        txt.as_str(),
        Options::ENABLE_MATH
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_TABLES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_SMART_PUNCTUATION
            | Options::ENABLE_FOOTNOTES
            | Options::ENABLE_GFM,
    )
    .into_offset_iter();

    for (e, r) in p {
        match e {
            Event::Start(Tag::Heading { .. }) => {
                styles.push((range(r), MDStyle::Heading as usize));
            }
            Event::Start(Tag::BlockQuote(v)) => {
                styles.push((range(r), MDStyle::BlockQuote as usize));
            }
            Event::Start(Tag::CodeBlock(v)) => {
                styles.push((range(r), MDStyle::CodeBlock as usize));
            }
            Event::Start(Tag::FootnoteDefinition(v)) => {
                styles.push((range(r), MDStyle::FootnoteDefinition as usize));
            }
            Event::Start(Tag::Item) => {
                // only color the marker
                let r = if let Some(s) = state.text_slice(range(r.clone())) {
                    let mut n = 0;
                    for c in s.bytes() {
                        if c == b' ' {
                            break;
                        }
                        n += 1;
                    }
                    range(r.start..r.start + n)
                } else {
                    range(r)
                };
                styles.push((r, MDStyle::Item as usize));
            }
            Event::Start(Tag::Emphasis) => {
                styles.push((range(r), MDStyle::Emphasis as usize));
            }
            Event::Start(Tag::Strong) => {
                styles.push((range(r), MDStyle::Strong as usize));
            }
            Event::Start(Tag::Strikethrough) => {
                styles.push((range(r), MDStyle::Strikethrough as usize));
            }
            Event::Start(Tag::Link { .. }) => {
                styles.push((range(r), MDStyle::Link as usize));
            }
            Event::Start(Tag::Image { .. }) => {
                styles.push((range(r), MDStyle::Image as usize));
            }
            Event::Start(Tag::MetadataBlock { .. }) => {
                styles.push((range(r), MDStyle::MetadataBlock as usize));
            }

            Event::Code(v) => {
                styles.push((range(r), MDStyle::CodeInline as usize));
            }
            Event::InlineMath(v) => {
                styles.push((range(r), MDStyle::MathInline as usize));
            }
            Event::DisplayMath(v) => {
                styles.push((range(r), MDStyle::MathDisplay as usize));
            }
            Event::FootnoteReference(v) => {
                styles.push((range(r), MDStyle::FootnoteReference as usize));
            }
            Event::Rule => {
                styles.push((range(r), MDStyle::Rule as usize));
            }
            Event::TaskListMarker(v) => {
                styles.push((range(r), MDStyle::TaskListMarker as usize));
            }

            _ => {}
        }
    }

    styles
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
