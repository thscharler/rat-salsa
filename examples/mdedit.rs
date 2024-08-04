#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unreachable_pub)]

use crate::facilities::{Facility, MDFileDialog, MDFileDialogState};
use crate::mdedit::{MDEdit, MDEditState};
use anyhow::Error;
#[allow(unused_imports)]
use log::debug;
use pulldown_cmark::{Event, Options, Parser, Tag};
use rat_salsa::timer::TimeOut;
use rat_salsa::{run_tui, AppState, AppWidget, Control, RenderContext, RunConfig};
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
use ratatui::prelude::{Line, Modifier, Stylize};
use ratatui::style::Style;
use ratatui::widgets::{Block, BorderType, Padding, StatefulWidget};
use std::fs;
use std::ops::Range;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

type AppContext<'a> = rat_salsa::AppContext<'a, GlobalState, MDAction, Error>;

fn main() -> Result<(), Error> {
    setup_logging()?;

    let config = MDConfig {
        show_ctrl: false,
        new_line: if cfg!(windows) {
            "\r\n".to_string()
        } else {
            "\n".to_string()
        },
    };
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

    // pub mdfocus: Option<MDFocus>,
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

#[derive(Debug)]
pub struct MDConfig {
    pub show_ctrl: bool,
    pub new_line: String,
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
        // s.editor.edit.focus.set_name("edit");
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
                        Control::Changed
                    }
                }
                FileOutcome::Cancel => {
                    Control::Changed
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
            2 => dark_themes()
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
        ctx: &mut RenderContext<'_, GlobalState>,
    ) -> Result<(), Error> {
        let t0 = SystemTime::now();

        let r = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);
        let s = Layout::horizontal([Constraint::Percentage(61), Constraint::Percentage(39)])
            .split(r[1]);

        MDEdit.render(r[0], buf, &mut state.editor, ctx)?;

        let menu_struct = Menu {
            show_ctrl: ctx.g.cfg.show_ctrl,
            use_crlf: ctx.g.cfg.new_line == "\r\n",
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
            let l_msg = layout_middle(
                r[0],
                Constraint::Percentage(19),
                Constraint::Percentage(19),
                Constraint::Percentage(19),
                Constraint::Percentage(19),
            );
            let err = MsgDialog::new()
                .block(
                    Block::bordered()
                        .style(ctx.g.theme.dialog_style())
                        .border_type(BorderType::Rounded)
                        .title_style(Style::new().fg(ctx.g.scheme().red[0]))
                        .padding(Padding::new(1, 1, 1, 1)),
                )
                .styles(ctx.g.theme.msg_dialog_style());
            err.render(l_msg, buf, &mut ctx.g.error_dlg);
        }

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
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

impl AppState<GlobalState, MDAction, Error> for MDAppState {
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
            ct_event!(resized) => Control::Changed,
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
                MenuOutcome::MenuActivated(0, 0) => Control::Message(MDAction::MenuOpen),
                MenuOutcome::MenuActivated(0, 1) => Control::Message(MDAction::MenuSave),
                MenuOutcome::MenuActivated(1, 0) => {
                    ctx.g.cfg.show_ctrl = !ctx.g.cfg.show_ctrl;
                    Control::Changed
                }
                MenuOutcome::MenuActivated(1, 1) => {
                    if ctx.g.cfg.new_line == "\r\n" {
                        ctx.g.cfg.new_line = "\n".into();
                    } else {
                        ctx.g.cfg.new_line = "\r\n".into();
                    }
                    Control::Changed
                }
                MenuOutcome::MenuSelected(2, n) => {
                    ctx.g.theme = dark_themes()[n].clone();
                    Control::Changed
                }
                r => r.into(),
            }
        );

        or_else!(r, self.editor.crossterm(event, ctx)?);

        or_else!(
            r,
            match self.menu.handle(event, Regular) {
                MenuOutcome::Activated(3) => Control::Quit,
                r => r.into(),
            }
        );

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        ctx.g.status.status(3, format!("H {:.3?}", el).to_string());

        Ok(r)
    }

    fn message(
        &mut self,
        event: &mut MDAction,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<MDAction>, Error> {
        let t0 = SystemTime::now();

        let mut r;
        r = match event {
            MDAction::Message(s) => {
                ctx.g.status.status(0, &*s);
                Control::Changed
            }
            _ => Control::Continue,
        };

        ctx.focus = Some(self.focus());

        // TODO: actions
        or_else!(r, self.editor.message(event, ctx)?);

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        ctx.g.status.status(3, format!("A {:.3?}", el).to_string());

        Ok(r)
    }

    fn error(&self, event: Error, ctx: &mut AppContext<'_>) -> Result<Control<MDAction>, Error> {
        ctx.g.error_dlg.title("Error occured");
        ctx.g.error_dlg.append(format!("{:?}", &*event).as_str());
        Ok(Control::Changed)
    }
}

pub mod mdsingle {
    use crate::{collect_ast, text_style, AppContext, GlobalState, MDAction};
    use anyhow::{anyhow, Error};
    use crossterm::event::Event;
    use log::debug;
    use rat_salsa::timer::{TimeOut, TimerDef, TimerHandle};
    use rat_salsa::{AppState, AppWidget, Control, RenderContext};
    use rat_widget::event::{ct_event, flow_ok, HandleEvent, Outcome, Regular, TextOutcome};
    use rat_widget::focus::{FocusFlag, HasFocusFlag};
    use rat_widget::scrolled::Scroll;
    use rat_widget::text::undo::UndoVec;
    use rat_widget::textarea::{TextArea, TextAreaState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::prelude::StatefulWidget;
    use std::fs;
    use std::fs::File;
    use std::io::{BufWriter, Write};
    use std::path::{Path, PathBuf};
    use std::time::{Duration, Instant};

    #[derive(Debug, Default, Clone)]
    pub struct MDSingle;

    #[derive(Debug)]
    pub struct MDSingleState {
        pub path: Option<PathBuf>,
        pub changed: bool,
        pub edit: TextAreaState,
        pub parse_timer: Option<TimerHandle>,
    }

    impl Default for MDSingleState {
        fn default() -> Self {
            Self {
                path: None,
                changed: false,
                edit: TextAreaState::default(),
                parse_timer: None,
            }
        }
    }

    impl Clone for MDSingleState {
        fn clone(&self) -> Self {
            Self {
                path: self.path.clone(),
                changed: self.changed,
                edit: self.edit.clone(),
                parse_timer: None,
            }
        }
    }

    impl AppWidget<GlobalState, MDAction, Error> for MDSingle {
        type State = MDSingleState;

        fn render(
            &self,
            area: Rect,
            buf: &mut Buffer,
            state: &mut Self::State,
            ctx: &mut RenderContext<'_, GlobalState>,
        ) -> Result<(), Error> {
            TextArea::new()
                .styles(ctx.g.theme.textarea_style())
                .vscroll(
                    Scroll::new()
                        .split_mark_offset(1)
                        .scroll_by(1)
                        .styles(ctx.g.theme.scroll_style()),
                )
                .show_ctrl(ctx.g.cfg.show_ctrl)
                .text_style(text_style(ctx))
                .render(area, buf, &mut state.edit);
            ctx.set_screen_cursor(state.edit.screen_cursor());

            if state.is_focused() {
                let cursor = state.edit.cursor();

                let sel = state.edit.selection();
                let sel_len = if sel.start.y == sel.end.y {
                    sel.end.x.saturating_sub(sel.start.x)
                } else {
                    sel.end.y.saturating_sub(sel.start.y) + 1
                };

                ctx.g
                    .status
                    .status(1, format!("{}:{}|{}", cursor.x, cursor.y, sel_len));
            }

            Ok(())
        }
    }

    impl HasFocusFlag for MDSingleState {
        fn focus(&self) -> &FocusFlag {
            self.edit.focus()
        }

        fn area(&self) -> Rect {
            self.edit.area()
        }

        fn navigable(&self) -> bool {
            self.edit.navigable()
        }

        fn primary_keys(&self) -> bool {
            self.edit.primary_keys()
        }
    }

    impl MDSingleState {
        pub fn parse_markdown(&mut self) {
            let styles = collect_ast(&self.edit);
            self.edit.set_styles(styles);
        }

        pub fn open(&mut self, path: &Path, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            self.path = Some(path.into());
            let t = fs::read_to_string(path)?;
            self.edit.set_value(t.as_str());
            self.parse_timer = Some(
                ctx.add_timer(TimerDef::new().next(Instant::now() + Duration::from_millis(0))),
            );
            Ok(())
        }

        pub fn save_as(&mut self, path: &Path) -> Result<(), Error> {
            self.path = Some(path.into());
            self.save()
        }

        pub fn save(&mut self) -> Result<(), Error> {
            if self.changed {
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

                self.changed = false;
            }
            Ok(())
        }
    }

    impl AppState<GlobalState, MDAction, Error> for MDSingleState {
        fn timer(
            &mut self,
            event: &TimeOut,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<MDAction>, Error> {
            debug!(
                "test_parse_timer {:?} <> {:?} => {}",
                self.parse_timer,
                event.handle,
                self.parse_timer == Some(event.handle)
            );
            if self.parse_timer == Some(event.handle) {
                self.parse_markdown();
                return Ok(Control::Changed);
            }
            Ok(Control::Continue)
        }

        fn crossterm(
            &mut self,
            event: &Event,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<MDAction>, Error> {
            flow_ok!(match event {
                ct_event!(key press CONTROL-'c') => {
                    use cli_clipboard;
                    if let Some(v) = self.edit.selected_value() {
                        let r = cli_clipboard::set_contents(v.to_string());
                    }
                    Outcome::Changed
                }
                ct_event!(key press CONTROL-'x') => {
                    use cli_clipboard;
                    if let Some(v) = self.edit.selected_value() {
                        _ = cli_clipboard::set_contents(v.to_string());
                    }
                    self.edit.delete_range(self.edit.selection());
                    Outcome::Changed
                }
                ct_event!(key press CONTROL-'v') => {
                    // todo: might do the insert two times depending on the terminal.
                    use cli_clipboard;
                    if let Ok(v) = cli_clipboard::get_contents() {
                        self.edit.insert_str(&v);
                    }
                    Outcome::Changed
                }
                _ => Outcome::Continue,
            });

            flow_ok!(match self.edit.handle(event, Regular) {
                TextOutcome::TextChanged => {
                    // restart timer
                    debug!("old timer {:?}", self.parse_timer);
                    self.parse_timer = Some(ctx.replace_timer(
                        self.parse_timer,
                        TimerDef::new().next(Instant::now() + Duration::from_millis(200)),
                    ));
                    debug!("timer {:?}", self.parse_timer);
                    Control::Changed
                }
                r => r.into(),
            });

            Ok(Control::Continue)
        }
    }
}

pub mod mdtabbed {
    use crate::mdsingle::{MDSingle, MDSingleState};
    use crate::{AppContext, GlobalState, MDAction};
    use anyhow::Error;
    use crossterm::event::Event;
    use log::debug;
    use rat_salsa::event::flow_ok;
    use rat_salsa::timer::TimeOut;
    use rat_salsa::{AppState, AppWidget, Control, RenderContext};
    use rat_widget::event::{HandleEvent, Regular};
    use rat_widget::focus::HasFocusFlag;
    use rat_widget::tabbed::attached::AttachedTabs;
    use rat_widget::tabbed::{Tabbed, TabbedState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::prelude::{Line, StatefulWidget};
    use std::borrow::Cow;

    #[derive(Debug, Default)]
    pub struct MDTabbed;

    #[derive(Debug, Default)]
    pub struct MDTabbedState {
        pub tabbed: TabbedState,
        pub tabs: Vec<MDSingleState>,
    }

    impl AppWidget<GlobalState, MDAction, Error> for MDTabbed {
        type State = MDTabbedState;

        fn render(
            &self,
            area: Rect,
            buf: &mut Buffer,
            state: &mut Self::State,
            ctx: &mut RenderContext<'_, GlobalState>,
        ) -> Result<(), Error> {
            Tabbed::new()
                .tab_type(AttachedTabs::new())
                .styles(ctx.g.theme.tabbed_style())
                .as_if_focused(state.is_focused())
                .tabs(state.tabs.iter().map(|v| {
                    let title = if let Some(path) = v.path.as_ref() {
                        path.file_name().unwrap_or_default().to_string_lossy()
                    } else {
                        Cow::Borrowed("**New**")
                    };
                    Line::from(title.to_string())
                }))
                .render(area, buf, &mut state.tabbed);

            if let Some(tab) = state.tabbed.selected() {
                MDSingle.render(state.tabbed.widget_area, buf, &mut state.tabs[tab], ctx)?;
            }

            Ok(())
        }
    }

    impl AppState<GlobalState, MDAction, Error> for MDTabbedState {
        fn timer(
            &mut self,
            event: &TimeOut,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, MDAction, Error>,
        ) -> Result<Control<MDAction>, Error> {
            for v in &mut self.tabs {
                debug!("timer tabs");
                flow_ok!(v.timer(event, ctx)?);
            }
            debug!("no timer tabs");
            Ok(Control::Continue)
        }

        fn crossterm(
            &mut self,
            event: &Event,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<MDAction>, Error> {
            flow_ok!(self.tabbed.handle(event, Regular));
            if let Some(selected) = self.tabbed.selected() {
                flow_ok!(self.tabs[selected].crossterm(event, ctx)?);
            }
            Ok(Control::Continue)
        }
    }

    impl MDTabbedState {
        pub fn gained_focus(&self) -> bool {
            if let Some(selected) = self.tabbed.selected() {
                self.tabs[selected].gained_focus()
            } else {
                false
            }
        }

        pub fn is_focused(&self) -> bool {
            if let Some(selected) = self.tabbed.selected() {
                self.tabs[selected].is_focused()
            } else {
                false
            }
        }
    }
}

pub mod mdsplit {
    use crate::mdtabbed::{MDTabbed, MDTabbedState};
    use crate::{AppContext, GlobalState, MDAction};
    use anyhow::Error;
    use crossterm::event::Event;
    use log::debug;
    use rat_salsa::timer::TimeOut;
    use rat_salsa::{AppState, AppWidget, Control, RenderContext};
    use rat_widget::event::{flow_ok, HandleEvent, Regular};
    use rat_widget::focus::{Focus, FocusFlag, HasFocus, HasFocusFlag};
    use rat_widget::splitter::{Split, SplitState, SplitType};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Rect};
    use ratatui::prelude::StatefulWidget;

    #[derive(Debug, Default)]
    pub struct MDSplit;

    #[derive(Debug, Default)]
    pub struct MDSplitState {
        pub focus: FocusFlag,
        pub splitter: SplitState,
        pub selected_split: Option<usize>,
        pub split: Vec<MDTabbedState>,
    }

    impl AppWidget<GlobalState, MDAction, Error> for MDSplit {
        type State = MDSplitState;

        fn render(
            &self,
            area: Rect,
            buf: &mut Buffer,
            state: &mut Self::State,
            ctx: &mut RenderContext<'_, GlobalState>,
        ) -> Result<(), Error> {
            let (s0, s1) = Split::new()
                .constraints(vec![Constraint::Fill(1); state.split.len()])
                .mark_offset(1)
                .split_type(SplitType::FullPlain)
                .styles(ctx.g.theme.split_style())
                .direction(Direction::Horizontal)
                .into_widgets();

            s0.render(area, buf, &mut state.splitter);

            for (i, edit_area) in state.splitter.widget_areas.iter().enumerate() {
                MDTabbed.render(*edit_area, buf, &mut state.split[i], ctx)?;
            }
            s1.render(area, buf, &mut state.splitter);

            Ok(())
        }
    }

    impl HasFocus for MDSplitState {
        fn focus(&self) -> Focus {
            let mut f = Focus::new_grp(&self.focus, &[]);
            for v in self.split.iter() {
                if let Some(tab) = v.tabbed.selected() {
                    f.add(&v.tabs[tab]);
                }
            }
            f
        }
    }

    impl MDSplitState {}

    impl AppState<GlobalState, MDAction, Error> for MDSplitState {
        fn timer(
            &mut self,
            event: &TimeOut,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<MDAction>, Error> {
            for v in &mut self.split {
                debug!("timer split");
                flow_ok!(v.timer(event, ctx)?);
            }
            debug!("no timer split");
            Ok(Control::Continue)
        }

        fn crossterm(
            &mut self,
            event: &Event,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<MDAction>, Error> {
            for (idx, v) in self.split.iter().enumerate() {
                if v.gained_focus() {
                    self.selected_split = Some(idx);
                }
            }

            flow_ok!(self.splitter.handle(event, Regular));
            for w in &mut self.split {
                flow_ok!(w.crossterm(event, ctx)?);
            }

            Ok(Control::Continue)
        }
    }
}

pub mod mdedit {
    use crate::facilities::Facility;
    use crate::mdsingle::MDSingleState;
    use crate::mdsplit::{MDSplit, MDSplitState};
    use crate::mdtabbed::MDTabbedState;
    use crate::{AppContext, GlobalState, MDAction, RenderContext};
    use anyhow::Error;
    use crossterm::event::Event;
    #[allow(unused_imports)]
    use log::debug;
    use rat_salsa::event::{ct_event, flow_ok};
    use rat_salsa::timer::TimeOut;
    use rat_salsa::{AppState, AppWidget, Control};
    use rat_widget::event::ConsumedEvent;
    use rat_widget::focus::{Focus, HasFocus, HasFocusFlag};
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::StatefulWidget;
    use std::path::Path;

    #[derive(Debug, Default)]
    pub struct MDEdit;

    #[derive(Debug, Default)]
    pub struct MDEditState {
        pub edit: MDSplitState,
    }

    impl AppWidget<GlobalState, MDAction, Error> for MDEdit {
        type State = MDEditState;

        fn render(
            &self,
            area: Rect,
            buf: &mut Buffer,
            state: &mut Self::State,
            ctx: &mut RenderContext<'_, GlobalState>,
        ) -> Result<(), Error> {
            MDSplit.render(area, buf, &mut state.edit, ctx)?;
            Ok(())
        }
    }

    impl HasFocus for MDEditState {
        fn focus(&self) -> Focus {
            self.edit.focus()
        }
    }

    impl AppState<GlobalState, MDAction, Error> for MDEditState {
        fn timer(
            &mut self,
            event: &TimeOut,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<MDAction>, Error> {
            flow_ok!(self.edit.timer(event, ctx)?);
            Ok(Control::Changed)
        }

        fn crossterm(
            &mut self,
            event: &Event,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<MDAction>, Error> {
            flow_ok!(match event {
                ct_event!(key press CONTROL-'s') => {
                    self.save()?;
                    Control::Changed
                }
                ct_event!(key press CONTROL-'h') => {
                    self.split(ctx)?;
                    Control::Changed
                }
                _ => Control::Continue,
            });

            // todo: ctrl+w for window nav
            // todo: ctrl+t for tab nav

            let r = self.edit.crossterm(event, ctx)?;

            // synchronize instances
            if r.is_consumed() {
                let (replay_path, replay) = if let Some(sel_edit) = self.selected_mut() {
                    (sel_edit.path.clone(), sel_edit.edit.recent_replay())
                } else {
                    (None, Vec::default())
                };
                if !replay.is_empty() {
                    for v in &mut self.edit.split {
                        for t in &mut v.tabs {
                            if t.path == replay_path && !t.is_focused() {
                                t.edit.replay(&replay);
                            }
                        }
                    }
                }
            }

            Ok(r)
        }

        fn message(
            &mut self,
            event: &mut MDAction,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, MDAction, Error>,
        ) -> Result<Control<MDAction>, Error> {
            flow_ok!(match event {
                MDAction::MenuOpen => ctx.g.file_dlg.engage(
                    |w| {
                        w.open_dialog(".")?;
                        Ok(Control::Changed)
                    },
                    |p| Ok(Control::Message(MDAction::Open(p))),
                )?,
                MDAction::MenuSave => ctx.g.file_dlg.engage(
                    |w| {
                        w.save_dialog(".", "")?;
                        Ok(Control::Changed)
                    },
                    |p| Ok(Control::Message(MDAction::SaveAs(p))),
                )?,

                MDAction::Open(p) => {
                    self.open(p, ctx)?;
                    Control::Changed
                }
                MDAction::Save() => {
                    self.save()?;
                    Control::Changed
                }
                MDAction::SaveAs(p) => {
                    self.save_as(p)?;
                    Control::Changed
                }
                _ => Control::Continue,
            });
            Ok(Control::Continue)
        }
    }

    impl MDEditState {
        pub fn selected(&self) -> Option<&MDSingleState> {
            for v in &self.edit.split {
                if let Some(tab) = v.tabbed.selected() {
                    if v.tabs[tab].is_focused() {
                        return Some(&v.tabs[tab]);
                    }
                }
            }
            None
        }

        pub fn selected_mut(&mut self) -> Option<&mut MDSingleState> {
            for v in &mut self.edit.split {
                if let Some(tab) = v.tabbed.selected() {
                    if v.tabs[tab].is_focused() {
                        return Some(&mut v.tabs[tab]);
                    }
                }
            }
            None
        }

        pub fn open(&mut self, path: &Path, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            let mut new = MDSingleState::default();
            let new_focus = new.focus().clone();

            new.open(path, ctx)?;

            if self.edit.selected_split.is_none() {
                self.edit.split.push(MDTabbedState::default());
                self.edit.selected_split = Some(0);
            }
            if let Some(selected_split) = self.edit.selected_split {
                let tab = &mut self.edit.split[selected_split];
                tab.tabs.push(new);
                tab.tabbed.set_selected(tab.tabs.len() - 1);
            }

            if let Some(focus) = &mut ctx.focus {
                // just add at end, the exact order is not important here.
                focus.add_flag(&new_focus, Rect::default());
                focus.focus_flag(&new_focus);
            }

            Ok(())
        }

        pub fn split(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            let Some(sel_single) = self.selected_mut() else {
                return Ok(());
            };

            sel_single
                .edit
                .undo_buffer_mut()
                .expect("undo")
                .set_replay(true);

            let mut new = sel_single.clone();
            let new_focus = new.focus().clone();

            if let Some(selected_split) = self.edit.selected_split {
                self.edit
                    .split
                    .insert(selected_split + 1, MDTabbedState::default());
                self.edit.selected_split = Some(selected_split + 1);
            } else {
                self.edit.split.push(MDTabbedState::default());
                self.edit.selected_split = Some(0);
            }

            if let Some(selected_split) = self.edit.selected_split {
                let tab = &mut self.edit.split[selected_split];
                tab.tabs.push(new);
            }

            if let Some(focus) = &mut ctx.focus {
                // just add at end, the exact order is not important here.
                focus.add_flag(&new_focus, Rect::default());
                focus.focus_flag(&new_focus);
            }

            Ok(())
        }

        pub fn save(&mut self) -> Result<(), Error> {
            Ok(())
        }

        pub fn save_as(&mut self, path: &Path) -> Result<(), Error> {
            Ok(())
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

fn text_style(ctx: &mut RenderContext<'_, GlobalState>) -> [Style; 17] {
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

fn setup_logging() -> Result<(), Error> {
    _ = fs::remove_file("log.log");
    fern::Dispatch::new()
        .format(|out, message, _record| out.finish(format_args!("{}", message)))
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("log.log")?)
        .apply()?;
    Ok(())
}
