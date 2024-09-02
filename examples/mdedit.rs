#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unreachable_pub)]

use crate::facilities::{Facility, MDFileDialog, MDFileDialogState};
use crate::mdedit::{MDEdit, MDEditState};
use anyhow::Error;
#[allow(unused_imports)]
use log::debug;
use rat_salsa::event::try_flow;
use rat_salsa::timer::TimeOut;
use rat_salsa::{run_tui, AppState, AppWidget, Control, RenderContext, RunConfig};
use rat_theme::dark_theme::DarkTheme;
use rat_theme::scheme::IMPERIAL;
use rat_theme::{dark_themes, Scheme};
use rat_widget::event::{ct_event, ConsumedEvent, Dialog, HandleEvent, Popup, Regular};
use rat_widget::focus::{Focus, HasFocus, HasFocusFlag};
use rat_widget::layout::layout_middle;
use rat_widget::menubar::{MenuBarState, MenuStructure, Menubar};
use rat_widget::menuline::MenuOutcome;
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use rat_widget::popup_menu::{MenuItem, Placement, Separator};
use rat_widget::statusline::{StatusLine, StatusLineState};
use rat_widget::text::HasScreenCursor;
use rat_widget::util::menu_str;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Padding, StatefulWidget};
use std::fmt::Debug;
use std::fs;
use std::path::PathBuf;
use std::str::from_utf8;
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

    let app = MDRoot;
    let mut state = MDRootState::default();

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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MDAction {
    Message(String),
    MenuNew,
    MenuOpen,
    MenuSave,
    MenuSaveAs,

    FocusedFile(PathBuf),

    SyncEdit,

    New(PathBuf),
    Open(PathBuf),
    SelectOrOpen(PathBuf),
    SelectOrOpenSplit(PathBuf),
    SaveAs(PathBuf),
    Save,
    Split,
    Close,
    CloseAt(usize, usize),
    SelectAt(usize, usize),
}

// -----------------------------------------------------------------------

trait AppFocus<Global, Message, Error>
where
    Message: 'static + Send + Debug,
    Error: 'static + Send + Debug,
{
    fn focus_changed(
        &mut self,
        ctx: &mut rat_salsa::AppContext<'_, Global, Message, Error>,
    ) -> Result<(), Error>;
}

// -----------------------------------------------------------------------

#[derive(Debug)]
pub struct MDRoot;

#[derive(Debug, Default)]
pub struct MDRootState {
    pub app: MDAppState,
}

impl AppWidget<GlobalState, MDAction, Error> for MDRoot {
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

impl AppState<GlobalState, MDAction, Error> for MDRootState {
    fn init(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
        self.app.init(ctx)
    }

    fn timer(
        &mut self,
        event: &TimeOut,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<MDAction>, Error> {
        let t0 = SystemTime::now();

        let r = self.app.timer(event, ctx)?;

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

        try_flow!(match &event {
            ct_event!(resized) => Control::Changed,
            ct_event!(key press CONTROL-'q') => Control::Quit,
            _ => Control::Continue,
        });

        // keyboard + mouse focus
        ctx.focus = Some(self.app.focus());

        let r = self.app.crossterm(event, ctx)?;

        if ctx.focus().gained_focus().is_some() {
            self.app.focus_changed(ctx)?;
        }

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

        ctx.focus = Some(self.app.focus());

        let r = self.app.message(event, ctx)?;

        if ctx.focus().gained_focus().is_some() {
            self.app.focus_changed(ctx)?;
        }

        if r == Control::Changed {
            let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
            ctx.g.status.status(4, format!("A {:.0?}", el).to_string());
        }

        Ok(r)
    }

    fn error(&self, event: Error, ctx: &mut AppContext<'_>) -> Result<Control<MDAction>, Error> {
        self.app.error(event, ctx)
    }
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
            editor: MDEditState::default(),
            menu: MenuBarState::named("menu"),
        };
        s
    }
}

pub mod facilities {
    use crate::MDAction;
    use anyhow::Error;
    use crossterm::event::Event;
    use rat_salsa::event::try_flow;
    use rat_salsa::Control;
    use rat_widget::event::{Dialog, FileOutcome, HandleEvent};
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

#[derive(Debug)]
struct Menu {
    show_ctrl: bool,
    use_crlf: bool,
}

impl<'a> MenuStructure<'a> for Menu {
    fn menus(&'a self) -> Vec<(Line<'a>, Option<char>)> {
        vec![
            menu_str("_File"),
            menu_str("_Edit"),
            menu_str("_View"),
            menu_str("_Theme"),
            menu_str("_Quit"),
        ]
    }

    fn submenu(&'a self, n: usize) -> Vec<MenuItem<'a>> {
        match n {
            0 => {
                vec![
                    MenuItem::Item3("New..".into(), Some('n'), Line::from("Ctrl-N").italic()),
                    MenuItem::Item3("Open..".into(), Some('o'), Line::from("Ctrl-O").italic()),
                    MenuItem::Item3("Save..".into(), Some('s'), Line::from("Ctrl-S").italic()),
                    MenuItem::Item2(Line::from("Save as.."), Some('a')),
                ]
            }
            1 => {
                vec![
                    MenuItem::Item3("Format table".into(), None, Line::from("Alt-T").italic()),
                    MenuItem::Item3(
                        "Format equal".into(),
                        None,
                        Line::from("Alt-Shift-T").italic(),
                    ),
                ]
            }
            2 => {
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
                    MenuItem::Item3(
                        "Split view".into(),
                        Some('s'),
                        Line::from("Ctrl-W D").italic(),
                    ),
                ]
            }
            3 => dark_themes()
                .iter()
                .map(|v| MenuItem::Item(v.name().to_string().into()))
                .collect(),
            _ => Vec::default(),
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

impl AppFocus<GlobalState, MDAction, Error> for MDAppState {
    fn focus_changed(
        &mut self,
        ctx: &mut rat_salsa::AppContext<'_, GlobalState, MDAction, Error>,
    ) -> Result<(), Error> {
        self.editor.focus_changed(ctx)
    }
}

impl AppState<GlobalState, MDAction, Error> for MDAppState {
    fn init(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
        self.menu.bar.select(Some(0));
        self.menu.focus().set(true);
        self.editor.init(ctx)?;
        Ok(())
    }

    fn timer(
        &mut self,
        event: &TimeOut,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<MDAction>, Error> {
        let r = self.editor.timer(event, ctx)?;
        Ok(r)
    }

    fn crossterm(
        &mut self,
        event: &crossterm::event::Event,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<MDAction>, Error> {
        try_flow!(ctx.g.error_dlg.handle(event, Dialog));
        try_flow!(ctx.g.file_dlg.handle(event)?);

        let f = Control::from(ctx.focus_mut().handle(event, Regular));

        f.and_try(|| {
            // regular global
            try_flow!(match &event {
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
                    ctx.g.error_dlg.append(from_utf8(HELP)?);
                    Control::Changed
                }
                _ => Control::Continue,
            });

            try_flow!(match self.menu.handle(event, Popup) {
                MenuOutcome::MenuActivated(0, 0) => Control::Message(MDAction::MenuNew),
                MenuOutcome::MenuActivated(0, 1) => Control::Message(MDAction::MenuOpen),
                MenuOutcome::MenuActivated(0, 2) => Control::Message(MDAction::MenuSave),
                MenuOutcome::MenuActivated(0, 3) => Control::Message(MDAction::MenuSaveAs),
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
                    Control::Changed
                }
                MenuOutcome::MenuActivated(2, 1) => {
                    if ctx.g.cfg.new_line == "\r\n" {
                        ctx.g.cfg.new_line = "\n".into();
                    } else {
                        ctx.g.cfg.new_line = "\r\n".into();
                    }
                    Control::Changed
                }
                MenuOutcome::MenuActivated(2, 2) => {
                    Control::Message(MDAction::Split)
                }
                MenuOutcome::MenuSelected(3, n) => {
                    ctx.g.theme = dark_themes()[n].clone();
                    Control::Changed
                }
                r => r.into(),
            });

            try_flow!(self.editor.crossterm(event, ctx)?);
            try_flow!(match self.menu.handle(event, Regular) {
                MenuOutcome::Activated(4) => Control::Quit,
                r => r.into(),
            });

            Ok(Control::Continue)
        })
    }

    fn message(
        &mut self,
        event: &mut MDAction,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<MDAction>, Error> {
        try_flow!(match event {
            MDAction::Message(s) => {
                ctx.g.status.status(0, &*s);
                Control::Changed
            }
            MDAction::MenuNew => {
                ctx.g.file_dlg.engage(
                    |w| {
                        w.save_dialog(".", "")?;
                        Ok(Control::Changed)
                    },
                    |p| Ok(Control::Message(MDAction::New(p))),
                )?
            }
            MDAction::MenuOpen => {
                ctx.g.file_dlg.engage(
                    |w| {
                        w.open_dialog(".")?;
                        Ok(Control::Changed)
                    },
                    |p| Ok(Control::Message(MDAction::Open(p))),
                )?
            }
            MDAction::MenuSave => {
                Control::Message(MDAction::Save)
            }
            MDAction::MenuSaveAs => {
                ctx.g.file_dlg.engage(
                    |w| {
                        w.save_dialog(".", "")?;
                        Ok(Control::Changed)
                    },
                    |p| Ok(Control::Message(MDAction::SaveAs(p))),
                )?
            }
            _ => Control::Continue,
        });

        try_flow!(self.editor.message(event, ctx)?);

        Ok(Control::Continue)
    }

    fn error(&self, event: Error, ctx: &mut AppContext<'_>) -> Result<Control<MDAction>, Error> {
        ctx.g.error_dlg.title("Error occured");
        ctx.g.error_dlg.append(format!("{:?}", &*event).as_str());
        Ok(Control::Changed)
    }
}

// Extended text-editing for markdown.
pub mod markdown {
    use anyhow::{anyhow, Error};
    use log::debug;
    use pulldown_cmark::{Event, Options, Parser, Tag};
    use rat_salsa::event::ct_event;
    use rat_widget::event::{flow, HandleEvent, Regular, TextOutcome};
    use rat_widget::text::{upos_type, TextPosition, TextRange};
    use rat_widget::textarea::TextAreaState;
    use std::cmp::max;
    use std::ops::Range;
    use unicode_segmentation::UnicodeSegmentation;

    // Markdown styles
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum MDStyle {
        Heading = 0,
        BlockQuote,
        CodeBlock,
        FootnoteDefinition,
        FootnoteReference,
        ItemTag,
        Item,
        Emphasis,
        Strong,
        Strikethrough,
        Link, // 10
        Image,
        MetadataBlock,
        CodeInline,
        MathInline,
        MathDisplay,
        Rule,
        TaskListMarker,
        Html,
        Table,
        TableHead, // 20
        TableRow,
        TableCell,
        Paragraph,
        List,
    }

    impl From<MDStyle> for usize {
        fn from(value: MDStyle) -> Self {
            value as usize
        }
    }

    impl TryFrom<usize> for MDStyle {
        type Error = Error;

        fn try_from(value: usize) -> Result<Self, Self::Error> {
            use MDStyle::*;
            Ok(match value {
                0 => Heading,
                1 => BlockQuote,
                2 => CodeBlock,
                3 => FootnoteDefinition,
                4 => FootnoteReference,
                5 => ItemTag,
                6 => Item,
                7 => Emphasis,
                8 => Strong,
                9 => Strikethrough,
                10 => Link,
                11 => Image,
                12 => MetadataBlock,
                13 => CodeInline,
                14 => MathInline,
                15 => MathDisplay,
                16 => Rule,
                17 => TaskListMarker,
                18 => Html,
                19 => Table,
                20 => TableHead,
                21 => TableRow,
                22 => TableCell,
                23 => Paragraph,
                24 => List,
                _ => return Err(anyhow!("invalid style {}", value)),
            })
        }
    }

    pub fn md_format(state: &mut TextAreaState, equal_width: bool) -> TextOutcome {
        if let Some((table_byte, table_range)) = md_table(state) {
            let cursor = state.cursor();

            let (table, new_cursor) = reformat_md_table(
                state.str_slice_byte(table_byte).as_ref(),
                table_range,
                cursor,
                equal_width,
                state.newline(),
            );

            state.begin_undo_seq();
            state.delete_range(table_range);
            state
                .value
                .insert_str(table_range.start, &table)
                .expect("fine");
            state.set_cursor(new_cursor, false);
            state.end_undo_seq();
            TextOutcome::TextChanged
        } else if let Some((
            item_byte, //
            item_range,
            para_byte,
            para_range,
        )) = md_item_paragraph(state)
        {
            let item_str = state.str_slice_byte(item_byte.clone());
            let item = parse_md_item(item_byte.start, item_str.as_ref());
            let item_pos = state.byte_pos(item.mark_bytes.start);
            let item_text_pos = state.byte_pos(item.text_bytes.start);

            let text_indent0 = if item_pos.y == para_range.start.y {
                "".to_string()
            } else {
                " ".repeat((item_text_pos.x - para_range.start.x) as usize)
            };
            let text_indent = " ".repeat(item_text_pos.x as usize);

            let para_text = state.str_slice_byte(para_byte);
            let (para_text, _) = textwrap::unfill(para_text.as_ref());
            let wrap = textwrap::fill(
                para_text.as_ref(),
                textwrap::Options::new(65)
                    .initial_indent(&text_indent0)
                    .subsequent_indent(&text_indent),
            );

            state.begin_undo_seq();
            state.delete_range(para_range);
            state
                .value
                .insert_str(para_range.start, &wrap)
                .expect("fine");
            state.set_cursor(para_range.start, false);
            state.end_undo_seq();
            TextOutcome::TextChanged
        } else if let Some((item_byte, item_range)) = md_item(state) {
            let item_str = state.str_slice_byte(item_byte.clone());
            let item = parse_md_item(item_byte.start, item_str.as_ref());
            let item_text_range = state.byte_range(item.text_bytes.clone());
            let text_indent = " ".repeat(item_text_range.start.x as usize);

            let item_text = state.str_slice_byte(item.text_bytes);
            let (item_text, _) = textwrap::unfill(item_text.as_ref());
            let item_wrap = textwrap::fill(
                item_text.as_ref(),
                textwrap::Options::new(65).subsequent_indent(&text_indent),
            );

            state.begin_undo_seq();
            state.delete_range(item_text_range);
            state
                .value
                .insert_str(item_text_range.start, &item_wrap)
                .expect("fine");
            state.set_cursor(item_text_range.start, false);
            state.end_undo_seq();
            TextOutcome::TextChanged
        } else if let Some((para_byte, para_range)) = md_paragraph(state) {
            let cursor = state.cursor();

            let txt = state.str_slice_byte(para_byte);
            let wrap = textwrap::refill(txt.as_ref(), 65);

            state.begin_undo_seq();
            state.delete_range(para_range);
            state
                .value
                .insert_str(para_range.start, &wrap)
                .expect("fine");
            state.set_cursor(para_range.start, false);
            state.end_undo_seq();
            TextOutcome::TextChanged
        } else {
            TextOutcome::Continue
        }
    }

    pub fn md_line_break(state: &mut TextAreaState) -> TextOutcome {
        let cursor = state.cursor();
        if is_md_table(state) {
            let line = state.line_at(cursor.y);
            if cursor.x == state.line_width(cursor.y) {
                let (x, row) = empty_md_row(line.as_ref(), state.newline());
                state.insert_str(row);
                state.set_cursor((x, cursor.y + 1), false);
                TextOutcome::TextChanged
            } else {
                let (x, row) = split_md_row(line.as_ref(), cursor.x, state.newline());
                state.begin_undo_seq();
                state.delete_range(TextRange::new((0, cursor.y), (0, cursor.y + 1)));
                state.insert_str(row);
                state.set_cursor((x, cursor.y + 1), false);
                state.end_undo_seq();
                TextOutcome::TextChanged
            }
        } else {
            let cursor = state.cursor();
            if cursor.x == state.line_width(cursor.y) {
                let (maybe_table, maybe_header) = is_md_maybe_table(state);
                if maybe_header {
                    let line = state.line_at(cursor.y);
                    let (x, row) = empty_md_row(line.as_ref(), state.newline());
                    state.insert_str(row);
                    state.set_cursor((x, cursor.y + 1), false);
                    TextOutcome::TextChanged
                } else if maybe_table {
                    let line = state.line_at(cursor.y);
                    let (x, row) = create_md_title(line.as_ref(), state.newline());
                    state.insert_str(row);
                    state.set_cursor((x, cursor.y + 1), false);
                    TextOutcome::TextChanged
                } else {
                    TextOutcome::Continue
                }
            } else {
                TextOutcome::Continue
            }
        }
    }

    pub fn md_tab(state: &mut TextAreaState) -> TextOutcome {
        if is_md_table(state) {
            let cursor = state.cursor();
            let row = state.line_at(cursor.y);
            let x = next_tab_md_row(row.as_ref(), cursor.x);
            state.set_cursor((x, cursor.y), false);
            state.set_move_col(Some(x));
            TextOutcome::TextChanged
        } else if is_md_item(state) {
            let cursor = state.cursor();

            let (item_byte, item_range) = md_item(state).expect("item");
            let indent_x = if item_range.start.y < cursor.y {
                let item_str = state.str_slice_byte(item_byte.clone());
                let item = parse_md_item(item_byte.start, item_str.as_ref());
                state.byte_pos(item.text_bytes.start).x
            } else if let Some((prev_byte, prev_range)) = md_prev_item(state) {
                let prev_str = state.str_slice_byte(prev_byte.clone());
                let prev_item = parse_md_item(prev_byte.start, prev_str.as_ref());
                state.byte_pos(prev_item.text_bytes.start).x
            } else {
                0
            };

            if cursor.x < indent_x {
                state
                    .value
                    .insert_str(cursor, &(" ".repeat((indent_x - cursor.x) as usize)))
                    .expect("fine");
                TextOutcome::TextChanged
            } else {
                TextOutcome::Continue
            }
        } else {
            TextOutcome::Continue
        }
    }

    pub fn md_backtab(state: &mut TextAreaState) -> TextOutcome {
        if is_md_table(state) {
            let cursor = state.cursor();

            let row_str = state.line_at(cursor.y);
            let x = prev_tab_md_row(row_str.as_ref(), cursor.x);

            state.set_cursor((x, cursor.y), false);
            state.set_move_col(Some(x));
            TextOutcome::TextChanged
        } else {
            TextOutcome::Continue
        }
    }

    fn md_xxx(state: &TextAreaState) -> TextOutcome {
        let cursor = state.cursor();
        let cursor_byte = state.byte_at(cursor).start;

        let mut sty = Vec::new();
        state.styles_at(cursor_byte, &mut sty);
        for (r, s) in sty {
            debug!("style {:?}: {:?}", cursor, MDStyle::try_from(s));
        }

        if let Some(list_byte) = state.style_match(cursor_byte, MDStyle::List as usize) {
            let mut sty = Vec::new();
            state.styles_in(list_byte, &mut sty);

            for (r, s) in sty {
                if s == MDStyle::Item as usize {
                    let txt = state.str_slice_byte(r.clone());
                    let item = parse_md_item(r.start, txt.as_ref());
                    debug!("{:#?}", item);
                }
            }
        }
        TextOutcome::Unchanged
    }

    // qualifier for markdown-editing.
    #[derive(Debug)]
    pub struct MarkDown;

    impl HandleEvent<crossterm::event::Event, MarkDown, TextOutcome> for TextAreaState {
        fn handle(&mut self, event: &crossterm::event::Event, qualifier: MarkDown) -> TextOutcome {
            flow!(match event {
                ct_event!(key press ALT-'p') => md_xxx(self),

                ct_event!(key press ALT-'f') => md_format(self, false),
                ct_event!(key press ALT_SHIFT-'F') => md_format(self, true),
                ct_event!(keycode press Enter) => md_line_break(self),
                ct_event!(keycode press Tab) => md_tab(self),
                ct_event!(keycode press SHIFT-BackTab) => md_backtab(self),
                _ => TextOutcome::Continue,
            });

            self.handle(event, Regular)
        }
    }

    pub fn parse_md_styles(state: &TextAreaState) -> Vec<(Range<usize>, usize)> {
        let mut styles = Vec::new();

        let txt = state.text();

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
                    styles.push((r, MDStyle::Heading as usize));
                }
                Event::Start(Tag::BlockQuote(v)) => {
                    styles.push((r, MDStyle::BlockQuote as usize));
                }
                Event::Start(Tag::CodeBlock(v)) => {
                    styles.push((r, MDStyle::CodeBlock as usize));
                }
                Event::Start(Tag::FootnoteDefinition(v)) => {
                    styles.push((r, MDStyle::FootnoteDefinition as usize));
                }
                Event::Start(Tag::Item) => {
                    // only color the marker
                    let text = state.str_slice_byte(r.clone());
                    let item = parse_md_item(r.start, text.as_ref());
                    styles.push((
                        item.mark_bytes.start..item.mark_bytes.end,
                        MDStyle::ItemTag as usize,
                    ));
                    styles.push((r, MDStyle::Item as usize));
                }
                Event::Start(Tag::Emphasis) => {
                    styles.push((r, MDStyle::Emphasis as usize));
                }
                Event::Start(Tag::Strong) => {
                    styles.push((r, MDStyle::Strong as usize));
                }
                Event::Start(Tag::Strikethrough) => {
                    styles.push((r, MDStyle::Strikethrough as usize));
                }
                Event::Start(Tag::Link { .. }) => {
                    styles.push((r, MDStyle::Link as usize));
                }
                Event::Start(Tag::Image { .. }) => {
                    styles.push((r, MDStyle::Image as usize));
                }
                Event::Start(Tag::MetadataBlock { .. }) => {
                    styles.push((r, MDStyle::MetadataBlock as usize));
                }
                Event::Start(Tag::Paragraph) => {
                    styles.push((r, MDStyle::Paragraph as usize));
                }
                Event::Start(Tag::HtmlBlock) => {
                    styles.push((r, MDStyle::Html as usize));
                }
                Event::Start(Tag::List(_)) => {
                    styles.push((r, MDStyle::List as usize));
                }
                Event::Start(Tag::Table(_)) => {
                    styles.push((r, MDStyle::Table as usize));
                }
                Event::Start(Tag::TableHead) => {
                    styles.push((r, MDStyle::TableHead as usize));
                }
                Event::Start(Tag::TableRow) => {
                    styles.push((r, MDStyle::TableRow as usize));
                }
                Event::Start(Tag::TableCell) => {
                    styles.push((r, MDStyle::TableCell as usize));
                }
                Event::Code(v) => {
                    styles.push((r, MDStyle::CodeInline as usize));
                }
                Event::InlineMath(v) => {
                    styles.push((r, MDStyle::MathInline as usize));
                }
                Event::DisplayMath(v) => {
                    styles.push((r, MDStyle::MathDisplay as usize));
                }
                Event::FootnoteReference(v) => {
                    styles.push((r, MDStyle::FootnoteReference as usize));
                }
                Event::Rule => {
                    styles.push((r, MDStyle::Rule as usize));
                }
                Event::TaskListMarker(v) => {
                    styles.push((r, MDStyle::TaskListMarker as usize));
                }
                Event::Html(v) | Event::InlineHtml(v) => {
                    styles.push((r, MDStyle::Html as usize));
                }

                _ => {}
            }
        }

        styles
    }

    /// Length as grapheme count, excluding line breaks.
    fn str_line_len(s: &str) -> upos_type {
        let it = s.graphemes(true);
        it.filter(|c| *c != "\n" && *c != "\r\n").count() as upos_type
    }

    fn is_md_maybe_table(state: &TextAreaState) -> (bool, bool) {
        let mut gr = state.line_graphemes(state.cursor().y);
        let (maybe_table, maybe_header) = if let Some(first) = gr.next() {
            if first == "|" {
                if let Some(second) = gr.next() {
                    if second == "-" {
                        (true, true)
                    } else {
                        (true, false)
                    }
                } else {
                    (true, false)
                }
            } else {
                (false, false)
            }
        } else {
            (false, false)
        };
        (maybe_table, maybe_header)
    }

    fn is_md_table(state: &TextAreaState) -> bool {
        let cursor = state.cursor();
        let cursor_byte = state.byte_at(cursor).start;
        state
            .style_match(cursor_byte, MDStyle::Table as usize)
            .is_some()
    }

    fn md_table(state: &TextAreaState) -> Option<(Range<usize>, TextRange)> {
        let cursor = state.cursor();
        let cursor_byte = state.byte_at(cursor).start;

        let row_byte = state.style_match(cursor_byte, MDStyle::Table as usize);

        if let Some(row_byte) = row_byte {
            Some((row_byte.clone(), state.byte_range(row_byte)))
        } else {
            None
        }
    }

    fn is_md_paragraph(state: &TextAreaState) -> bool {
        let cursor = state.cursor();
        let cursor_byte = state.byte_at(cursor).start;
        state
            .style_match(cursor_byte, MDStyle::Paragraph as usize)
            .is_some()
    }

    fn md_paragraph(state: &TextAreaState) -> Option<(Range<usize>, TextRange)> {
        let cursor = state.cursor();
        let cursor_byte = state.byte_at(cursor).start;

        let row_byte = state.style_match(cursor_byte, MDStyle::Paragraph as usize);

        if let Some(row_byte) = row_byte {
            Some((row_byte.clone(), state.byte_range(row_byte)))
        } else {
            None
        }
    }

    fn md_item_paragraph(
        state: &TextAreaState,
    ) -> Option<(Range<usize>, TextRange, Range<usize>, TextRange)> {
        let cursor = state.cursor();
        let cursor_byte = state.byte_at(cursor).start;

        let mut sty = Vec::new();
        state.styles_at(cursor_byte, &mut sty);

        let mut r_list = None;
        let mut r_para = None;
        for (r, s) in sty {
            if s == MDStyle::List as usize {
                r_list = Some(r.clone());
            }
            if s == MDStyle::Paragraph as usize {
                r_para = Some(r.clone());
            }
        }

        if let Some(r_list) = r_list {
            if let Some(r_para) = r_para {
                Some((
                    r_list.clone(),
                    state.byte_range(r_list),
                    r_para.clone(),
                    state.byte_range(r_para),
                ))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn is_md_item(state: &TextAreaState) -> bool {
        let cursor = state.cursor();
        let cursor_byte = state.byte_at(cursor).start;
        state
            .style_match(cursor_byte, MDStyle::Item as usize)
            .is_some()
    }

    fn md_prev_item(state: &TextAreaState) -> Option<(Range<usize>, TextRange)> {
        let cursor = state.cursor();
        let cursor_byte = state.byte_at(cursor).start;

        let item_byte = state.style_match(cursor_byte, MDStyle::Item as usize);
        let list_byte = state.style_match(cursor_byte, MDStyle::List as usize);

        if let Some(list_byte) = list_byte {
            if let Some(item_byte) = item_byte {
                let mut sty = Vec::new();
                state.styles_in(list_byte.start..item_byte.start, &mut sty);

                let prev = sty.iter().filter(|v| v.1 == MDStyle::Item as usize).last();

                if let Some((prev_bytes, _)) = prev {
                    let prev = state.byte_range(prev_bytes.clone());
                    Some((prev_bytes.clone(), prev))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn md_item(state: &TextAreaState) -> Option<(Range<usize>, TextRange)> {
        let cursor = state.cursor();
        let cursor_byte = state.byte_at(cursor).start;

        let item_byte = state.style_match(cursor_byte, MDStyle::Item as usize);

        if let Some(item_byte) = item_byte {
            Some((item_byte.clone(), state.byte_range(item_byte)))
        } else {
            None
        }
    }

    fn prev_tab_md_row(txt: &str, pos: upos_type) -> upos_type {
        let row = parse_md_row(txt, pos);
        if row.cursor_idx > 0 {
            row.row[row.cursor_idx - 1].graphemes.start
        } else {
            pos
        }
    }

    fn next_tab_md_row(txt: &str, pos: upos_type) -> upos_type {
        let row = parse_md_row(txt, pos);
        if row.cursor_idx + 1 < row.row.len() {
            row.row[row.cursor_idx + 1].graphemes.start
        } else {
            pos
        }
    }

    // reformat
    fn reformat_md_table(
        txt: &str,
        range: TextRange,
        cursor: TextPosition,
        eq_width: bool,
        new_line: &str,
    ) -> (String, TextPosition) {
        use std::fmt::Write;

        let mut table = Vec::new();
        for (row_idx, row) in txt.lines().enumerate() {
            if !row.is_empty() {
                if range.start.y + row_idx as upos_type == cursor.y {
                    table.push(parse_md_row(row, cursor.x));
                } else {
                    table.push(parse_md_row(row, 0));
                }
            }
        }
        let mut width = Vec::new();
        // only use header widths
        if let Some(row) = table.first() {
            for (idx, cell) in row.row.iter().enumerate() {
                if idx >= width.len() {
                    width.push(str_line_len(cell.txt));
                } else {
                    let len = str_line_len(cell.txt);
                    width[idx] = max(width[idx], len)
                }
            }
        }
        if eq_width {
            let max_w = width.iter().max().copied().unwrap_or_default();
            let width_end = width.len() - 1;
            for w in &mut width[1..width_end] {
                *w = max_w;
            }
        }

        let mut buf = String::new();
        let mut buf_col = 0;
        for (row_idx, row) in table.iter().enumerate() {
            let mut col_pos = 0;
            for idx in 1..width.len() {
                // relocate cursor
                if range.start.y + row_idx as upos_type == cursor.y {
                    if idx == row.cursor_idx {
                        buf_col = col_pos + 1 + row.cursor_offset;
                    }
                }

                let len = width[idx];
                col_pos += len + 1;
                buf.push('|');
                if let Some(cell) = row.row.get(idx) {
                    if cell.txt.starts_with('-') {
                        _ = write!(buf, "{}", "-".repeat(len as usize));
                    } else {
                        _ = write!(
                            buf,
                            " {:1$} ",
                            cell.txt.trim(),
                            len.saturating_sub(2) as usize
                        );
                    }
                } else {
                    _ = write!(buf, "{}", " ".repeat(len as usize));
                }
            }
            buf.push_str(new_line);
        }

        (buf, TextPosition::new(buf_col, cursor.y))
    }

    // create underlines under the header
    fn create_md_title(txt: &str, newline: &str) -> (upos_type, String) {
        let row = parse_md_row(txt, 0);

        let mut new_row = String::new();
        new_row.push_str(newline);
        new_row.push_str(row.row[0].txt);
        new_row.push('|');
        for idx in 1..row.row.len() - 1 {
            for g in row.row[idx].txt.graphemes(true) {
                new_row.push('-');
            }
            new_row.push('|');
        }

        let len = str_line_len(&new_row);

        (len, new_row)
    }

    // add a line break
    fn split_md_row(txt: &str, cursor: upos_type, newline: &str) -> (upos_type, String) {
        let row = parse_md_row(txt, 0);

        let mut tmp0 = String::new();
        let mut tmp1 = String::new();
        let mut tmp_pos = 0;
        tmp0.push('|');
        tmp1.push('|');
        for row in &row.row[1..row.row.len() - 1] {
            if row.graphemes.contains(&cursor) {
                tmp_pos = row.graphemes.start + 1;

                let mut pos = row.graphemes.start;
                if cursor > row.graphemes.start {
                    tmp1.push(' ');
                }
                for g in row.txt.graphemes(true) {
                    if pos < cursor {
                        tmp0.push_str(g);
                    } else {
                        tmp1.push_str(g);
                    }
                    pos += 1;
                }
                pos = row.graphemes.start;
                for g in row.txt.graphemes(true) {
                    if pos < cursor {
                        // omit one blank
                        if pos != row.graphemes.start || cursor == row.graphemes.start {
                            tmp1.push(' ');
                        }
                    } else {
                        tmp0.push(' ');
                    }
                    pos += 1;
                }
            } else if row.graphemes.start < cursor {
                tmp0.push_str(row.txt);
                tmp1.push_str(" ".repeat(row.graphemes.len()).as_str());
            } else if row.graphemes.start >= cursor {
                tmp0.push_str(" ".repeat(row.graphemes.len()).as_str());
                tmp1.push_str(row.txt);
            }

            tmp0.push('|');
            tmp1.push('|');
        }
        tmp0.push_str(newline);
        tmp0.push_str(tmp1.as_str());
        tmp0.push_str(newline);

        (tmp_pos, tmp0)
    }

    // duplicate as empty row
    fn empty_md_row(txt: &str, newline: &str) -> (upos_type, String) {
        let row = parse_md_row(txt, 0);
        let mut new_row = String::new();
        new_row.push_str(newline);
        new_row.push('|');
        for idx in 1..row.row.len() - 1 {
            for g in row.row[idx].txt.graphemes(true) {
                new_row.push(' ');
            }
            new_row.push('|');
        }

        let x = if row.row.len() > 1 && row.row[1].txt.len() > 0 {
            str_line_len(row.row[0].txt) + 1 + 1
        } else {
            str_line_len(row.row[0].txt) + 1
        };

        (x, new_row)
    }

    // parse a single list item into marker and text.
    #[derive(Debug)]
    struct MDItem<'a> {
        mark_bytes: Range<usize>,
        mark: &'a str,
        mark_suffix: &'a str,
        mark_nr: Option<usize>,
        text_bytes: Range<usize>,
        text: &'a str,
    }

    fn parse_md_item(start: usize, txt: &str) -> MDItem<'_> {
        let mut mark_byte_start = 0;
        let mut mark_byte_end = 0;
        let mut mark_bullet = None;
        let mut mark_ordered = None;
        let mut mark_suffix = None;
        let mut mark_nr = None;
        let mut text_byte_start = 0;
        let mut text_byte_end = 0;
        let mut text_str = None;

        #[derive(Debug, PartialEq)]
        enum It {
            Leading,
            BulletMark,
            OrderedMark,
            OrderedSuffix,
            TextLeading,
        }

        let mut state = It::Leading;
        for (idx, c) in txt.bytes().enumerate() {
            if state == It::BulletMark {
                state = It::TextLeading;
            }
            if state == It::Leading {
                if c == b'+' || c == b'-' || c == b'*' {
                    mark_byte_start = idx;
                    mark_byte_end = idx + 1;
                    mark_bullet = Some(&txt[idx..idx + 1]);
                    state = It::BulletMark;
                } else if c.is_ascii_digit() {
                    mark_byte_start = idx;
                    state = It::OrderedMark;
                } else if c == b' ' || c == b'\t' {
                    // ok
                } else {
                    // broken??
                    state = It::TextLeading;
                }
            }
            if state == It::OrderedSuffix {
                state = It::TextLeading
            }
            if state == It::OrderedMark {
                if c.is_ascii_digit() {
                    // ok
                } else if c == b'.' || c == b')' {
                    mark_byte_end = idx + 1;
                    mark_ordered = Some(&txt[mark_byte_start..idx]);
                    mark_nr = Some(txt[mark_byte_start..idx].parse::<usize>().expect("nr"));
                    mark_suffix = Some(&txt[idx..idx + 1]);
                    state = It::OrderedSuffix;
                } else {
                    state = It::TextLeading;
                }
            }
            if state == It::TextLeading {
                if c == b' ' || c == b'\t' {
                    // ok
                } else {
                    text_byte_start = idx;
                    text_byte_end = txt.len();
                    text_str = Some(&txt[idx..]);
                    break;
                }
            }
        }

        MDItem {
            mark_bytes: start + mark_byte_start..start + mark_byte_end,
            mark: mark_bullet.unwrap_or_else(|| mark_ordered.unwrap_or("")),
            mark_suffix: mark_suffix.unwrap_or(""),
            mark_nr,
            text_bytes: start + text_byte_start..start + text_byte_end,
            text: text_str.unwrap_or(""),
        }
    }

    #[derive(Debug)]
    struct MDCell<'a> {
        txt: &'a str,
        graphemes: Range<upos_type>,
        bytes: Range<usize>,
    }

    #[derive(Debug)]
    struct MDRow<'a> {
        row: Vec<MDCell<'a>>,
        cursor_idx: usize,
        cursor_offset: upos_type,
    }

    // split single row. translate x-position to cell+cell_offset.
    // __info__: returns the string before the first | and the string after the last | too!!
    fn parse_md_row(txt: &str, x: upos_type) -> MDRow<'_> {
        let mut tmp = MDRow {
            row: Default::default(),
            cursor_idx: 0,
            cursor_offset: 0,
        };

        let mut byte_start = 0;
        let mut grapheme_start = 0;
        let mut grapheme_last = 0;
        let mut esc = false;
        let mut cell_offset = 0;
        for (idx, (byte_idx, c)) in txt.grapheme_indices(true).enumerate() {
            if idx == x as usize {
                tmp.cursor_idx = tmp.row.len();
                tmp.cursor_offset = cell_offset;
            }

            if c == "\\" {
                cell_offset += 1;
                esc = true;
            } else if c == "|" && !esc {
                cell_offset = 0;
                tmp.row.push(MDCell {
                    txt: &txt[byte_start..byte_idx],
                    graphemes: grapheme_start..idx as upos_type,
                    bytes: byte_start..byte_idx,
                });
                byte_start = byte_idx + 1;
                grapheme_start = idx as upos_type + 1;
            } else {
                cell_offset += 1;
                esc = false;
            }

            grapheme_last = idx as upos_type;
        }

        tmp.row.push(MDCell {
            txt: &txt[byte_start..txt.len()],
            graphemes: grapheme_start..grapheme_last,
            bytes: byte_start..txt.len(),
        });

        tmp
    }
}

// Editor for a single file.
pub mod mdfile {
    use crate::markdown::{md_format, parse_md_styles, MarkDown};
    use crate::{AppContext, GlobalState, MDAction};
    use anyhow::Error;
    use log::{debug, warn};
    use rat_salsa::timer::{TimeOut, TimerDef, TimerHandle};
    use rat_salsa::{AppState, AppWidget, Control, RenderContext};
    use rat_widget::event::{try_flow, HandleEvent, TextOutcome};
    use rat_widget::focus::{FocusFlag, HasFocusFlag, Navigation};
    use rat_widget::line_number::{LineNumberState, LineNumbers};
    use rat_widget::scrolled::Scroll;
    use rat_widget::text::clipboard::{Clipboard, ClipboardError};
    use rat_widget::text::{upos_type, HasScreenCursor};
    use rat_widget::textarea::{TextArea, TextAreaState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::prelude::{Modifier, StatefulWidget, Style};
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

    impl AppWidget<GlobalState, MDAction, Error> for MDFile {
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
                        .border_type(BorderType::Plain)
                        .borders(Borders::RIGHT),
                )
                .vscroll(
                    Scroll::new()
                        .start_margin(self.start_margin)
                        .scroll_by(1)
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

    fn text_style(ctx: &mut RenderContext<'_, GlobalState>) -> [Style; 25] {
        [
            Style::default().fg(ctx.g.scheme().yellow[2]).underlined(), // Heading,
            Style::default().fg(ctx.g.scheme().orange[2]),              // BlockQuote,
            Style::default().fg(ctx.g.scheme().redpink[2]),             // CodeBlock,
            Style::default().fg(ctx.g.scheme().bluegreen[3]),           // FootnodeDefinition
            Style::default().fg(ctx.g.scheme().bluegreen[2]),           // FootnodeReference
            Style::default().fg(ctx.g.scheme().orange[2]),              // ItemTag
            Style::default(),                                           // Item
            Style::default()
                .fg(ctx.g.scheme().white[3])
                .add_modifier(Modifier::ITALIC), // Emphasis
            Style::default().fg(ctx.g.scheme().white[3]),               // Strong
            Style::default().fg(ctx.g.scheme().gray[2]),                // Strikethrough
            Style::default().fg(ctx.g.scheme().bluegreen[2]),           // Link
            Style::default().fg(ctx.g.scheme().bluegreen[2]),           // Image
            Style::default().fg(ctx.g.scheme().orange[2]),              // MetadataBlock
            Style::default().fg(ctx.g.scheme().redpink[2]),             // CodeInline
            Style::default().fg(ctx.g.scheme().redpink[2]),             // MathInline
            Style::default().fg(ctx.g.scheme().redpink[2]),             // MathDisplay
            Style::default().fg(ctx.g.scheme().white[3]),               // Rule
            Style::default().fg(ctx.g.scheme().orange[2]),              // TaskListMarker
            Style::default().fg(ctx.g.scheme().gray[2]),                // Html
            Style::default().fg(ctx.g.scheme().white[1]),               // Table-Head
            Style::default(),                                           // Table
            Style::default().fg(ctx.g.scheme().white[1]),               // Table-Row
            Style::default().fg(ctx.g.scheme().white[1]),               // Table-Cell
            Style::default(), /*.bg(ctx.g.scheme().deepblue[0])*/
            // Paragraph
            Style::default(), /*.fg(ctx.g.scheme().magenta[3])*/             // List
        ]
    }

    impl HasFocusFlag for MDFileState {
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

    impl AppState<GlobalState, MDAction, Error> for MDFileState {
        fn timer(
            &mut self,
            event: &TimeOut,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<MDAction>, Error> {
            if self.parse_timer == Some(event.handle) {
                debug!("parse markdown!!");
                self.parse_markdown();
                return Ok(Control::Changed);
            }
            Ok(Control::Continue)
        }

        fn crossterm(
            &mut self,
            event: &crossterm::event::Event,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<MDAction>, Error> {
            // call markdown event-handling instead of regular.
            try_flow!(match self.edit.handle(event, MarkDown) {
                TextOutcome::TextChanged => {
                    self.text_changed(ctx)
                }
                r => r.into(),
            });

            Ok(Control::Continue)
        }
    }

    impl MDFileState {
        // New editor with fresh file.
        pub fn new_file(path: &Path) -> Self {
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
        ) -> Control<MDAction> {
            match md_format(&mut self.edit, eq_width) {
                TextOutcome::TextChanged => self.text_changed(ctx),
                r => r.into(),
            }
        }

        // Flag any text-changes.
        pub fn text_changed(&mut self, ctx: &mut AppContext<'_>) -> Control<MDAction> {
            self.changed = true;
            // send sync
            ctx.queue(Control::Message(MDAction::SyncEdit));
            // restart timer
            self.parse_timer = Some(ctx.replace_timer(
                self.parse_timer,
                TimerDef::new().next(Instant::now() + Duration::from_millis(200)),
            ));
            Control::Changed
        }
    }
}

// combined split + tab structure
pub mod split_tab {
    use crate::mdfile::{MDFile, MDFileState};
    use crate::{AppContext, AppFocus, GlobalState, MDAction};
    use anyhow::Error;
    use crossterm::event::Event;
    use log::debug;
    use rat_salsa::timer::TimeOut;
    use rat_salsa::{AppState, AppWidget, Control, RenderContext};
    use rat_widget::event::{try_flow, HandleEvent, Regular, TabbedOutcome};
    use rat_widget::focus::{ContainerFlag, Focus, HasFocus, HasFocusFlag};
    use rat_widget::splitter::{Split, SplitState, SplitType};
    use rat_widget::tabbed::attached::AttachedTabs;
    use rat_widget::tabbed::{Tabbed, TabbedState};
    use rat_widget::text::undo_buffer::UndoEntry;
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Rect};
    use ratatui::prelude::{Line, StatefulWidget};
    use ratatui::widgets::BorderType;
    use std::path::Path;

    #[derive(Debug, Default)]
    pub struct SplitTab;

    #[derive(Debug)]
    pub struct SplitTabState {
        pub focus: ContainerFlag,
        pub splitter: SplitState,
        pub sel_split: Option<usize>,
        pub tabbed: Vec<TabbedState>,
        pub tabs: Vec<Vec<MDFileState>>,
    }

    impl Default for SplitTabState {
        fn default() -> Self {
            Self {
                focus: ContainerFlag::named("split_tab"),
                splitter: SplitState::named("splitter"),
                sel_split: None,
                tabbed: vec![],
                tabs: vec![],
            }
        }
    }

    impl AppWidget<GlobalState, MDAction, Error> for SplitTab {
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
                            ctx.g.theme.tabbed_style().focus_style.expect("style")
                        } else if md.is_focused() {
                            ctx.g.theme.primary(1)
                        } else {
                            ctx.g.theme.tabbed_style().select_style.expect("style")
                        }
                    } else {
                        ctx.g.theme.tabbed_style().select_style.expect("style")
                    }
                } else {
                    ctx.g.theme.tabbed_style().select_style.expect("style")
                };

                Tabbed::new()
                    .tab_type(AttachedTabs::new().join_1(BorderType::Rounded))
                    .closeable(true)
                    .styles(ctx.g.theme.tabbed_style())
                    .select_style(select_style)
                    .tabs(state.tabs[idx_split].iter().map(|v| {
                        let title = v.path.file_name().unwrap_or_default().to_string_lossy();
                        Line::from(title.to_string())
                    }))
                    .render(*edit_area, buf, &mut state.tabbed[idx_split]);

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
        fn focus(&self) -> Focus {
            let mut f = Focus::new_container(self.focus.clone(), Rect::default());
            f.add(&self.splitter);
            for (idx_split, tabbed) in self.tabbed.iter().enumerate() {
                f.add(&self.tabbed[idx_split]);
                if let Some(idx_tab) = tabbed.selected() {
                    f.add(&self.tabs[idx_split][idx_tab]);
                }
            }
            f
        }

        fn container(&self) -> Option<ContainerFlag> {
            Some(self.focus.clone())
        }
    }

    impl AppFocus<GlobalState, MDAction, Error> for SplitTabState {
        fn focus_changed(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            // Find which split contains the current focus.
            let old_split = self.sel_split;
            for (idx_split, tabbed) in self.tabbed.iter().enumerate() {
                if let Some(idx_tab) = tabbed.selected() {
                    if self.tabs[idx_split][idx_tab].gained_focus() {
                        self.sel_split = Some(idx_split);
                        break;
                    }
                }
            }
            debug!("focus_changed {:?} <> {:?}", old_split, self.sel_split);
            if old_split != self.sel_split {
                if let Some((_, md)) = self.selected() {
                    ctx.queue(Control::Message(MDAction::FocusedFile(md.path.clone())));
                }
            }

            Ok(())
        }
    }

    impl AppState<GlobalState, MDAction, Error> for SplitTabState {
        fn timer(
            &mut self,
            event: &TimeOut,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<MDAction>, Error> {
            for split in &mut self.tabs {
                for tab in split {
                    try_flow!(tab.timer(event, ctx)?);
                }
            }
            Ok(Control::Continue)
        }

        fn crossterm(
            &mut self,
            event: &Event,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<MDAction>, Error> {
            try_flow!(self.splitter.handle(event, Regular));
            for (idx_split, tabbed) in self.tabbed.iter_mut().enumerate() {
                try_flow!(match tabbed.handle(event, Regular) {
                    TabbedOutcome::Close(n) => {
                        Control::Message(MDAction::CloseAt(idx_split, n))
                    }
                    TabbedOutcome::Select(n) => {
                        Control::Message(MDAction::SelectAt(idx_split, n))
                    }
                    r => r.into(),
                });

                if let Some(idx_tab) = tabbed.selected() {
                    try_flow!(self.tabs[idx_split][idx_tab].crossterm(event, ctx)?);
                }
            }

            Ok(Control::Continue)
        }
    }

    impl SplitTabState {
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
                    return tab.save();
                }
            }
            Ok(())
        }

        // Run the replay for the file at path.
        pub fn replay(&mut self, id: (usize, usize), path: &Path, replay: &[UndoEntry]) {
            for (idx_split, tabs) in self.tabs.iter_mut().enumerate() {
                for (idx_tab, tab) in tabs.iter_mut().enumerate() {
                    if id != (idx_split, idx_tab) && tab.path == path {
                        tab.edit.replay_log(replay);
                    }
                }
            }
        }
    }
}

// md files in current directory.
pub mod file_list {
    use crate::{AppFocus, GlobalState, MDAction};
    use anyhow::Error;
    use crossterm::event::Event;
    use rat_salsa::event::{ct_event, try_flow};
    use rat_salsa::{AppContext, AppState, AppWidget, Control, RenderContext};
    use rat_widget::event::{HandleEvent, Regular};
    use rat_widget::focus::{FocusFlag, HasFocusFlag};
    use rat_widget::list::selection::RowSelection;
    use rat_widget::list::{List, ListState};
    use rat_widget::scrolled::Scroll;
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Layout, Rect};
    use ratatui::prelude::Line;
    use ratatui::widgets::StatefulWidget;
    use std::fs;
    use std::path::{Path, PathBuf};

    #[derive(Debug, Default)]
    pub struct FileList;

    #[derive(Debug)]
    pub struct FileListState {
        pub files_dir: PathBuf,
        pub files: Vec<PathBuf>,
        pub file_list: ListState<RowSelection>,
    }

    impl Default for FileListState {
        fn default() -> Self {
            Self {
                files_dir: Default::default(),
                files: vec![],
                file_list: ListState::named("file_list"),
            }
        }
    }

    impl AppWidget<GlobalState, MDAction, Error> for FileList {
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

            buf.set_style(l_file_list[0], ctx.g.theme.data());

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

            Ok(())
        }
    }

    impl HasFocusFlag for FileListState {
        fn focus(&self) -> FocusFlag {
            self.file_list.focus()
        }

        fn area(&self) -> Rect {
            self.file_list.area()
        }
    }

    impl AppFocus<GlobalState, MDAction, Error> for FileListState {
        fn focus_changed(&mut self, ctx: &mut crate::AppContext<'_>) -> Result<(), Error> {
            Ok(())
        }
    }

    impl AppState<GlobalState, MDAction, Error> for FileListState {
        fn init(
            &mut self,
            ctx: &mut AppContext<'_, GlobalState, MDAction, Error>,
        ) -> Result<(), Error> {
            self.load(&Path::new("."))?;
            Ok(())
        }

        fn crossterm(
            &mut self,
            event: &Event,
            ctx: &mut AppContext<'_, GlobalState, MDAction, Error>,
        ) -> Result<Control<MDAction>, Error> {
            if self.file_list.is_focused() {
                try_flow!(match event {
                    ct_event!(keycode press Enter) => {
                        if let Some(row) = self.file_list.selected() {
                            Control::Message(MDAction::SelectOrOpen(self.files[row].clone()))
                        } else {
                            Control::Continue
                        }
                    }
                    ct_event!(key press '+') => {
                        if let Some(row) = self.file_list.selected() {
                            Control::Message(MDAction::SelectOrOpenSplit(self.files[row].clone()))
                        } else {
                            Control::Continue
                        }
                    }
                    _ => Control::Continue,
                });
            }
            try_flow!(match event {
                ct_event!(mouse any for m)
                    if self.file_list.mouse.doubleclick(self.file_list.area, m) =>
                {
                    if let Some(row) = self.file_list.row_at_clicked((m.column, m.row)) {
                        Control::Message(MDAction::SelectOrOpen(self.files[row].clone()))
                    } else {
                        Control::Continue
                    }
                }

                _ => Control::Continue,
            });
            try_flow!(self.file_list.handle(event, Regular));

            Ok(Control::Continue)
        }
    }

    impl FileListState {
        /// Read directory.
        pub fn load(&mut self, dir: &Path) -> Result<(), Error> {
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

// overall editor
pub mod mdedit {
    use crate::file_list::{FileList, FileListState};
    use crate::mdfile::MDFileState;
    use crate::split_tab::{SplitTab, SplitTabState};
    use crate::{AppContext, AppFocus, GlobalState, MDAction, RenderContext};
    use anyhow::Error;
    use crossterm::event::Event;
    #[allow(unused_imports)]
    use log::debug;
    use rat_salsa::event::{ct_event, try_flow};
    use rat_salsa::timer::TimeOut;
    use rat_salsa::{AppState, AppWidget, Control};
    use rat_widget::event::{HandleEvent, Regular};
    use rat_widget::focus::{Focus, HasFocus, HasFocusFlag};
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

    impl AppWidget<GlobalState, MDAction, Error> for MDEdit {
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
        fn focus(&self) -> Focus {
            let mut f = Focus::default();
            f.add(&self.file_list);
            f.add_container(&self.split_tab);
            f
        }
    }

    impl AppFocus<GlobalState, MDAction, Error> for MDEditState {
        fn focus_changed(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            self.file_list.focus_changed(ctx)?;
            self.split_tab.focus_changed(ctx)?;
            Ok(())
        }
    }

    impl AppState<GlobalState, MDAction, Error> for MDEditState {
        fn init(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            self.file_list.load(&Path::new("."))?;
            Ok(())
        }

        fn timer(
            &mut self,
            event: &TimeOut,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<MDAction>, Error> {
            try_flow!(self.split_tab.timer(event, ctx)?);
            Ok(Control::Changed)
        }

        fn crossterm(
            &mut self,
            event: &Event,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<MDAction>, Error> {
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
                        Control::Message(MDAction::Close)
                    }
                    ct_event!(key press CONTROL-'d')
                    | ct_event!(key press 'd')
                    | ct_event!(key press '+') => {
                        Control::Message(MDAction::Split)
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
                    debug!("ctrl+n");
                    Control::Message(MDAction::MenuNew)
                }
                ct_event!(key press CONTROL-'o') => {
                    debug!("ctrl+o");
                    Control::Message(MDAction::MenuOpen)
                }
                ct_event!(key press CONTROL-'s') => {
                    debug!("ctrl+s");
                    Control::Message(MDAction::Save)
                }
                ct_event!(keycode press F(2)) => {
                    ctx.focus().focus(&self.file_list);
                    Control::Changed
                }
                ct_event!(key press CONTROL-'w') => {
                    self.window_cmd = true;
                    Control::Changed
                }
                ct_event!(focus_lost) => {
                    Control::Message(MDAction::Save)
                }
                _ => Control::Continue,
            });

            try_flow!(self.split_files.handle(event, Regular));

            try_flow!(self.file_list.crossterm(event, ctx)?);
            try_flow!(self.split_tab.crossterm(event, ctx)?);

            Ok(Control::Continue)
        }

        fn message(
            &mut self,
            event: &mut MDAction,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, MDAction, Error>,
        ) -> Result<Control<MDAction>, Error> {
            try_flow!(match event {
                MDAction::New(p) => {
                    self.new(p, ctx)?;
                    Control::Changed
                }
                MDAction::SelectOrOpen(p) => {
                    self.select_or_open(p, ctx)?;
                    Control::Changed
                }
                MDAction::SelectOrOpenSplit(p) => {
                    self.select_or_open_split(p, ctx)?;
                    Control::Changed
                }
                MDAction::Open(p) => {
                    self.open(p, ctx)?;
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
                MDAction::Close => {
                    if let Some(pos) = self.split_tab.selected_pos() {
                        self.split_tab.close((pos.0, pos.1), ctx)?;
                        Control::Changed
                    } else {
                        Control::Continue
                    }
                }
                MDAction::CloseAt(idx_split, idx_tab) => {
                    self.split_tab.close((*idx_split, *idx_tab), ctx)?;
                    Control::Changed
                }
                MDAction::SelectAt(idx_split, idx_tab) => {
                    self.split_tab.select((*idx_split, *idx_tab), ctx);
                    Control::Changed
                }

                MDAction::Split => {
                    self.split(ctx)?;
                    Control::Changed
                }

                MDAction::SyncEdit => {
                    // synchronize instances
                    let (id_sel, sel_path, replay) =
                        if let Some((id_sel, sel)) = self.split_tab.selected_mut() {
                            (id_sel, sel.path.clone(), sel.edit.recent_replay_log())
                        } else {
                            ((0, 0), PathBuf::default(), Vec::default())
                        };
                    if !replay.is_empty() {
                        self.split_tab.replay(id_sel, &sel_path, &replay);
                    }
                    Control::Changed
                }

                MDAction::FocusedFile(p) => {
                    debug!("focused file {:?}", p);
                    if let Some(parent) = p.parent() {
                        self.file_list.load(parent)?;
                    }
                    self.file_list.select(p)?;
                    Control::Changed
                }

                _ => Control::Continue,
            });
            Ok(Control::Continue)
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

            let new = MDFileState::new_file(&path);
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
