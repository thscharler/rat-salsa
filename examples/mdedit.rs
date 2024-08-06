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
use rat_widget::event::{ct_event, or_else, ConsumedEvent, Dialog, HandleEvent, Popup, Regular};
use rat_widget::focus::{Focus, HasFocus, HasFocusFlag};
use rat_widget::layout::layout_middle;
use rat_widget::menubar::{MenuBarState, MenuStructure, Menubar, StaticMenu};
use rat_widget::menuline::MenuOutcome;
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use rat_widget::popup_menu::{MenuItem, Placement, Separator};
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

#[derive(Debug, PartialEq, Eq)]
pub enum MDAction {
    Message(String),
    MenuNew,
    MenuOpen,
    MenuSave,
    MenuSaveAs,
    New(PathBuf),
    Open(PathBuf),
    Show(String, Vec<(TextRange, usize)>),
    SaveAs(PathBuf),
    Save,
    CloseSelected,
    Close(usize, usize),
    Select(usize, usize),
    Split,
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
        ("_File", &["_New", "_Open", "_Save", "Save _as"]), //
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

    fn submenu(&'a self, n: usize) -> Vec<MenuItem<'a>> {
        match n {
            1 => {
                vec![
                    if self.show_ctrl {
                        MenuItem::Item("\u{2611} Control chars".into())
                    } else {
                        MenuItem::Item("\u{2610} Control chars".into())
                    },
                    if self.use_crlf {
                        MenuItem::Item("\u{2611} Use CR+LF".into())
                    } else {
                        MenuItem::Item("\u{2610} Use CR+LF".into())
                    },
                    MenuItem::Sep(Separator::Dotted),
                    MenuItem::Item2("Split view".into(), Some('s')),
                ]
            }
            2 => dark_themes()
                .iter()
                .map(|v| MenuItem::Item(v.name().to_string().into()))
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
        ctx.g.status.status(2, format!("R {:.0?}", el).to_string());

        let status = StatusLine::new()
            .layout([
                Constraint::Fill(1),
                Constraint::Length(14),
                Constraint::Length(7),
                Constraint::Length(7),
                Constraint::Length(7),
            ])
            .styles(vec![
                ctx.g.theme.status_style(),
                ctx.g.theme.deepblue(3),
                ctx.g.theme.deepblue(2),
                ctx.g.theme.deepblue(1),
                ctx.g.theme.deepblue(0),
            ]);
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

        if r == Control::Changed {
            let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
            ctx.g.status.status(4, format!("T {:.0?}", el).to_string());
        }

        Ok(r)
    }

    fn crossterm(
        &mut self,
        event: &crossterm::event::Event,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<MDAction>, Error> {
        let t0 = SystemTime::now();

        ctx.focus = Some(self.focus());

        let mut r;
        r = match &event {
            ct_event!(resized) => Control::Changed,
            ct_event!(key press CONTROL-'q') => Control::Quit,
            ct_event!(keycode press Esc) => {
                if !self.menu.is_focused() {
                    ctx.focus.as_ref().expect("focus").focus(&self.menu);
                    Control::Changed
                } else {
                    Control::Continue
                }
            }
            _ => Control::Continue,
        };

        or_else!(r, ctx.g.error_dlg.handle(event, Dialog).into());
        or_else!(r, ctx.g.file_dlg.handle(event)?);

        // focus
        if !r.is_consumed() {
            let f = ctx.focus.as_mut().expect("focus").handle(event, Regular);
            // extended focus handling
            if !f.is_consumed() {}
            ctx.queue(f);
        }

        or_else!(
            r,
            match self.menu.handle(event, Popup) {
                MenuOutcome::MenuActivated(0, 0) => Control::Message(MDAction::MenuNew),
                MenuOutcome::MenuActivated(0, 1) => Control::Message(MDAction::MenuOpen),
                MenuOutcome::MenuActivated(0, 2) => Control::Message(MDAction::MenuSave),
                MenuOutcome::MenuActivated(0, 3) => Control::Message(MDAction::MenuSaveAs),
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
                MenuOutcome::MenuActivated(1, 2) => {
                    Control::Message(MDAction::Split)
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

        if r == Control::Changed {
            let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
            ctx.g.status.status(3, format!("H {:.0?}", el).to_string());
        }

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

        if r == Control::Changed {
            let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
            ctx.g.status.status(4, format!("A {:.0?}", el).to_string());
        }

        Ok(r)
    }

    fn error(&self, event: Error, ctx: &mut AppContext<'_>) -> Result<Control<MDAction>, Error> {
        ctx.g.error_dlg.title("Error occured");
        ctx.g.error_dlg.append(format!("{:?}", &*event).as_str());
        Ok(Control::Changed)
    }
}

pub mod mdfile {
    use crate::{collect_ast, text_style, AppContext, GlobalState, MDAction};
    use anyhow::Error;
    use crossterm::event::Event;
    use rat_salsa::timer::{TimeOut, TimerDef, TimerHandle};
    use rat_salsa::{AppState, AppWidget, Control, RenderContext};
    use rat_widget::event::{ct_event, flow_ok, HandleEvent, Outcome, Regular, TextOutcome};
    use rat_widget::focus::{FocusFlag, HasFocusFlag};
    use rat_widget::scrolled::Scroll;
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
    pub struct MDFile;

    #[derive(Debug)]
    pub struct MDFileState {
        pub path: PathBuf,
        pub changed: bool,
        pub edit: TextAreaState,
        pub parse_timer: Option<TimerHandle>,
    }

    impl Clone for MDFileState {
        fn clone(&self) -> Self {
            Self {
                path: self.path.clone(),
                changed: self.changed,
                edit: self.edit.clone(),
                parse_timer: None,
            }
        }
    }

    impl AppWidget<GlobalState, MDAction, Error> for MDFile {
        type State = MDFileState;

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

    impl HasFocusFlag for MDFileState {
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

    impl MDFileState {
        pub fn new(path: &Path) -> Self {
            Self {
                path: path.into(),
                changed: false,
                edit: Default::default(),
                parse_timer: None,
            }
        }

        pub fn parse_markdown(&mut self) {
            let styles = collect_ast(&self.edit);
            self.edit.set_styles(styles);
        }

        pub fn open(&mut self, path: &Path, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            self.path = path.into();
            let t = fs::read_to_string(path)?;
            self.edit.set_value(t.as_str());
            self.parse_timer = Some(
                ctx.add_timer(TimerDef::new().next(Instant::now() + Duration::from_millis(0))),
            );
            Ok(())
        }

        pub fn save_as(&mut self, path: &Path) -> Result<(), Error> {
            self.path = path.into();
            self.save()
        }

        pub fn save(&mut self) -> Result<(), Error> {
            if self.changed {
                let mut f = BufWriter::new(File::create(&self.path)?);
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

    impl AppState<GlobalState, MDAction, Error> for MDFileState {
        fn timer(
            &mut self,
            event: &TimeOut,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<MDAction>, Error> {
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
                    self.changed = true;
                    self.edit.delete_range(self.edit.selection());
                    Outcome::Changed
                }
                ct_event!(key press CONTROL-'v') => {
                    // todo: might do the insert two times depending on the terminal.
                    use cli_clipboard;
                    if let Ok(v) = cli_clipboard::get_contents() {
                        self.changed = true;
                        self.edit.insert_str(&v);
                    }
                    Outcome::Changed
                }
                _ => Outcome::Continue,
            });

            flow_ok!(match self.edit.handle(event, Regular) {
                TextOutcome::TextChanged => {
                    self.changed = true;
                    // restart timer
                    self.parse_timer = Some(ctx.replace_timer(
                        self.parse_timer,
                        TimerDef::new().next(Instant::now() + Duration::from_millis(200)),
                    ));
                    Control::Changed
                }
                r => r.into(),
            });

            Ok(Control::Continue)
        }
    }
}

pub mod mdstructure {
    use crate::mdfile::{MDFile, MDFileState};
    use crate::{AppContext, GlobalState, MDAction};
    use anyhow::Error;
    use crossterm::event::Event;
    use rat_salsa::timer::TimeOut;
    use rat_salsa::{AppState, AppWidget, Control, RenderContext};
    use rat_widget::event::{flow_ok, HandleEvent, Regular, TabbedOutcome};
    use rat_widget::focus::{Focus, FocusFlag, HasFocus, HasFocusFlag};
    use rat_widget::splitter::{Split, SplitState, SplitType};
    use rat_widget::tabbed::attached::AttachedTabs;
    use rat_widget::tabbed::{Tabbed, TabbedState};
    use rat_widget::text::undo::UndoEntry;
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Rect};
    use ratatui::prelude::{Line, StatefulWidget};
    use std::path::Path;

    #[derive(Debug, Default)]
    pub struct MDStructure;

    #[derive(Debug, Default)]
    pub struct MDStructureState {
        pub focus: FocusFlag,
        pub splitter: SplitState,
        pub sel_split: Option<usize>,
        pub tabbed: Vec<TabbedState>,
        pub tabs: Vec<Vec<MDFileState>>,
    }

    impl AppWidget<GlobalState, MDAction, Error> for MDStructure {
        type State = MDStructureState;

        fn render(
            &self,
            area: Rect,
            buf: &mut Buffer,
            state: &mut Self::State,
            ctx: &mut RenderContext<'_, GlobalState>,
        ) -> Result<(), Error> {
            let (s0, s1) = Split::new()
                .constraints(vec![Constraint::Fill(1); state.tabbed.len()])
                .mark_offset(1)
                .split_type(SplitType::FullPlain)
                .styles(ctx.g.theme.split_style())
                .direction(Direction::Horizontal)
                .into_widgets();

            s0.render(area, buf, &mut state.splitter);

            for (idx_split, edit_area) in state.splitter.widget_areas.iter().enumerate() {
                Tabbed::new()
                    .tab_type(AttachedTabs::new())
                    .closeable(true)
                    .styles(ctx.g.theme.tabbed_style())
                    .as_if_focused(state.sel_split == Some(idx_split))
                    .tabs(state.tabs[idx_split].iter().map(|v| {
                        let title = v.path.file_name().unwrap_or_default().to_string_lossy();
                        Line::from(title.to_string())
                    }))
                    .render(*edit_area, buf, &mut state.tabbed[idx_split]);

                if let Some(idx_tab) = state.tabbed[idx_split].selected() {
                    MDFile.render(
                        state.tabbed[idx_split].widget_area,
                        buf,
                        &mut state.tabs[idx_split][idx_tab],
                        ctx,
                    )?;
                }
            }

            s1.render(area, buf, &mut state.splitter);

            Ok(())
        }
    }

    impl HasFocus for MDStructureState {
        fn focus(&self) -> Focus {
            let mut f = Focus::new_grp(&self.focus, &[]);
            f.add(&self.splitter);
            for (idx_split, tabbed) in self.tabbed.iter().enumerate() {
                f.add(&self.tabbed[idx_split]);
                if let Some(idx_tab) = tabbed.selected() {
                    f.add(&self.tabs[idx_split][idx_tab]);
                }
            }
            f
        }
    }

    impl MDStructureState {
        pub fn select(&mut self, pos: (usize, usize), ctx: &mut AppContext<'_>) {
            if pos.0 < self.tabs.len() {
                if pos.1 < self.tabs[pos.0].len() {
                    self.sel_split = Some(pos.0);
                    self.tabbed[pos.0].select(pos.1);
                    ctx.focus
                        .as_mut()
                        .expect("focus")
                        .focus(&self.tabs[pos.0][pos.1]);
                }
            }
        }

        pub fn select_next(&mut self, ctx: &mut AppContext<'_>) -> bool {
            if let Some(idx_split) = self.sel_split {
                if idx_split < self.tabs.len() {
                    let new_split = idx_split + 1;
                    let new_tab = self.tabbed[new_split].selected().unwrap_or_default();
                    self.select((new_split, new_tab), ctx);
                    return true;
                }
            }
            false
        }

        pub fn select_prev(&mut self, ctx: &mut AppContext<'_>) -> bool {
            if let Some(idx_split) = self.sel_split {
                if idx_split > 0 {
                    let new_split = idx_split - 1;
                    let new_tab = self.tabbed[new_split].selected().unwrap_or_default();
                    self.select((new_split, new_tab), ctx);
                    return true;
                }
            }
            false
        }

        pub fn close(
            &mut self,
            pos: (usize, usize),
            ctx: &mut AppContext<'_>,
        ) -> Result<(), Error> {
            if pos.0 < self.tabs.len() {
                if pos.1 < self.tabs[pos.0].len() {
                    self.tabs[pos.0][pos.1].save()?;
                    self.tabs[pos.0].remove(pos.1);

                    if self.tabs[pos.0].len() == 0 {
                        self.tabs.remove(pos.0);
                        self.tabbed.remove(pos.0);
                        if let Some(sel_split) = self.sel_split {
                            if sel_split == pos.0 {
                                if sel_split > 0 {
                                    self.sel_split = Some(sel_split - 1);
                                } else if self.tabbed.len() > 0 {
                                    self.sel_split = Some(0);
                                } else {
                                    self.sel_split = None;
                                }
                            }
                        }
                        if self.sel_split == Some(pos.0) {}
                    } else if pos.1 > self.tabs[pos.0].len() {
                        self.tabbed[pos.0].select(self.tabs[pos.0].len() - 1);
                    }
                }
            }
            Ok(())
        }

        pub fn show_edit(&mut self, pos: (usize, usize), new: MDFileState) {
            if pos.0 == self.tabs.len() {
                self.tabs.push(Vec::new());
                self.tabbed.push(TabbedState::default());
            }
            self.tabs[pos.0].insert(pos.1, new);
        }

        pub fn selected_pos(&self) -> Option<(usize, usize)> {
            if let Some(idx_split) = self.sel_split {
                if let Some(idx_tab) = self.tabbed[idx_split].selected() {
                    return Some((idx_split, idx_tab));
                }
            }
            None
        }

        pub fn selected(&self) -> Option<((usize, usize), &MDFileState)> {
            if let Some(idx_split) = self.sel_split {
                if let Some(idx_tab) = self.tabbed[idx_split].selected() {
                    return Some(((idx_split, idx_tab), &self.tabs[idx_split][idx_tab]));
                }
            }
            None
        }

        pub fn selected_mut(&mut self) -> Option<((usize, usize), &mut MDFileState)> {
            if let Some(idx_split) = self.sel_split {
                if let Some(idx_tab) = self.tabbed[idx_split].selected() {
                    return Some(((idx_split, idx_tab), &mut self.tabs[idx_split][idx_tab]));
                }
            }
            None
        }

        pub fn for_path(&self, path: &Path) -> Option<((usize, usize), &MDFileState)> {
            for (idx_split, tabs) in self.tabs.iter().enumerate() {
                for (idx_tab, tab) in tabs.iter().enumerate() {
                    if tab.path == path {
                        return Some(((idx_split, idx_tab), tab));
                    }
                }
            }
            None
        }

        pub fn for_path_mut(&mut self, path: &Path) -> Option<((usize, usize), &mut MDFileState)> {
            for (idx_split, tabs) in self.tabs.iter_mut().enumerate() {
                for (idx_tab, tab) in tabs.iter_mut().enumerate() {
                    if tab.path == path {
                        return Some(((idx_split, idx_tab), tab));
                    }
                }
            }
            None
        }

        pub fn replay(&mut self, id: (usize, usize), path: &Path, replay: &[UndoEntry]) {
            for (idx_split, tabs) in self.tabs.iter_mut().enumerate() {
                for (idx_tab, tab) in tabs.iter_mut().enumerate() {
                    if id != (idx_split, idx_tab) && tab.path == path {
                        tab.edit.replay(replay);
                    }
                }
            }
        }

        pub fn save(&mut self) -> Result<(), Error> {
            for (idx_split, tabs) in self.tabs.iter_mut().enumerate() {
                for (idx_tab, tab) in tabs.iter_mut().enumerate() {
                    return tab.save();
                }
            }
            Ok(())
        }
    }

    impl AppState<GlobalState, MDAction, Error> for MDStructureState {
        fn timer(
            &mut self,
            event: &TimeOut,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<MDAction>, Error> {
            for split in &mut self.tabs {
                for tab in split {
                    flow_ok!(tab.timer(event, ctx)?);
                }
            }
            Ok(Control::Continue)
        }

        fn crossterm(
            &mut self,
            event: &Event,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<MDAction>, Error> {
            for (idx_split, tabbed) in self.tabbed.iter().enumerate() {
                if let Some(idx_tab) = tabbed.selected() {
                    if self.tabs[idx_split][idx_tab].gained_focus() {
                        self.sel_split = Some(idx_split);
                        break;
                    }
                }
            }

            flow_ok!(self.splitter.handle(event, Regular));
            for (idx_split, tabbed) in self.tabbed.iter_mut().enumerate() {
                flow_ok!(match tabbed.handle(event, Regular) {
                    TabbedOutcome::Close(n) => {
                        Control::Message(MDAction::Close(idx_split, n))
                    }
                    TabbedOutcome::Select(n) => {
                        Control::Message(MDAction::Select(idx_split, n))
                    }
                    r => r.into(),
                });

                if let Some(idx_tab) = tabbed.selected() {
                    flow_ok!(self.tabs[idx_split][idx_tab].crossterm(event, ctx)?);
                }
            }

            Ok(Control::Continue)
        }
    }
}

pub mod mdedit {
    use crate::facilities::Facility;
    use crate::mdfile::MDFileState;
    use crate::mdstructure::{MDStructure, MDStructureState};
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
    use std::path::{Path, PathBuf};

    #[derive(Debug, Default)]
    pub struct MDEdit;

    #[derive(Debug, Default)]
    pub struct MDEditState {
        pub window_cmd: bool,
        pub structure: MDStructureState,
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
            MDStructure.render(area, buf, &mut state.structure, ctx)?;

            if state.window_cmd {
                ctx.g.status.status(1, "^W");
            }

            Ok(())
        }
    }

    impl HasFocus for MDEditState {
        fn focus(&self) -> Focus {
            self.structure.focus()
        }
    }

    impl AppState<GlobalState, MDAction, Error> for MDEditState {
        fn timer(
            &mut self,
            event: &TimeOut,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<MDAction>, Error> {
            flow_ok!(self.structure.timer(event, ctx)?);
            Ok(Control::Changed)
        }

        fn crossterm(
            &mut self,
            event: &Event,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<MDAction>, Error> {
            if self.window_cmd {
                self.window_cmd = false;
                flow_ok!(match event {
                    ct_event!(keycode press Left) => {
                        self.structure.select_prev(ctx);
                        Control::Changed
                    }
                    ct_event!(keycode press Right) => {
                        self.structure.select_next(ctx);
                        Control::Changed
                    }
                    ct_event!(key press 'c') => {
                        Control::Message(MDAction::CloseSelected)
                    }
                    ct_event!(key press CONTROL-'t') | ct_event!(key press 't') => {
                        if let Some((pos, sel)) = self.structure.selected() {
                            if sel.is_focused() {
                                ctx.focus
                                    .as_ref()
                                    .expect("focus")
                                    .focus(&self.structure.tabbed[pos.0]);
                            } else {
                                ctx.focus.as_ref().expect("focus").focus(sel);
                            }
                            Control::Changed
                        } else {
                            Control::Unchanged
                        }
                    }
                    ct_event!(key press CONTROL-'s') | ct_event!(key press 's') => {
                        if let Some((pos, sel)) = self.structure.selected() {
                            if sel.is_focused() {
                                ctx.focus
                                    .as_ref()
                                    .expect("focus")
                                    .focus(&self.structure.splitter);
                            } else {
                                ctx.focus.as_ref().expect("focus").focus(sel);
                            }
                            Control::Changed
                        } else {
                            Control::Unchanged
                        }
                    }
                    _ => Control::Continue,
                });
            }

            flow_ok!(match event {
                ct_event!(key press CONTROL-'n') => {
                    Control::Message(MDAction::MenuNew)
                }
                ct_event!(key press CONTROL-'s') => {
                    Control::Message(MDAction::Save)
                }
                ct_event!(key press CONTROL-'d') => {
                    Control::Message(MDAction::Split)
                }
                ct_event!(key release CONTROL-'w') => {
                    self.window_cmd = true;
                    Control::Changed
                }
                ct_event!(focus_lost) => {
                    Control::Message(MDAction::Save)
                }
                _ => Control::Continue,
            });

            let r = self.structure.crossterm(event, ctx)?;

            // synchronize instances
            if r.is_consumed() {
                let (id_sel, sel_path, replay) =
                    if let Some((id_sel, sel)) = self.structure.selected_mut() {
                        (id_sel, sel.path.clone(), sel.edit.recent_replay())
                    } else {
                        ((0, 0), PathBuf::default(), Vec::default())
                    };
                if !replay.is_empty() {
                    self.structure.replay(id_sel, &sel_path, &replay);
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
                MDAction::MenuNew => {
                    ctx.g.file_dlg.engage(
                        |w| {
                            w.save_dialog(".", "")?;
                            Ok(Control::Changed)
                        },
                        |p| Ok(Control::Message(MDAction::New(p))),
                    )?
                }
                MDAction::MenuOpen => ctx.g.file_dlg.engage(
                    |w| {
                        w.open_dialog(".")?;
                        Ok(Control::Changed)
                    },
                    |p| Ok(Control::Message(MDAction::Open(p))),
                )?,
                MDAction::MenuSave => Control::Message(MDAction::Save),
                MDAction::MenuSaveAs => ctx.g.file_dlg.engage(
                    |w| {
                        w.save_dialog(".", "")?;
                        Ok(Control::Changed)
                    },
                    |p| Ok(Control::Message(MDAction::SaveAs(p))),
                )?,

                MDAction::New(p) => {
                    self.new(p, ctx)?;
                    Control::Changed
                }
                MDAction::Open(p) => {
                    self.open(p, ctx)?;
                    Control::Changed
                }
                MDAction::CloseSelected => {
                    if let Some(pos) = self.structure.selected_pos() {
                        self.structure.close((pos.0, pos.1), ctx)?;
                        Control::Changed
                    } else {
                        Control::Continue
                    }
                }
                MDAction::Close(idx_split, idx_tab) => {
                    self.structure.close((*idx_split, *idx_tab), ctx)?;
                    Control::Changed
                }
                MDAction::Select(idx_split, idx_tab) => {
                    self.structure.select((*idx_split, *idx_tab), ctx);
                    Control::Changed
                }
                MDAction::Save => {
                    self.save()?;
                    Control::Changed
                }
                MDAction::SaveAs(p) => {
                    self.save_as(p)?;
                    Control::Changed
                }
                MDAction::Split => {
                    self.split(ctx)?;
                    Control::Changed
                }
                _ => Control::Continue,
            });
            Ok(Control::Continue)
        }
    }

    impl MDEditState {
        fn check_ext(path: &Path) -> PathBuf {
            let mut path = path.to_path_buf();
            if path.extension().is_none() {
                path.set_extension("md");
            }
            path
        }

        pub fn new(&mut self, path: &Path, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            let path = Self::check_ext(path);

            let mut new = MDFileState::new(&path);
            new.changed = true;

            let pos = if let Some(pos) = self.structure.selected_pos() {
                (pos.0, pos.1 + 1)
            } else {
                (0, 0)
            };

            self.structure.show_edit(pos, new);

            ctx.focus = Some(self.focus());
            self.structure.select(pos, ctx);

            Ok(())
        }

        pub fn open(&mut self, path: &Path, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            let new = if let Some((_, md)) = self.structure.for_path_mut(path) {
                md.edit.undo_buffer_mut().expect("undo").set_replay(true);
                md.clone()
            } else {
                let mut new = MDFileState::new(path);
                new.open(path, ctx)?;
                new
            };

            let pos = if let Some(pos) = self.structure.selected_pos() {
                (pos.0, pos.1 + 1)
            } else {
                (0, 0)
            };

            self.structure.show_edit(pos, new);

            ctx.focus = Some(self.focus());
            self.structure.select(pos, ctx);

            Ok(())
        }

        pub fn split(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            let Some((pos, sel)) = self.structure.selected_mut() else {
                return Ok(());
            };

            sel.edit.undo_buffer_mut().expect("undo").set_replay(true);

            let new = sel.clone();

            let new_pos = if pos.0 + 1 == self.structure.tabs.len() {
                (pos.0 + 1, 0)
            } else {
                (pos.0 + 1, self.structure.tabs[pos.0 + 1].len())
            };
            self.structure.show_edit(new_pos, new);

            ctx.focus = Some(self.focus());
            self.structure.select(pos, ctx);

            Ok(())
        }

        pub fn save(&mut self) -> Result<(), Error> {
            self.structure.save()?;
            Ok(())
        }

        pub fn save_as(&mut self, path: &Path) -> Result<(), Error> {
            let path = Self::check_ext(path);
            if let Some((pos, t)) = self.structure.selected_mut() {
                t.save_as(&path)?;
            }
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
