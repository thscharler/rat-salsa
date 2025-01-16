#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unreachable_pub)]

use crate::facilities::MDFileDialogState;
use crate::root::{MDRoot, MDRootState};
use anyhow::Error;
use rat_salsa::poll::{PollCrossterm, PollRendered, PollTasks, PollTimers};
use rat_salsa::rendered::RenderedEvent;
use rat_salsa::timer::TimeOut;
use rat_salsa::{run_tui, RunConfig};
use rat_theme::dark_theme::DarkTheme;
use rat_theme::scheme::IMPERIAL;
use rat_theme::Scheme;
use rat_widget::msgdialog::MsgDialogState;
use rat_widget::statusline::StatusLineState;
use std::fs;
use std::path::PathBuf;

mod mdedit_parts;

type AppContext<'a> = rat_salsa::AppContext<'a, GlobalState, MDEvent, Error>;

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

    let app = MDRoot;
    let mut state = MDRootState::default();

    run_tui(
        app,
        &mut global,
        &mut state,
        RunConfig::default()?
            .threads(1)
            .poll(PollCrossterm)
            .poll(PollTasks)
            .poll(PollTimers)
            .poll(PollRendered),
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
    pub message_dlg: MsgDialogState,
    pub file_dlg: MDFileDialogState,
}

impl GlobalState {
    fn new(cfg: MDConfig, theme: DarkTheme) -> Self {
        Self {
            cfg,
            theme,
            status: Default::default(),
            error_dlg: Default::default(),
            message_dlg: Default::default(),
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

#[derive(Debug)]
pub struct MDConfig {
    pub show_ctrl: bool,
    pub new_line: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum MDEvent {
    Event(crossterm::event::Event),
    TimeOut(TimeOut),
    Rendered,

    Message(String),

    MenuNew,
    MenuOpen,
    MenuSave,
    MenuSaveAs,

    CfgShowCtrl,
    CfgNewline,

    SyncEdit,

    New(PathBuf),
    Open(PathBuf),
    SelectOrOpen(PathBuf),
    SelectOrOpenSplit(PathBuf),
    SaveAs(PathBuf),
    Save,
    Split,
    JumpToFiles,
    HideFiles,
    Close,
    CloseAt(usize, usize),
    SelectAt(usize, usize),
}

impl From<RenderedEvent> for MDEvent {
    fn from(_: RenderedEvent) -> Self {
        Self::Rendered
    }
}

impl From<crossterm::event::Event> for MDEvent {
    fn from(value: crossterm::event::Event) -> Self {
        Self::Event(value)
    }
}

impl From<TimeOut> for MDEvent {
    fn from(value: TimeOut) -> Self {
        Self::TimeOut(value)
    }
}

// -----------------------------------------------------------------------
// -----------------------------------------------------------------------
mod root {
    use crate::app::{MDApp, MDAppState};
    use crate::{AppContext, GlobalState, MDEvent};
    use anyhow::Error;
    use rat_salsa::{AppState, AppWidget, Control, RenderContext};
    use rat_widget::event::{ct_event, try_flow, ConsumedEvent};
    use rat_widget::focus::FocusBuilder;
    use rat_widget::statusline::StatusLine;
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Layout, Rect};
    use ratatui::prelude::StatefulWidget;
    use std::time::{Duration, SystemTime};

    #[derive(Debug)]
    pub struct MDRoot;

    #[derive(Debug, Default)]
    pub struct MDRootState {
        pub app: MDAppState,
    }

    impl AppWidget<GlobalState, MDEvent, Error> for MDRoot {
        type State = MDRootState;

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

            MDApp.render(area, buf, &mut state.app, ctx)?;

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
                    ctx.g.theme.status_base(),
                    ctx.g.theme.deepblue(3),
                    ctx.g.theme.deepblue(2),
                    ctx.g.theme.deepblue(1),
                    ctx.g.theme.deepblue(0),
                ]);
            status.render(s[1], buf, &mut ctx.g.status);

            Ok(())
        }
    }

    impl AppState<GlobalState, MDEvent, Error> for MDRootState {
        fn init(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            self.app.init(ctx)
        }

        fn event(
            &mut self,
            event: &MDEvent,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, MDEvent, Error>,
        ) -> Result<Control<MDEvent>, Error> {
            let t0 = SystemTime::now();

            let mut r = match event {
                MDEvent::Event(event) => {
                    try_flow!(match &event {
                        ct_event!(resized) => Control::Changed,
                        ct_event!(key press CONTROL-'q') => Control::Quit,
                        _ => Control::Continue,
                    });

                    Control::Continue
                }
                MDEvent::Rendered => {
                    // rebuild keyboard + mouse focus
                    ctx.focus = Some(FocusBuilder::rebuild_for(&self.app, ctx.focus.take()));
                    Control::Continue
                }
                _ => Control::Continue,
            };

            r = r.or_else_try(|| self.app.event(event, ctx))?;

            if r == Control::Changed {
                let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
                ctx.g.status.status(4, format!("T {:.0?}", el).to_string());
            }

            Ok(r)
        }

        fn error(&self, event: Error, ctx: &mut AppContext<'_>) -> Result<Control<MDEvent>, Error> {
            self.app.error(event, ctx)
        }
    }
}

// -----------------------------------------------------------------------
// -----------------------------------------------------------------------
pub mod facilities {
    use crate::MDEvent;
    use anyhow::Error;
    use crossterm::event::Event;
    use rat_salsa::Control;
    use rat_widget::event::{try_flow, Dialog, FileOutcome, HandleEvent};
    use rat_widget::file_dialog::{FileDialog, FileDialogState, FileDialogStyle};
    use rat_widget::text::HasScreenCursor;
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
        pub handle: Option<fn(PathBuf) -> Result<Control<MDEvent>, Error>>,
    }

    impl Facility<FileDialogState, PathBuf, MDEvent, Error> for MDFileDialogState {
        fn engage(
            &mut self,
            prepare: impl FnOnce(&mut FileDialogState) -> Result<Control<MDEvent>, Error>,
            handle: fn(PathBuf) -> Result<Control<MDEvent>, Error>,
        ) -> Result<Control<MDEvent>, Error> {
            let r = prepare(&mut self.file_dlg);
            if r.is_ok() {
                self.handle = Some(handle);
            }
            r
        }

        fn handle(&mut self, event: &Event) -> Result<Control<MDEvent>, Error> {
            try_flow!(match self.file_dlg.handle(event, Dialog)? {
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

    impl HasScreenCursor for MDFileDialogState {
        fn screen_cursor(&self) -> Option<(u16, u16)> {
            self.file_dlg.screen_cursor()
        }
    }
}

// -----------------------------------------------------------------------
// -----------------------------------------------------------------------
mod app {
    use crate::facilities::{Facility, MDFileDialog};
    use crate::mdedit::{MDEdit, MDEditState};
    use crate::{AppContext, GlobalState, MDEvent, CHEAT, HELP};
    use anyhow::Error;
    use rat_salsa::{AppState, AppWidget, Control, RenderContext};
    use rat_theme::dark_themes;
    use rat_widget::event::{
        ct_event, try_flow, ConsumedEvent, Dialog, HandleEvent, MenuOutcome, Popup, Regular,
    };
    use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus};
    use rat_widget::layout::layout_middle;
    use rat_widget::menu::{MenuBuilder, MenuStructure, Menubar, MenubarState, Separator};
    use rat_widget::msgdialog::MsgDialog;
    use rat_widget::popup::Placement;
    use rat_widget::text::HasScreenCursor;
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Layout, Rect};
    use ratatui::prelude::{StatefulWidget, Style};
    use ratatui::widgets::{Block, BorderType, Padding};
    use std::str::from_utf8;

    #[derive(Debug)]
    struct Menu {
        show_ctrl: bool,
        use_crlf: bool,
    }

    impl<'a> MenuStructure<'a> for Menu {
        fn menus(&'a self, menu: &mut MenuBuilder<'a>) {
            menu.item_parsed("_File")
                .item_parsed("_Edit")
                .item_parsed("_View")
                .item_parsed("_Theme")
                .item_parsed("_Quit");
        }

        fn submenu(&'a self, n: usize, submenu: &mut MenuBuilder<'a>) {
            match n {
                0 => {
                    submenu.item_parsed("_New..|Ctrl-N");
                    submenu.item_parsed("_Open..|Ctrl-O");
                    submenu.item_parsed("_Save..|Ctrl-S");
                    submenu.item_parsed("Save _as..");
                }
                1 => {
                    submenu.item_parsed("Format Item|Alt-F");
                    submenu.item_parsed("Alt-Format Item|Alt-Shift-F");
                }
                2 => {
                    if self.show_ctrl {
                        submenu.item_parsed("\u{2611} Control chars");
                    } else {
                        submenu.item_parsed("\u{2610} Control chars");
                    }
                    if self.use_crlf {
                        submenu.item_parsed("\u{2611} Use CR+LF");
                    } else {
                        submenu.item_parsed("\u{2610} Use CR+LF");
                    }
                    submenu.separator(Separator::Dotted);
                    submenu.item_parsed("_Split view|Ctrl-W D");
                    submenu.item_parsed("_Jump to File|F5");
                    submenu.item_parsed("_Hide files|F6");
                }
                3 => {
                    for t in dark_themes() {
                        submenu.item_string(t.name().into());
                    }
                }
                _ => {}
            }
        }
    }

    #[derive(Debug)]
    pub struct MDApp;

    #[derive(Debug)]
    pub struct MDAppState {
        pub editor: MDEditState,
        pub menu: MenubarState,
    }

    impl Default for MDAppState {
        fn default() -> Self {
            let s = Self {
                editor: MDEditState::default(),
                menu: MenubarState::named("menu"),
            };
            s
        }
    }

    impl AppWidget<GlobalState, MDEvent, Error> for MDApp {
        type State = MDAppState;

        fn render(
            &self,
            area: Rect,
            buf: &mut Buffer,
            state: &mut Self::State,
            ctx: &mut RenderContext<'_, GlobalState>,
        ) -> Result<(), Error> {
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
                .popup_width(25)
                .popup_block(Block::bordered())
                .popup_placement(Placement::Above)
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
                            .style(ctx.g.theme.dialog_base())
                            .border_type(BorderType::Rounded)
                            .title_style(Style::new().fg(ctx.g.scheme().red[0]))
                            .padding(Padding::new(1, 1, 1, 1)),
                    )
                    .styles(ctx.g.theme.msg_dialog_style());
                err.render(l_msg, buf, &mut ctx.g.error_dlg);
            }

            if ctx.g.message_dlg.active() {
                let l_msg = layout_middle(
                    r[0],
                    Constraint::Percentage(4),
                    Constraint::Percentage(4),
                    Constraint::Percentage(4),
                    Constraint::Percentage(4),
                );
                let err = MsgDialog::new()
                .block(
                    Block::bordered()
                        .style(
                            Style::default() //
                                .fg(ctx.g.theme.scheme().white[2])
                                .bg(ctx.g.theme.scheme().deepblue[0]),
                        )
                        .border_type(BorderType::Rounded)
                        .title_style(Style::new().fg(ctx.g.scheme().bluegreen[0]))
                        .padding(Padding::new(1, 1, 1, 1)),
                )
                .styles(ctx.g.theme.msg_dialog_style());
                err.render(l_msg, buf, &mut ctx.g.message_dlg);
            }

            Ok(())
        }
    }

    impl HasFocus for MDAppState {
        fn build(&self, builder: &mut FocusBuilder) {
            builder.widget(&self.menu);
            builder.widget(&self.editor);
        }

        fn focus(&self) -> FocusFlag {
            unimplemented!("don't use this")
        }

        fn area(&self) -> Rect {
            unimplemented!("don't use this")
        }
    }

    impl AppState<GlobalState, MDEvent, Error> for MDAppState {
        fn init(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            self.menu.bar.select(Some(0));
            self.menu.focus().set(true);
            self.editor.init(ctx)?;
            Ok(())
        }

        fn event(
            &mut self,
            event: &MDEvent,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, MDEvent, Error>,
        ) -> Result<Control<MDEvent>, Error> {
            let mut r = match event {
                MDEvent::Event(event) => self.crossterm(event, ctx)?,
                _ => self.other(event, ctx)?,
            };

            r = r.or_else_try(|| self.editor.event(event, ctx))?;

            if self.editor.set_active_split() {
                self.editor.sync_views(ctx)?;
            }

            Ok(r)
        }

        fn error(&self, event: Error, ctx: &mut AppContext<'_>) -> Result<Control<MDEvent>, Error> {
            ctx.g.error_dlg.title("Error occured");
            ctx.g.error_dlg.append(format!("{:?}", &*event).as_str());
            Ok(Control::Changed)
        }
    }

    impl MDAppState {
        fn crossterm(
            &mut self,
            event: &crossterm::event::Event,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<MDEvent>, Error> {
            try_flow!(ctx.g.error_dlg.handle(event, Dialog));
            try_flow!(ctx.g.message_dlg.handle(event, Dialog));
            try_flow!(ctx.g.file_dlg.handle(event)?);

            let f = Control::from(ctx.focus_mut().handle(event, Regular));
            ctx.queue(f);

            // regular global
            let mut r = match &event {
                ct_event!(keycode press Esc) => {
                    if !self.menu.is_focused() {
                        ctx.focus().focus(&self.menu);
                        Control::Changed
                    } else {
                        if let Some((_, last_edit)) = self.editor.split_tab.selected() {
                            ctx.focus().focus(last_edit);
                            Control::Changed
                        } else {
                            Control::Continue
                        }
                    }
                }
                ct_event!(keycode press F(1)) => {
                    let txt = from_utf8(HELP)?;
                    let mut txt2 = String::new();
                    for l in txt.lines() {
                        txt2.push_str(l);
                        txt2.push('\n');
                    }
                    ctx.g.message_dlg.append(&txt2);
                    Control::Changed
                }
                ct_event!(keycode press F(2)) => {
                    let txt = from_utf8(CHEAT)?;
                    let mut txt2 = String::new();
                    for l in txt.lines() {
                        txt2.push_str(l);
                        txt2.push('\n');
                    }
                    ctx.g.message_dlg.append(&txt2);
                    Control::Changed
                }
                _ => Control::Continue,
            };

            r = r.or_else(|| match self.menu.handle(event, Popup) {
                MenuOutcome::MenuActivated(0, 0) => Control::Message(MDEvent::MenuNew),
                MenuOutcome::MenuActivated(0, 1) => Control::Message(MDEvent::MenuOpen),
                MenuOutcome::MenuActivated(0, 2) => Control::Message(MDEvent::MenuSave),
                MenuOutcome::MenuActivated(0, 3) => Control::Message(MDEvent::MenuSaveAs),
                MenuOutcome::MenuActivated(1, 0) => {
                    if let Some((_, sel)) = self.editor.split_tab.selected_mut() {
                        ctx.focus().focus(sel);
                        sel.md_format_table(false, ctx)
                    } else {
                        Control::Continue
                    }
                }
                MenuOutcome::MenuActivated(1, 1) => {
                    if let Some((_, sel)) = self.editor.split_tab.selected_mut() {
                        ctx.focus().focus(sel);
                        sel.md_format_table(true, ctx)
                    } else {
                        Control::Continue
                    }
                }
                MenuOutcome::MenuActivated(2, 0) => {
                    ctx.g.cfg.show_ctrl = !ctx.g.cfg.show_ctrl;
                    Control::Message(MDEvent::CfgShowCtrl)
                }
                MenuOutcome::MenuActivated(2, 1) => {
                    if ctx.g.cfg.new_line == "\r\n" {
                        ctx.g.cfg.new_line = "\n".into();
                    } else {
                        ctx.g.cfg.new_line = "\r\n".into();
                    }
                    Control::Message(MDEvent::CfgNewline)
                }
                MenuOutcome::MenuActivated(2, 2) => Control::Message(MDEvent::Split),
                MenuOutcome::MenuActivated(2, 3) => Control::Message(MDEvent::JumpToFiles),
                MenuOutcome::MenuActivated(2, 4) => Control::Message(MDEvent::HideFiles),
                MenuOutcome::MenuSelected(3, n) => {
                    ctx.g.theme = dark_themes()[n].clone();
                    Control::Changed
                }
                r => r.into(),
            });

            r = r.or_else(|| match self.menu.handle(event, Regular) {
                MenuOutcome::Activated(4) => Control::Quit,
                r => r.into(),
            });

            Ok(r)
        }

        fn other(
            &mut self,
            event: &MDEvent,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<MDEvent>, Error> {
            try_flow!(match event {
                MDEvent::Message(s) => {
                    ctx.g.status.status(0, &*s);
                    Control::Changed
                }
                MDEvent::MenuNew => {
                    ctx.g.file_dlg.engage(
                        |w| {
                            w.save_dialog_ext(".", "", "md")?;
                            Ok(Control::Changed)
                        },
                        |p| Ok(Control::Message(MDEvent::New(p))),
                    )?
                }
                MDEvent::MenuOpen => {
                    ctx.g.file_dlg.engage(
                        |w| {
                            w.open_dialog(".")?;
                            Ok(Control::Changed)
                        },
                        |p| Ok(Control::Message(MDEvent::Open(p))),
                    )?
                }
                MDEvent::MenuSave => {
                    Control::Message(MDEvent::Save)
                }
                MDEvent::MenuSaveAs => {
                    ctx.g.file_dlg.engage(
                        |w| {
                            w.save_dialog(".", "")?;
                            Ok(Control::Changed)
                        },
                        |p| Ok(Control::Message(MDEvent::SaveAs(p))),
                    )?
                }
                _ => Control::Continue,
            });

            Ok(Control::Continue)
        }
    }
}

// -----------------------------------------------------------------------
// Editor for a single file.
// -----------------------------------------------------------------------
pub mod mdfile {
    use crate::mdedit_parts::format::md_format;
    use crate::mdedit_parts::styles::parse_md_styles;
    use crate::mdedit_parts::MarkDown;
    use crate::{AppContext, GlobalState, MDEvent};
    use anyhow::Error;
    use log::warn;
    use rat_salsa::timer::{TimerDef, TimerHandle};
    use rat_salsa::{AppState, AppWidget, Control, RenderContext};
    use rat_widget::event::{HandleEvent, TextOutcome};
    use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus, Navigation};
    use rat_widget::line_number::{LineNumberState, LineNumbers};
    use rat_widget::scrolled::Scroll;
    use rat_widget::text::clipboard::{Clipboard, ClipboardError};
    use rat_widget::text::{upos_type, HasScreenCursor};
    use rat_widget::textarea::{TextArea, TextAreaState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::prelude::{StatefulWidget, Style};
    use ratatui::style::Stylize;
    use ratatui::widgets::{Block, BorderType, Borders};
    use std::fs;
    use std::fs::File;
    use std::io::{BufWriter, Write};
    use std::path::{Path, PathBuf};
    use std::time::{Duration, Instant};

    #[derive(Debug, Default, Clone)]
    pub struct MDFile {
        // vary start margin of the scrollbar
        pub start_margin: u16,
    }

    #[derive(Debug)]
    pub struct MDFileState {
        pub path: PathBuf,
        pub changed: bool,
        pub edit: TextAreaState,
        pub linenr: LineNumberState,
        pub parse_timer: Option<TimerHandle>,
    }

    impl Clone for MDFileState {
        fn clone(&self) -> Self {
            Self {
                path: self.path.clone(),
                changed: self.changed,
                edit: self.edit.clone(),
                linenr: Default::default(),
                parse_timer: None,
            }
        }
    }

    impl AppWidget<GlobalState, MDEvent, Error> for MDFile {
        type State = MDFileState;

        fn render(
            &self,
            area: Rect,
            buf: &mut Buffer,
            state: &mut Self::State,
            ctx: &mut RenderContext<'_, GlobalState>,
        ) -> Result<(), Error> {
            let line_nr = LineNumbers::new()
                .start(state.edit.offset().1 as upos_type)
                .end(state.edit.len_lines())
                .cursor(state.edit.cursor().y)
                // .relative(true)
                .styles(ctx.g.theme.line_nr_style());

            let line_nr_area = Rect::new(area.x, area.y, line_nr.width(), area.height);
            let text_area = Rect::new(
                area.x + line_nr.width(),
                area.y,
                area.width.saturating_sub(line_nr.width()),
                area.height,
            );

            line_nr.render(line_nr_area, buf, &mut state.linenr);

            TextArea::new()
                .styles(ctx.g.theme.textarea_style())
                .block(
                    Block::new()
                        .border_type(BorderType::Rounded)
                        .borders(Borders::RIGHT),
                )
                .vscroll(
                    Scroll::new()
                        .start_margin(self.start_margin)
                        .styles(ctx.g.theme.scroll_style()),
                )
                .text_style(text_style(ctx))
                .render(text_area, buf, &mut state.edit);
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

    fn text_style(ctx: &mut RenderContext<'_, GlobalState>) -> [Style; 34] {
        // base-style: Style::default().fg(self.s.white[0]).bg(self.s.black[1])
        [
            Style::default().fg(ctx.g.scheme().yellow[2]).underlined(), // Heading1,
            Style::default().fg(ctx.g.scheme().yellow[1]).underlined(), // Heading2,
            Style::default().fg(ctx.g.scheme().yellow[1]).underlined(), // Heading3,
            Style::default().fg(ctx.g.scheme().orange[2]).underlined(), // Heading4,
            Style::default().fg(ctx.g.scheme().orange[1]).underlined(), // Heading5,
            Style::default().fg(ctx.g.scheme().orange[1]).underlined(), // Heading6,
            //
            Style::default(),                               // Paragraph
            Style::default().fg(ctx.g.scheme().orange[2]),  // BlockQuote,
            Style::default().fg(ctx.g.scheme().redpink[2]), // CodeBlock,
            Style::default().fg(ctx.g.scheme().redpink[2]), // MathDisplay
            Style::default().fg(ctx.g.scheme().white[3]),   // Rule
            Style::default().fg(ctx.g.scheme().gray[3]),    // Html
            //
            Style::default().fg(ctx.g.scheme().bluegreen[2]), // Link
            Style::default().fg(ctx.g.scheme().bluegreen[2]), // LinkDef
            Style::default().fg(ctx.g.scheme().bluegreen[2]), // Image
            Style::default().fg(ctx.g.scheme().bluegreen[3]), // Footnote Definition
            Style::default().fg(ctx.g.scheme().bluegreen[2]), // Footnote Reference
            //
            Style::default(),                              // List
            Style::default(),                              // Item
            Style::default().fg(ctx.g.scheme().orange[2]), // TaskListMarker
            Style::default().fg(ctx.g.scheme().orange[2]), // ItemTag
            Style::default(),                              // DefinitionList
            Style::default().fg(ctx.g.scheme().orange[3]), // DefinitionListTitle
            Style::default().fg(ctx.g.scheme().orange[2]), // DefinitionListDefinition
            //
            Style::default(),                            // Table
            Style::default().fg(ctx.g.scheme().gray[3]), // Table-Head
            Style::default().fg(ctx.g.scheme().gray[3]), // Table-Row
            Style::default().fg(ctx.g.scheme().gray[3]), // Table-Cell
            //
            Style::default().fg(ctx.g.scheme().white[0]).italic(), // Emphasis
            Style::default().fg(ctx.g.scheme().white[3]).bold(),   // Strong
            Style::default().fg(ctx.g.scheme().gray[3]).crossed_out(), // Strikethrough
            Style::default().fg(ctx.g.scheme().redpink[2]),        // CodeInline
            Style::default().fg(ctx.g.scheme().redpink[2]),        // MathInline
            //
            Style::default().fg(ctx.g.scheme().orange[2]), // MetadataBlock
        ]
    }

    impl HasFocus for MDFileState {
        fn build(&self, builder: &mut FocusBuilder) {
            builder.append_leaf(self);
        }

        fn focus(&self) -> FocusFlag {
            self.edit.focus()
        }

        fn area(&self) -> Rect {
            self.edit.area()
        }

        fn navigable(&self) -> Navigation {
            self.edit.navigable()
        }
    }

    #[derive(Debug, Default, Clone)]
    struct CliClipboard;

    impl Clipboard for CliClipboard {
        fn get_string(&self) -> Result<String, ClipboardError> {
            match cli_clipboard::get_contents() {
                Ok(v) => Ok(v),
                Err(e) => {
                    warn!("{:?}", e);
                    Err(ClipboardError)
                }
            }
        }

        fn set_string(&self, s: &str) -> Result<(), ClipboardError> {
            match cli_clipboard::set_contents(s.to_string()) {
                Ok(_) => Ok(()),
                Err(e) => {
                    warn!("{:?}", e);
                    Err(ClipboardError)
                }
            }
        }
    }

    impl AppState<GlobalState, MDEvent, Error> for MDFileState {
        fn event(
            &mut self,
            event: &MDEvent,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, MDEvent, Error>,
        ) -> Result<Control<MDEvent>, Error> {
            let r = match event {
                MDEvent::TimeOut(event) => {
                    if self.parse_timer == Some(event.handle) {
                        self.parse_markdown();
                        Control::Changed
                    } else {
                        Control::Continue
                    }
                }
                MDEvent::Event(event) => {
                    // call markdown event-handling instead of regular.
                    match self.edit.handle(event, MarkDown) {
                        TextOutcome::TextChanged => self.text_changed(ctx),
                        r => r.into(),
                    }
                }
                MDEvent::CfgNewline => {
                    self.edit.set_newline(&ctx.g.cfg.new_line);
                    Control::Continue
                }
                MDEvent::CfgShowCtrl => {
                    self.edit.set_show_ctrl(ctx.g.cfg.show_ctrl);
                    Control::Continue
                }
                _ => Control::Continue,
            };

            Ok(r)
        }
    }

    impl MDFileState {
        // New editor with fresh file.
        pub fn new_file(path: &Path, ctx: &mut AppContext<'_>) -> Self {
            let mut path = path.to_path_buf();
            if path.extension().is_none() {
                path.set_extension("md");
            }

            let mut text_area = TextAreaState::named(
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .as_ref(),
            );
            text_area.set_clipboard(Some(CliClipboard));
            text_area.set_newline(&ctx.g.cfg.new_line);
            text_area.set_show_ctrl(ctx.g.cfg.show_ctrl);
            text_area.set_tab_width(4);

            Self {
                path: path.clone(),
                changed: true,
                edit: text_area,
                linenr: Default::default(),
                parse_timer: None,
            }
        }

        // New editor with existing file.
        pub fn open_file(path: &Path, ctx: &mut AppContext<'_>) -> Result<Self, Error> {
            let path = PathBuf::from(path);

            let mut text_area = TextAreaState::named(
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .as_ref(),
            );
            text_area.set_clipboard(Some(CliClipboard));
            let t = fs::read_to_string(&path)?;
            text_area.set_text(t.as_str());
            text_area.set_newline(&ctx.g.cfg.new_line);
            text_area.set_show_ctrl(ctx.g.cfg.show_ctrl);
            text_area.set_tab_width(4);

            Ok(Self {
                path: path.clone(),
                changed: false,
                edit: text_area,
                linenr: Default::default(),
                parse_timer: Some(
                    ctx.add_timer(TimerDef::new().next(Instant::now() + Duration::from_millis(0))),
                ),
            })
        }

        // Save as
        pub fn save_as(&mut self, path: &Path) -> Result<(), Error> {
            self.path = path.into();
            self.save()
        }

        // Save
        pub fn save(&mut self) -> Result<(), Error> {
            if self.changed {
                let mut f = BufWriter::new(File::create(&self.path)?);
                let mut buf = Vec::new();
                for line in self.edit.text().lines() {
                    buf.extend(line.bytes());
                    buf.extend_from_slice(self.edit.newline().as_bytes());
                }
                f.write_all(&buf)?;

                self.changed = false;
            }
            Ok(())
        }

        // Parse & set styles.
        pub fn parse_markdown(&mut self) {
            let styles = parse_md_styles(&self.edit);
            self.edit.set_styles(styles);
        }

        // Format selected table
        pub fn md_format_table(
            &mut self,
            eq_width: bool,
            ctx: &mut AppContext<'_>,
        ) -> Control<MDEvent> {
            match md_format(&mut self.edit, eq_width) {
                TextOutcome::TextChanged => self.text_changed(ctx),
                r => r.into(),
            }
        }

        // Flag any text-changes.
        pub fn text_changed(&mut self, ctx: &mut AppContext<'_>) -> Control<MDEvent> {
            self.changed = true;
            // send sync
            ctx.queue(Control::Message(MDEvent::SyncEdit));
            // restart timer
            self.parse_timer = Some(ctx.replace_timer(
                self.parse_timer,
                TimerDef::new().next(Instant::now() + Duration::from_millis(200)),
            ));
            Control::Changed
        }
    }
}

// -----------------------------------------------------------------------
// combined split + tab structure
// -----------------------------------------------------------------------
pub mod split_tab {
    use crate::mdfile::{MDFile, MDFileState};
    use crate::{AppContext, GlobalState, MDEvent};
    use anyhow::Error;
    use rat_salsa::timer::TimerDef;
    use rat_salsa::{AppState, AppWidget, Control, RenderContext};
    use rat_widget::event::{try_flow, ConsumedEvent, HandleEvent, Regular, TabbedOutcome};
    use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus};
    use rat_widget::splitter::{Split, SplitState, SplitType};
    use rat_widget::tabbed::{TabType, Tabbed, TabbedState};
    use rat_widget::text::undo_buffer::UndoEntry;
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Rect};
    use ratatui::prelude::{Line, StatefulWidget};
    use ratatui::symbols;
    use std::path::Path;
    use std::time::{Duration, Instant};

    #[derive(Debug, Default)]
    pub struct SplitTab;

    #[derive(Debug)]
    pub struct SplitTabState {
        pub container: FocusFlag,
        pub splitter: SplitState,
        pub sel_split: Option<usize>,
        pub sel_tab: Option<usize>,
        pub tabbed: Vec<TabbedState>,
        pub tabs: Vec<Vec<MDFileState>>,
    }

    impl Default for SplitTabState {
        fn default() -> Self {
            Self {
                container: FocusFlag::named("split_tab"),
                splitter: SplitState::named("splitter"),
                sel_split: None,
                sel_tab: None,
                tabbed: vec![],
                tabs: vec![],
            }
        }
    }

    impl AppWidget<GlobalState, MDEvent, Error> for SplitTab {
        type State = SplitTabState;

        fn render(
            &self,
            area: Rect,
            buf: &mut Buffer,
            state: &mut Self::State,
            ctx: &mut RenderContext<'_, GlobalState>,
        ) -> Result<(), Error> {
            let (s0, s1) = Split::new()
                .constraints(vec![Constraint::Fill(1); state.tabbed.len()])
                .mark_offset(0)
                .split_type(SplitType::Scroll)
                .styles(ctx.g.theme.split_style())
                .direction(Direction::Horizontal)
                .into_widgets();

            s0.render(area, buf, &mut state.splitter);

            let max_idx_split = state.splitter.widget_areas.len().saturating_sub(1);
            for (idx_split, edit_area) in state.splitter.widget_areas.iter().enumerate() {
                let select_style = if let Some((sel_pos, md)) = state.selected() {
                    if sel_pos.0 == idx_split {
                        if state.tabbed[idx_split].is_focused() {
                            ctx.g.theme.tabbed_style().focus.expect("style")
                        } else if md.is_focused() {
                            ctx.g.theme.primary(1)
                        } else {
                            ctx.g.theme.tabbed_style().select.expect("style")
                        }
                    } else {
                        ctx.g.theme.tabbed_style().select.expect("style")
                    }
                } else {
                    ctx.g.theme.tabbed_style().select.expect("style")
                };

                Tabbed::new()
                    .tab_type(TabType::Attached)
                    .closeable(true)
                    .styles(ctx.g.theme.tabbed_style())
                    .select_style(select_style)
                    .tabs(state.tabs[idx_split].iter().map(|v| {
                        let title = v.path.file_name().unwrap_or_default().to_string_lossy();
                        let title = format!(
                            "{}{}",
                            v.path.file_name().unwrap_or_default().to_string_lossy(),
                            if v.changed { " \u{1F5AB}" } else { "" }
                        );
                        Line::from(title)
                    }))
                    .render(*edit_area, buf, &mut state.tabbed[idx_split]);

                // fix block rendering
                let fix_area = state.tabbed[idx_split].block_area;
                if let Some(cell) = buf.cell_mut((fix_area.right() - 1, fix_area.y)) {
                    cell.set_symbol(symbols::line::ROUNDED_TOP_RIGHT);
                }

                if let Some(idx_tab) = state.tabbed[idx_split].selected() {
                    MDFile {
                        start_margin: if max_idx_split == idx_split { 0 } else { 1 },
                    }
                    .render(
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

    impl HasFocus for SplitTabState {
        fn build(&self, builder: &mut FocusBuilder) {
            let tag = builder.start(self);
            builder.widget(&self.splitter);
            for (idx_split, tabbed) in self.tabbed.iter().enumerate() {
                builder.widget(&self.tabbed[idx_split]);
                if let Some(idx_tab) = tabbed.selected() {
                    builder.widget(&self.tabs[idx_split][idx_tab]);
                }
            }
            builder.end(tag);
        }

        fn focus(&self) -> FocusFlag {
            self.container.clone()
        }

        fn area(&self) -> Rect {
            Rect::default()
        }
    }

    impl AppState<GlobalState, MDEvent, Error> for SplitTabState {
        fn event(
            &mut self,
            event: &MDEvent,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, MDEvent, Error>,
        ) -> Result<Control<MDEvent>, Error> {
            let mut r = match event {
                MDEvent::Event(event) => {
                    try_flow!(self.splitter.handle(event, Regular));
                    for (idx_split, tabbed) in self.tabbed.iter_mut().enumerate() {
                        try_flow!(match tabbed.handle(event, Regular) {
                            TabbedOutcome::Close(n) => {
                                Control::Message(MDEvent::CloseAt(idx_split, n))
                            }
                            TabbedOutcome::Select(n) => {
                                Control::Message(MDEvent::SelectAt(idx_split, n))
                            }
                            r => r.into(),
                        });
                    }
                    Control::Continue
                }
                _ => Control::Continue,
            };

            r = r.or_else_try(|| {
                match event {
                    MDEvent::Event(_) => {
                        for (idx_split, tabbed) in self.tabbed.iter_mut().enumerate() {
                            if let Some(idx_tab) = tabbed.selected() {
                                try_flow!(self.tabs[idx_split][idx_tab].event(event, ctx)?);
                            }
                        }
                    }
                    _ => {
                        for tab in &mut self.tabs {
                            for ed in tab {
                                try_flow!(ed.event(event, ctx)?);
                            }
                        }
                    }
                }
                Ok::<_, Error>(Control::Continue)
            })?;

            Ok(r)
        }
    }

    impl SplitTabState {
        // Establish the active split+tab using the currently focused tab.
        pub fn set_active_split(&mut self) -> bool {
            // Find which split contains the current focus.
            let old_split = self.sel_split;
            let old_tab = self.sel_tab;

            for (idx_split, tabbed) in self.tabbed.iter().enumerate() {
                if let Some(idx_tab) = tabbed.selected() {
                    if self.tabs[idx_split][idx_tab].is_focused() {
                        self.sel_split = Some(idx_split);
                        self.sel_tab = Some(idx_tab);
                        break;
                    }
                }
            }

            old_split != self.sel_split || old_tab != self.sel_tab
        }

        // Add file at position (split-idx, tab-idx).
        pub fn open(&mut self, pos: (usize, usize), new: MDFileState, ctx: &mut AppContext<'_>) {
            if pos.0 == self.tabs.len() {
                self.tabs.push(Vec::new());
                self.tabbed
                    .push(TabbedState::named(format!("tabbed-{}", pos.0).as_str()));
            }
            if let Some(sel_tab) = self.tabbed[pos.0].selected() {
                if sel_tab >= pos.1 {
                    self.tabbed[pos.0].select(Some(sel_tab + 1));
                }
            } else {
                self.tabbed[pos.0].select(Some(0));
            }
            self.tabs[pos.0].insert(pos.1, new);

            ctx.focus_mut().update_container(self);
        }

        // Close tab (split-idx, tab-idx).
        pub fn close(
            &mut self,
            pos: (usize, usize),
            ctx: &mut AppContext<'_>,
        ) -> Result<(), Error> {
            if pos.0 < self.tabs.len() {
                if pos.1 < self.tabs[pos.0].len() {
                    self.tabs[pos.0][pos.1].save()?;

                    // remove tab
                    self.tabs[pos.0].remove(pos.1);
                    if let Some(sel_tab) = self.tabbed[pos.0].selected() {
                        let new_tab = if sel_tab >= pos.1 {
                            if sel_tab > 0 {
                                Some(sel_tab - 1)
                            } else {
                                None
                            }
                        } else {
                            if sel_tab == 0 {
                                None
                            } else {
                                Some(sel_tab)
                            }
                        };
                        self.tabbed[pos.0].select(new_tab);
                    }

                    // maybe remove split
                    if self.tabs[pos.0].len() == 0 {
                        self.tabs.remove(pos.0);
                        self.tabbed.remove(pos.0);
                        if let Some(sel_split) = self.sel_split {
                            let new_split = if sel_split >= pos.0 {
                                if sel_split > 0 {
                                    Some(sel_split - 1)
                                } else {
                                    None
                                }
                            } else {
                                if sel_split == 0 {
                                    None
                                } else {
                                    Some(sel_split)
                                }
                            };
                            self.sel_split = new_split;
                        }
                    }

                    ctx.focus_mut().update_container(self);
                }
            }
            Ok(())
        }

        // Select by (split-idx, tab-idx)
        pub fn select(&mut self, pos: (usize, usize), ctx: &mut AppContext<'_>) {
            if pos.0 < self.tabs.len() {
                if pos.1 < self.tabs[pos.0].len() {
                    self.sel_split = Some(pos.0);
                    self.tabbed[pos.0].select(Some(pos.1));

                    ctx.focus_mut().update_container(self);
                    ctx.focus().focus(&self.tabs[pos.0][pos.1]);
                }
            }
        }

        // Select next split
        pub fn select_next(&mut self, ctx: &mut AppContext<'_>) -> bool {
            if let Some(idx_split) = self.sel_split {
                if idx_split + 1 < self.tabs.len() {
                    let new_split = idx_split + 1;
                    let new_tab = self.tabbed[new_split].selected().unwrap_or_default();
                    self.select((new_split, new_tab), ctx);
                    return true;
                }
            }
            false
        }

        // Select prev split
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

        // Position of the current focus.
        pub fn selected_pos(&self) -> Option<(usize, usize)> {
            if let Some(idx_split) = self.sel_split {
                if let Some(idx_tab) = self.tabbed[idx_split].selected() {
                    return Some((idx_split, idx_tab));
                }
            }
            None
        }

        // Last known focus and position.
        pub fn selected(&self) -> Option<((usize, usize), &MDFileState)> {
            if let Some(idx_split) = self.sel_split {
                if let Some(idx_tab) = self.tabbed[idx_split].selected() {
                    return Some(((idx_split, idx_tab), &self.tabs[idx_split][idx_tab]));
                }
            }
            None
        }

        // Last known focus and position.
        pub fn selected_mut(&mut self) -> Option<((usize, usize), &mut MDFileState)> {
            if let Some(idx_split) = self.sel_split {
                if let Some(idx_tab) = self.tabbed[idx_split].selected() {
                    return Some(((idx_split, idx_tab), &mut self.tabs[idx_split][idx_tab]));
                }
            }
            None
        }

        // Find the editor for the path.
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

        // Find the editor for the path.
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

        // Save all files.
        pub fn save(&mut self) -> Result<(), Error> {
            for (idx_split, tabs) in self.tabs.iter_mut().enumerate() {
                for (idx_tab, tab) in tabs.iter_mut().enumerate() {
                    tab.save()?
                }
            }
            Ok(())
        }

        // Run the replay for the file at path.
        pub fn replay(
            &mut self,
            id: (usize, usize),
            path: &Path,
            replay: &[UndoEntry],
            ctx: &mut AppContext<'_>,
        ) {
            for (idx_split, tabs) in self.tabs.iter_mut().enumerate() {
                for (idx_tab, tab) in tabs.iter_mut().enumerate() {
                    if id != (idx_split, idx_tab) && tab.path == path {
                        tab.edit.replay_log(replay);
                        // restart timer
                        tab.parse_timer = Some(ctx.replace_timer(
                            tab.parse_timer,
                            TimerDef::new().next(Instant::now() + Duration::from_millis(200)),
                        ));
                    }
                }
            }
        }
    }
}

// -----------------------------------------------------------------------
// md files in current directory.
// -----------------------------------------------------------------------
pub mod file_list {
    use crate::{GlobalState, MDEvent};
    use anyhow::Error;
    use rat_salsa::{AppContext, AppState, AppWidget, Control, RenderContext};
    use rat_widget::event::{ct_event, try_flow, HandleEvent, MenuOutcome, Popup, Regular};
    use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus};
    use rat_widget::list::selection::RowSelection;
    use rat_widget::list::{List, ListState};
    use rat_widget::menu::{PopupConstraint, PopupMenu, PopupMenuState};
    use rat_widget::scrolled::Scroll;
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Layout, Position, Rect};
    use ratatui::prelude::Line;
    use ratatui::widgets::{Block, StatefulWidget};
    use std::fs;
    use std::path::{Path, PathBuf};

    #[derive(Debug, Default)]
    pub struct FileList;

    #[derive(Debug)]
    pub struct FileListState {
        pub container: FocusFlag,
        pub files_dir: PathBuf,
        pub files: Vec<PathBuf>,
        pub file_list: ListState<RowSelection>,

        pub popup_pos: (u16, u16),
        pub popup: PopupMenuState,
    }

    impl Default for FileListState {
        fn default() -> Self {
            Self {
                container: Default::default(),
                files_dir: Default::default(),
                files: vec![],
                file_list: ListState::named("file_list"),
                popup_pos: (0, 0),
                popup: Default::default(),
            }
        }
    }

    impl AppWidget<GlobalState, MDEvent, Error> for FileList {
        type State = FileListState;

        fn render(
            &self,
            area: Rect,
            buf: &mut Buffer,
            state: &mut Self::State,
            ctx: &mut RenderContext<'_, GlobalState>,
        ) -> Result<(), Error> {
            let l_file_list =
                Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).split(area);

            buf.set_style(l_file_list[0], ctx.g.theme.container_base());

            List::default()
                .styles(ctx.g.theme.list_style())
                .scroll(Scroll::new().styles(ctx.g.theme.scroll_style()))
                .items(state.files.iter().map(|v| {
                    if let Some(name) = v.file_name() {
                        Line::from(name.to_string_lossy().to_string())
                    } else {
                        Line::from("???")
                    }
                }))
                .render(l_file_list[1], buf, &mut state.file_list);

            if state.popup.is_active() {
                PopupMenu::new()
                    .styles(ctx.g.theme.menu_style())
                    .block(Block::bordered())
                    .constraint(PopupConstraint::RightTop(Rect::new(
                        state.popup_pos.0,
                        state.popup_pos.1,
                        0,
                        0,
                    )))
                    .offset((-1, -1))
                    .boundary(state.file_list.area)
                    .item_parsed("_New")
                    .item_parsed("_Open")
                    .item_parsed("_Delete")
                    .render(Rect::default(), buf, &mut state.popup);
            }

            Ok(())
        }
    }

    impl HasFocus for FileListState {
        fn build(&self, builder: &mut FocusBuilder) {
            let tag = builder.start(self);
            builder.widget(&self.file_list);
            builder.end(tag);
        }

        fn focus(&self) -> FocusFlag {
            self.container.clone()
        }

        fn area(&self) -> Rect {
            self.file_list.area()
        }
    }

    impl AppState<GlobalState, MDEvent, Error> for FileListState {
        fn init(
            &mut self,
            ctx: &mut AppContext<'_, GlobalState, MDEvent, Error>,
        ) -> Result<(), Error> {
            self.load(&Path::new("."))?;
            Ok(())
        }

        fn event(
            &mut self,
            event: &MDEvent,
            ctx: &mut AppContext<'_, GlobalState, MDEvent, Error>,
        ) -> Result<Control<MDEvent>, Error> {
            match event {
                MDEvent::Event(event) => {
                    try_flow!(match self.popup.handle(event, Popup) {
                        MenuOutcome::Activated(0) => {
                            Control::Message(MDEvent::MenuNew)
                        }
                        MenuOutcome::Activated(1) => {
                            if let Some(pos) = self.file_list.row_at_clicked(self.popup_pos) {
                                Control::Message(MDEvent::Open(self.files[pos].clone()))
                            } else {
                                Control::Changed
                            }
                        }
                        MenuOutcome::Activated(2) => {
                            Control::Message(MDEvent::Message("buh".into()))
                        }
                        r => r.into(),
                    });

                    if self.file_list.is_focused() {
                        try_flow!(match event {
                            ct_event!(keycode press Enter) => {
                                if let Some(row) = self.file_list.selected() {
                                    Control::Message(MDEvent::SelectOrOpen(self.files[row].clone()))
                                } else {
                                    Control::Continue
                                }
                            }
                            ct_event!(key press '+') => {
                                if let Some(row) = self.file_list.selected() {
                                    Control::Message(MDEvent::SelectOrOpenSplit(
                                        self.files[row].clone(),
                                    ))
                                } else {
                                    Control::Continue
                                }
                            }
                            _ => Control::Continue,
                        });
                    }
                    try_flow!(match event {
                        ct_event!(mouse down Right for x,y)
                            if self.file_list.area.contains(Position::new(*x, *y)) =>
                        {
                            self.popup_pos = (*x, *y);
                            self.popup.set_active(true);
                            Control::Changed
                        }
                        ct_event!(mouse any for m)
                            if self.file_list.mouse.doubleclick(self.file_list.area, m) =>
                        {
                            if let Some(row) = self.file_list.row_at_clicked((m.column, m.row)) {
                                Control::Message(MDEvent::SelectOrOpen(self.files[row].clone()))
                            } else {
                                Control::Continue
                            }
                        }

                        _ => Control::Continue,
                    });
                    try_flow!(self.file_list.handle(event, Regular));

                    Ok(Control::Continue)
                }
                _ => Ok(Control::Continue),
            }
        }
    }

    impl FileListState {
        /// Current directory.
        pub fn current_dir(&self) -> &Path {
            &self.files_dir
        }

        /// Current file
        pub fn current_file(&self) -> Option<&Path> {
            if let Some(selected) = self.file_list.selected() {
                if selected < self.files.len() {
                    Some(&self.files[selected])
                } else {
                    None
                }
            } else {
                None
            }
        }

        /// Read directory.
        pub fn load(&mut self, dir: &Path) -> Result<(), Error> {
            self.files_dir = dir.into();
            self.files.clear();
            if let Ok(rd) = fs::read_dir(dir) {
                for f in rd {
                    let Ok(f) = f else {
                        continue;
                    };
                    let f = f.path();
                    if let Some(ext) = f.extension() {
                        if ext == "md" {
                            self.files.push(f);
                        }
                    }
                }
            }
            if self.files.len() > 0 {
                if let Some(sel) = self.file_list.selected() {
                    if sel > self.files.len() {
                        self.file_list.select(Some(self.files.len() - 1));
                    }
                } else {
                    self.file_list.select(Some(0));
                }
            } else {
                self.file_list.select(None);
            }
            Ok(())
        }

        /// Select this file.
        pub fn select(&mut self, file: &Path) -> Result<(), Error> {
            self.file_list.clear_selection();
            for (i, f) in self.files.iter().enumerate() {
                if file == f {
                    self.file_list.select(Some(i));
                    break;
                }
            }
            Ok(())
        }
    }
}

// -----------------------------------------------------------------------
// overall editor
// -----------------------------------------------------------------------
pub mod mdedit {
    use crate::file_list::{FileList, FileListState};
    use crate::mdfile::MDFileState;
    use crate::split_tab::{SplitTab, SplitTabState};
    use crate::{AppContext, GlobalState, MDEvent};
    use anyhow::Error;
    use rat_salsa::{AppState, AppWidget, Control, RenderContext};
    use rat_widget::event::{ct_event, try_flow, ConsumedEvent, HandleEvent, Regular};
    use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus};
    use rat_widget::splitter::{Split, SplitState, SplitType};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Rect};
    use ratatui::widgets::StatefulWidget;
    use std::path::{Path, PathBuf};

    #[derive(Debug, Default)]
    pub struct MDEdit;

    #[derive(Debug, Default)]
    pub struct MDEditState {
        pub window_cmd: bool,

        pub split_files: SplitState,
        pub file_list: FileListState,
        pub split_tab: SplitTabState,
    }

    impl AppWidget<GlobalState, MDEvent, Error> for MDEdit {
        type State = MDEditState;

        fn render(
            &self,
            area: Rect,
            buf: &mut Buffer,
            state: &mut Self::State,
            ctx: &mut RenderContext<'_, GlobalState>,
        ) -> Result<(), Error> {
            let (s0, s1) = Split::new()
                .styles(ctx.g.theme.split_style())
                .mark_offset(0)
                .constraints([Constraint::Length(15), Constraint::Fill(1)])
                .direction(Direction::Horizontal)
                .split_type(SplitType::FullQuadrantInside)
                .into_widgets();

            s0.render(area, buf, &mut state.split_files);

            FileList.render(
                state.split_files.widget_areas[0],
                buf,
                &mut state.file_list,
                ctx,
            )?;

            SplitTab.render(
                state.split_files.widget_areas[1],
                buf,
                &mut state.split_tab,
                ctx,
            )?;

            s1.render(area, buf, &mut state.split_files);

            if state.window_cmd {
                ctx.g.status.status(1, "^W");
            }

            Ok(())
        }
    }

    impl HasFocus for MDEditState {
        fn build(&self, builder: &mut FocusBuilder) {
            builder.widget(&self.file_list);
            builder.widget(&self.split_tab);
        }

        fn focus(&self) -> FocusFlag {
            unimplemented!("not in use, silent container")
        }

        fn area(&self) -> Rect {
            unimplemented!("not in use, silent container")
        }
    }

    impl AppState<GlobalState, MDEvent, Error> for MDEditState {
        fn init(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            self.file_list.load(&Path::new("."))?;
            Ok(())
        }

        fn event(
            &mut self,
            event: &MDEvent,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, MDEvent, Error>,
        ) -> Result<Control<MDEvent>, Error> {
            let mut r = match event {
                MDEvent::Event(event) => {
                    if self.window_cmd {
                        self.window_cmd = false;
                        try_flow!(match event {
                            ct_event!(key release CONTROL-'w') => {
                                self.window_cmd = true;
                                Control::Changed
                            }
                            ct_event!(keycode press Left) => {
                                self.split_tab.select_prev(ctx);
                                Control::Changed
                            }
                            ct_event!(keycode press Right) => {
                                self.split_tab.select_next(ctx);
                                Control::Changed
                            }
                            ct_event!(keycode press Tab) => {
                                ctx.focus().next();
                                Control::Changed
                            }
                            ct_event!(keycode press SHIFT-BackTab) => {
                                ctx.focus().prev();
                                Control::Changed
                            }
                            ct_event!(key press CONTROL-'c')
                            | ct_event!(key press 'c')
                            | ct_event!(key press 'x')
                            | ct_event!(key press CONTROL-'x') => {
                                Control::Message(MDEvent::Close)
                            }
                            ct_event!(key press CONTROL-'d')
                            | ct_event!(key press 'd')
                            | ct_event!(key press '+') => {
                                Control::Message(MDEvent::Split)
                            }
                            ct_event!(key press CONTROL-'t') | ct_event!(key press 't') => {
                                if let Some((pos, sel)) = self.split_tab.selected() {
                                    if sel.is_focused() {
                                        ctx.focus().focus(&self.split_tab.tabbed[pos.0]);
                                    } else {
                                        ctx.focus().focus(sel);
                                    }
                                }
                                Control::Changed
                            }
                            ct_event!(key press CONTROL-'s') | ct_event!(key press 's') => {
                                if let Some((pos, sel)) = self.split_tab.selected() {
                                    if sel.is_focused() {
                                        ctx.focus().focus(&self.split_tab.splitter);
                                    } else {
                                        ctx.focus().focus(sel);
                                    }
                                }
                                Control::Changed
                            }
                            _ => Control::Changed,
                        });
                    }

                    try_flow!(match event {
                        ct_event!(key press CONTROL-'n') => {
                            Control::Message(MDEvent::MenuNew)
                        }
                        ct_event!(key press CONTROL-'o') => {
                            Control::Message(MDEvent::MenuOpen)
                        }
                        ct_event!(key press CONTROL-'s') => {
                            Control::Message(MDEvent::Save)
                        }
                        ct_event!(keycode press F(5)) => {
                            self.jump_to_file(ctx)?
                        }
                        ct_event!(keycode press F(6)) => {
                            self.hide_files(ctx)?
                        }
                        ct_event!(key press CONTROL-'w') => {
                            self.window_cmd = true;
                            Control::Changed
                        }
                        ct_event!(focus_lost) => {
                            Control::Message(MDEvent::Save)
                        }
                        _ => Control::Continue,
                    });

                    try_flow!(self.split_files.handle(event, Regular));

                    Control::Continue
                }
                MDEvent::New(p) => {
                    self.new(p, ctx)?;
                    Control::Changed
                }
                MDEvent::SelectOrOpen(p) => {
                    self.select_or_open(p, ctx)?;
                    Control::Changed
                }
                MDEvent::SelectOrOpenSplit(p) => {
                    self.select_or_open_split(p, ctx)?;
                    Control::Changed
                }
                MDEvent::Open(p) => {
                    self.open(p, ctx)?;
                    Control::Changed
                }
                MDEvent::Save => {
                    self.save()?;
                    Control::Changed
                }
                MDEvent::SaveAs(p) => {
                    self.save_as(p)?;
                    Control::Changed
                }
                MDEvent::Close => {
                    if let Some(pos) = self.split_tab.selected_pos() {
                        self.split_tab.close((pos.0, pos.1), ctx)?;
                        Control::Changed
                    } else {
                        Control::Continue
                    }
                }
                MDEvent::CloseAt(idx_split, idx_tab) => {
                    self.split_tab.close((*idx_split, *idx_tab), ctx)?;
                    Control::Changed
                }
                MDEvent::SelectAt(idx_split, idx_tab) => {
                    self.split_tab.select((*idx_split, *idx_tab), ctx);
                    Control::Changed
                }

                MDEvent::Split => {
                    self.split(ctx)?;
                    Control::Changed
                }
                MDEvent::CfgShowCtrl => Control::Changed,
                MDEvent::CfgNewline => Control::Changed,

                MDEvent::JumpToFiles => self.jump_to_file(ctx)?,
                MDEvent::HideFiles => self.hide_files(ctx)?,

                MDEvent::SyncEdit => {
                    // synchronize instances
                    let (id_sel, sel_path, replay) =
                        if let Some((id_sel, sel)) = self.split_tab.selected_mut() {
                            (id_sel, sel.path.clone(), sel.edit.recent_replay_log())
                        } else {
                            ((0, 0), PathBuf::default(), Vec::default())
                        };
                    if !replay.is_empty() {
                        self.split_tab.replay(id_sel, &sel_path, &replay, ctx);
                    }
                    Control::Changed
                }
                _ => Control::Continue,
            };

            r = r.or_else_try(|| self.file_list.event(event, ctx))?;
            r = r.or_else_try(|| self.split_tab.event(event, ctx))?;

            Ok(r)
        }
    }

    impl MDEditState {
        // Open new file.
        pub fn new(&mut self, path: &Path, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            let pos = if let Some(pos) = self.split_tab.selected_pos() {
                (pos.0, pos.1 + 1)
            } else {
                (0, 0)
            };

            let new = MDFileState::new_file(&path, ctx);
            self.split_tab.open(pos, new, ctx);
            self.split_tab.select(pos, ctx);

            Ok(())
        }

        // Open path.
        pub fn open_split(&mut self, path: &Path, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            let pos = if let Some(pos) = self.split_tab.selected_pos() {
                if pos.0 + 1 >= self.split_tab.tabs.len() {
                    (pos.0 + 1, 0)
                } else {
                    if let Some(sel_tab) = self.split_tab.tabbed[pos.0 + 1].selected() {
                        (pos.0 + 1, sel_tab + 1)
                    } else {
                        (pos.0 + 1, 0)
                    }
                }
            } else {
                (0, 0)
            };

            self._open(pos, path, ctx)
        }

        // Open path.
        pub fn open(&mut self, path: &Path, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            let pos = if let Some(pos) = self.split_tab.selected_pos() {
                (pos.0, pos.1 + 1)
            } else {
                (0, 0)
            };

            self._open(pos, path, ctx)
        }

        fn _open(
            &mut self,
            pos: (usize, usize),
            path: &Path,
            ctx: &mut AppContext<'_>,
        ) -> Result<(), Error> {
            let new = if let Some((_, md)) = self.split_tab.for_path_mut(path) {
                // enable replay and clone the buffer
                if let Some(undo) = md.edit.undo_buffer_mut() {
                    undo.enable_replay_log(true);
                }
                md.clone()
            } else {
                MDFileState::open_file(path, ctx)?
            };
            self.split_tab.open(pos, new, ctx);
            self.split_tab.select(pos, ctx);

            if let Some(parent) = path.parent() {
                self.file_list.load(parent)?;
            }
            self.file_list.select(path)?;

            Ok(())
        }

        // Focus path or open file.
        pub fn select_or_open(
            &mut self,
            path: &Path,
            ctx: &mut AppContext<'_>,
        ) -> Result<(), Error> {
            if let Some((pos, md)) = self.split_tab.for_path(path) {
                self.split_tab.select(pos, ctx);
            } else {
                self.open(path, ctx)?;
            }
            Ok(())
        }

        // Focus path or open file.
        pub fn select_or_open_split(
            &mut self,
            path: &Path,
            ctx: &mut AppContext<'_>,
        ) -> Result<(), Error> {
            if let Some((pos, md)) = self.split_tab.for_path(path) {
                self.split_tab.select(pos, ctx);
            } else {
                self.open_split(path, ctx)?;
            }
            Ok(())
        }

        // Save all.
        pub fn save(&mut self) -> Result<(), Error> {
            self.split_tab.save()?;

            self.file_list.load(&self.file_list.files_dir.clone())?;
            if let Some((_, mdfile)) = self.split_tab.selected() {
                self.file_list.select(&mdfile.path)?;
            }

            Ok(())
        }

        // Save selected as.
        pub fn save_as(&mut self, path: &Path) -> Result<(), Error> {
            let mut path = path.to_path_buf();
            if path.extension().is_none() {
                path.set_extension("md");
            }

            if let Some((pos, t)) = self.split_tab.selected_mut() {
                t.save_as(&path)?;
            }
            Ok(())
        }

        // Hide files
        pub fn hide_files(&mut self, ctx: &mut AppContext<'_>) -> Result<Control<MDEvent>, Error> {
            if self.split_files.is_hidden(0) {
                self.split_files.show_split(0);
            } else {
                self.split_files.hide_split(0);
            }
            Ok(Control::Changed)
        }

        // Select Files
        pub fn jump_to_file(
            &mut self,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<MDEvent>, Error> {
            let mut r = Control::Continue;

            if self.split_files.is_hidden(0) {
                self.split_files.show_split(0);
                r = Control::Changed;
            }
            if !self.file_list.is_focused() {
                ctx.focus().focus(&self.file_list.file_list);
                r = Control::Changed;
            } else {
                if let Some((_, last_edit)) = self.split_tab.selected() {
                    ctx.focus().focus(last_edit);
                    r = Control::Changed;
                }
            }

            Ok(r)
        }

        // Split current buffer.
        pub fn split(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            let Some((pos, sel)) = self.split_tab.selected_mut() else {
                return Ok(());
            };
            // enable replay and clone the buffer
            if let Some(undo) = sel.edit.undo_buffer_mut() {
                undo.enable_replay_log(true);
            }
            let new = sel.clone();

            let new_pos = if pos.0 + 1 == self.split_tab.tabs.len() {
                (pos.0 + 1, 0)
            } else {
                (pos.0 + 1, self.split_tab.tabs[pos.0 + 1].len())
            };
            self.split_tab.open(new_pos, new, ctx);
            self.split_tab.select(pos, ctx);

            Ok(())
        }

        // Establish the currently focus split+tab as the active split.
        pub fn set_active_split(&mut self) -> bool {
            self.split_tab.set_active_split()
        }

        // Sync views.
        pub fn sync_views(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            let path = if let Some((_, md)) = self.split_tab.selected() {
                Some(md.path.clone())
            } else {
                None
            };
            if let Some(path) = path {
                if self.sync_files(&path)? == Control::Changed {
                    ctx.queue(Control::Changed);
                }
            }
            Ok(())
        }

        // Sync file-list with the given file.
        pub fn sync_files(&mut self, file: &Path) -> Result<Control<MDEvent>, Error> {
            if let Some(parent) = file.parent() {
                if self.file_list.current_dir() != parent {
                    self.file_list.load(parent)?;
                    self.file_list.select(file)?;
                    Ok(Control::Changed)
                } else if self.file_list.current_file() != Some(file) {
                    self.file_list.select(file)?;
                    Ok(Control::Changed)
                } else {
                    Ok(Control::Unchanged)
                }
            } else {
                Ok(Control::Unchanged)
            }
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

static HELP: &[u8] = include_bytes!("mdedit.md");
static CHEAT: &[u8] = include_bytes!("cheat.md");

fn event_str(event: &MDEvent) -> String {
    use crossterm::event::*;

    match event {
        MDEvent::TimeOut(timeout) => format!("{:?}", timeout).to_string(),
        MDEvent::Event(event) => match event {
            Event::FocusGained => "focus-gained".into(),
            Event::FocusLost => "focus-lost".into(),
            Event::Key(KeyEvent {
                code,
                modifiers,
                kind,
                state,
            }) => {
                format!("key {:?} {} {:?}", code, mods(modifiers), kind)
            }
            Event::Mouse(MouseEvent {
                kind,
                column,
                row,
                modifiers,
            }) => {
                format!("mouse {:?} {:?} {}", kind, (*column, *row), mods(modifiers))
            }
            Event::Paste(v) => {
                format!("paste {:?}", v)
            }
            Event::Resize(x, y) => {
                format!("resize {:?}", (*x, *y))
            }
        },
        MDEvent::Message(message) => message.to_string(),
        v => format!("{:?}", v).to_string(),
    }
}

fn mods(modifiers: &crossterm::event::KeyModifiers) -> String {
    use crossterm::event::*;

    let mut s = String::new();
    if modifiers.contains(KeyModifiers::CONTROL) {
        s.push_str("CTRL ");
    }
    if modifiers.contains(KeyModifiers::ALT) {
        s.push_str("ALT ");
    }
    if modifiers.contains(KeyModifiers::SHIFT) {
        s.push_str("SHIFT ");
    }
    s
}
