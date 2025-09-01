#![allow(dead_code)]

use crate::event::LogScrollEvent;
use crate::scenery::{Scenery, SceneryState};
use anyhow::Error;
use rat_salsa::poll::{PollCrossterm, PollTimers};
use rat_salsa::{run_tui, RunConfig};
use rat_theme2::palettes::IMPERIAL;
use rat_theme2::DarkTheme;
use std::env::args;
use std::fs;
use std::path::PathBuf;

type AppContext<'a> = rat_salsa::AppContext<'a, GlobalState, LogScrollEvent, Error>;
type RenderContext<'a> = rat_salsa::RenderContext<'a, GlobalState>;

fn main() -> Result<(), Error> {
    setup_logging()?;

    let mut args = args();
    args.next();
    let Some(current) = args.next() else {
        eprintln!("usage: logscroll <file>");
        return Ok(());
    };
    let current = PathBuf::from(current);
    if !current.exists() {
        eprintln!("file {:?} does not exist.", current);
        return Ok(());
    }

    let config = LogScrollConfig::default();
    let theme = DarkTheme::new("Imperial".into(), IMPERIAL);

    let mut global = GlobalState::new(config, theme);
    global.current = current;

    let app = Scenery;
    let mut state = SceneryState::default();

    run_tui(
        app,
        &mut global,
        &mut state,
        RunConfig::default()?
            .poll(PollCrossterm)
            .poll(PollTimers::new()),
    )?;

    Ok(())
}

mod event {
    use rat_salsa::timer::TimeOut;

    #[derive(Debug)]
    pub enum LogScrollEvent {
        Event(crossterm::event::Event),
        TimeOut(TimeOut),
        Message(String),
        Status(usize, String),
    }

    impl From<crossterm::event::Event> for LogScrollEvent {
        fn from(value: crossterm::event::Event) -> Self {
            Self::Event(value)
        }
    }

    impl From<TimeOut> for LogScrollEvent {
        fn from(value: TimeOut) -> Self {
            Self::TimeOut(value)
        }
    }
}

mod scenery {
    use crate::event::LogScrollEvent;
    use crate::logscroll::{LogScroll, LogScrollState};
    use crate::{AppContext, GlobalState, RenderContext};
    use anyhow::Error;
    use log::debug;
    use rat_salsa::{AppState, AppWidget, Control};
    use rat_widget::event::{ct_event, ConsumedEvent, Dialog, HandleEvent, Regular};
    use rat_widget::focus::FocusBuilder;
    use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
    use rat_widget::statusline::{StatusLine, StatusLineState};
    use rat_widget::text::HasScreenCursor;
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Layout, Rect};
    use ratatui::widgets::StatefulWidget;
    use std::time::{Duration, SystemTime};

    #[derive(Debug)]
    pub struct Scenery;

    #[derive(Debug, Default)]
    pub struct SceneryState {
        pub logscroll: LogScrollState,
        pub status: StatusLineState,
        pub error_dlg: MsgDialogState,
    }

    impl AppWidget<GlobalState, LogScrollEvent, Error> for Scenery {
        type State = SceneryState;

        fn render(
            &self,
            area: Rect,
            buf: &mut Buffer,
            state: &mut Self::State,
            ctx: &mut RenderContext<'_>,
        ) -> Result<(), Error> {
            let t0 = SystemTime::now();

            let layout = Layout::vertical([
                Constraint::Length(1),
                Constraint::Fill(1), //
                Constraint::Length(1),
            ])
            .split(area);

            LogScroll.render(area, buf, &mut state.logscroll, ctx)?;

            ctx.set_screen_cursor(state.logscroll.screen_cursor());

            if state.error_dlg.active() {
                let err = MsgDialog::new().styles(ctx.g.theme.msg_dialog_style());
                err.render(area, buf, &mut state.error_dlg);
            }

            let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
            state
                .status
                .status(2, format!("R{} {:.0?}", ctx.count, el).to_string());

            let status = StatusLine::new()
                .layout([
                    Constraint::Fill(1),
                    Constraint::Length(12),
                    Constraint::Length(12),
                    Constraint::Length(12),
                ])
                .styles(ctx.g.theme.statusline_style());
            status.render(layout[2], buf, &mut state.status);

            Ok(())
        }
    }

    impl AppState<GlobalState, LogScrollEvent, Error> for SceneryState {
        fn init(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            ctx.focus = Some(FocusBuilder::build_for(&self.logscroll));
            ctx.focus().enable_log();
            self.logscroll.init(ctx)?;
            Ok(())
        }

        fn event(
            &mut self,
            event: &LogScrollEvent,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, LogScrollEvent, Error>,
        ) -> Result<Control<LogScrollEvent>, Error> {
            let t0 = SystemTime::now();

            let mut r = match event {
                LogScrollEvent::Event(event) => {
                    ctx.focus = Some(FocusBuilder::rebuild_for(&self.logscroll, ctx.focus.take()));
                    ctx.focus().enable_log();

                    let mut r = match &event {
                        ct_event!(resized) => Control::Changed,
                        ct_event!(key press CONTROL-'q') => Control::Quit,
                        _ => Control::Continue,
                    };

                    r = r.or_else(|| {
                        if self.error_dlg.active() {
                            self.error_dlg.handle(event, Dialog).into()
                        } else {
                            Control::Continue
                        }
                    });

                    let f = ctx.focus_mut().handle(event, Regular);
                    ctx.queue(f);

                    r
                }
                LogScrollEvent::Message(s) => {
                    self.error_dlg.append(s.as_str());
                    Control::Changed
                }
                LogScrollEvent::Status(n, s) => {
                    self.status.status(*n, s);
                    Control::Changed
                }
                _ => Control::Continue,
            };

            r = r.or_else_try(|| self.logscroll.event(event, ctx))?;

            let el = t0.elapsed()?;
            self.status.status(3, format!("E {:.0?}", el).to_string());

            Ok(r)
        }

        fn error(
            &self,
            event: Error,
            _ctx: &mut AppContext<'_>,
        ) -> Result<Control<LogScrollEvent>, Error> {
            debug!("ERROR {:#?}", event);
            // self.error_dlg.append(format!("{:?}", &*event).as_str());
            Ok(Control::Changed)
        }
    }
}

mod logscroll {
    use crate::event::LogScrollEvent;
    use crate::{AppContext, GlobalState, RenderContext};
    use anyhow::Error;
    use log::debug;
    use rat_salsa::timer::{TimerDef, TimerHandle};
    use rat_salsa::{AppState, AppWidget, Control};
    use rat_theme2::DarkTheme;
    use rat_widget::caption::{Caption, CaptionState, HotkeyPolicy};
    use rat_widget::event::{
        ct_event, try_flow, HandleEvent, ReadOnly, Regular, TableOutcome, TextOutcome,
    };
    use rat_widget::focus::{impl_has_focus, HasFocus, Navigation};
    use rat_widget::paired::{PairSplit, Paired, PairedState};
    use rat_widget::scrolled::Scroll;
    use rat_widget::splitter::{Split, SplitState};
    use rat_widget::table::selection::RowSelection;
    use rat_widget::table::{Table, TableContext, TableData, TableState};
    use rat_widget::text::{impl_screen_cursor, TextPosition};
    use rat_widget::text_input::{TextInput, TextInputState};
    use rat_widget::textarea::{TextArea, TextAreaState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Layout, Rect};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{StatefulWidget, Widget};
    use regex_cursor::engines::dfa::{find_iter, Regex};
    use regex_cursor::{Input, RopeyCursor};
    use ropey::RopeBuilder;
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};
    use std::ops::Range;
    use std::path::Path;
    use std::time::Duration;

    #[derive(Debug)]
    pub(crate) struct LogScroll;

    #[derive(Debug)]
    pub struct LogScrollState {
        split: SplitState,
        logtext: TextAreaState,
        find_label: CaptionState,
        find: TextInputState,
        find_matches: Vec<Range<usize>>,
        find_table: TableState<RowSelection>,
        pollution: TimerHandle,
    }

    impl Default for LogScrollState {
        fn default() -> Self {
            let mut zelf = Self {
                split: Default::default(),
                logtext: Default::default(),
                find_label: Default::default(),
                find: Default::default(),
                find_matches: Default::default(),
                find_table: Default::default(),
                pollution: Default::default(),
            };
            zelf.logtext.set_focus_navigation(Navigation::Regular);
            zelf
        }
    }

    impl AppWidget<GlobalState, LogScrollEvent, Error> for LogScroll {
        type State = LogScrollState;

        fn render(
            &self,
            area: Rect,
            buf: &mut Buffer,
            state: &mut Self::State,
            ctx: &mut RenderContext<'_>,
        ) -> Result<(), Error> {
            let l0 = Layout::vertical([
                Constraint::Length(1),
                Constraint::Fill(1), //
                Constraint::Length(1),
            ])
            .split(area);

            let (split, ls) = Split::vertical()
                .styles(ctx.g.theme.split_style())
                .constraints([
                    Constraint::Fill(1), //
                    Constraint::Length(25),
                ])
                .into_widget_layout(l0[1], &mut state.split);

            // left side

            TextArea::new()
                .vscroll(Scroll::new())
                .hscroll(Scroll::new())
                .styles(ctx.g.theme.textarea_style())
                .text_style_idx(99, ctx.g.theme.secondary(2))
                .text_style_idx(100, ctx.g.theme.limegreen(2))
                .render(ls[0], buf, &mut state.logtext);

            // right side

            let l2 = Layout::vertical([
                Constraint::Length(1), //
                Constraint::Fill(1),
            ])
            .split(ls[1]);

            Paired::new(
                Caption::parse("_Find| F3 ")
                    .styles(ctx.g.theme.caption_style())
                    .hotkey_policy(HotkeyPolicy::Always)
                    .spacing(1)
                    .link(&state.find.focus()),
                TextInput::new().styles(ctx.g.theme.text_style()),
            )
            .split(PairSplit::Fix1(8))
            .render(
                l2[0],
                buf,
                &mut PairedState::new(&mut state.find_label, &mut state.find),
            );

            Table::new()
                .vscroll(Scroll::new())
                .styles(ctx.g.theme.table_style())
                .data(FindData {
                    theme: &ctx.g.theme,
                    text: &state.logtext,
                    data: &state.find_matches,
                })
                .render(l2[1], buf, &mut state.find_table);

            split.render(l0[1], buf, &mut state.split);

            Ok(())
        }
    }

    struct FindData<'a> {
        theme: &'a DarkTheme,
        text: &'a TextAreaState,
        data: &'a [Range<usize>],
    }

    impl<'a> TableData<'a> for FindData<'a> {
        fn rows(&self) -> usize {
            self.data.len()
        }

        fn row_height(&self, _: usize) -> u16 {
            2
        }

        fn widths(&self) -> Vec<Constraint> {
            vec![Constraint::Fill(1)]
        }

        fn render_cell(
            &self,
            _ctx: &TableContext,
            _column: usize,
            row: usize,
            area: Rect,
            buf: &mut Buffer,
        ) {
            let data = self.data[row].clone();

            let line = self.text.byte_pos(data.start);

            let line_byte = self.text.byte_at(TextPosition::new(0, line.y));
            let data_start = data.start - line_byte.start;
            let data_end = data.end - line_byte.start;

            let line_text = self.text.line_at(line.y);
            let line_prefix = &line_text[..data_start];
            let line_match = &line_text[data_start..data_end];
            let line_suffix = &line_text[data_end..];

            let pos_txt = format!("\u{2316} {}:{}", line.x, line.y);
            let l0 = Line::from_iter([
                Span::from(pos_txt), //
            ])
            .style(self.theme.deepblue(7));
            let l1 = Line::from_iter([
                Span::from(line_prefix),
                Span::from(line_match).style(self.theme.secondary(1)),
                Span::from(line_suffix),
            ]);

            l0.render(Rect::new(area.x, area.y, area.width, 1), buf);
            l1.render(Rect::new(area.x, area.y + 1, area.width, 1), buf);
        }
    }

    impl_has_focus!(logtext, split, find, find_table for LogScrollState);
    impl_screen_cursor!(logtext, find for LogScrollState);

    impl LogScrollState {
        fn load_file(&mut self, path: &Path) -> Result<(), Error> {
            let mut f = File::open(path)?;

            let mut buf = String::with_capacity(4096);
            let mut txt = RopeBuilder::new();
            loop {
                match f.read_to_string(&mut buf) {
                    Ok(0) => {
                        break;
                    }
                    Ok(_) => {
                        txt.append(&buf);
                    }
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }
            self.logtext.set_rope(txt.finish());
            self.logtext
                .set_cursor((0, self.logtext.len_lines()), false);
            self.logtext.scroll_cursor_to_visible();

            Ok(())
        }

        fn update_file(&mut self, path: &Path) -> Result<bool, Error> {
            let rope = self.logtext.rope().clone();
            let old_len_bytes = rope.len_bytes();

            if path.metadata()?.len() == old_len_bytes as u64 {
                return Ok(false);
            }

            let mut f = File::open(path)?;
            f.seek(SeekFrom::Start(old_len_bytes as u64))?;

            let cursor_at_end = self.logtext.cursor().y == self.logtext.len_lines()
                || self.logtext.cursor().y == self.logtext.len_lines() - 1;

            let mut buf = String::with_capacity(4096);
            loop {
                match f.read_to_string(&mut buf) {
                    Ok(0) => {
                        break;
                    }
                    Ok(_) => {
                        let pos = TextPosition::new(0, self.logtext.len_lines());
                        self.logtext.value.insert_str(pos, &buf).expect("fine");
                    }
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }

            if cursor_at_end {
                self.logtext
                    .value
                    .set_cursor(TextPosition::new(0, self.logtext.len_lines()), false);
                self.logtext.set_vertical_offset(
                    (self.logtext.len_lines() as usize)
                        .saturating_sub(self.logtext.vertical_page()),
                );
            }

            // update find
            let find = self.find.text();
            if !find.is_empty() {
                if let Ok(re) = Regex::new(find) {
                    let new_len_bytes = self.logtext.rope().len_bytes();

                    let cursor = RopeyCursor::new(
                        self.logtext.rope().byte_slice(old_len_bytes..new_len_bytes),
                    );
                    let input = Input::new(cursor);

                    for m in find_iter(&re, input) {
                        self.find_matches
                            .push(old_len_bytes + m.start()..old_len_bytes + m.end());
                    }
                    for m in &self.find_matches {
                        self.logtext.add_style(m.clone(), 99);
                    }
                }
            }

            Ok(true)
        }

        fn run_search(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            let text = self.find.text();

            if text.is_empty() {
                ctx.queue_event(LogScrollEvent::Status(0, String::default()));
                self.find.set_invalid(false);
                self.find_matches.clear();
                self.logtext.set_styles(Vec::default());
                self.find_table.clear_offset();
                self.find_table.clear_selection();
                return Ok(());
            }

            match Regex::new(text) {
                Ok(re) => {
                    if self.find.invalid() {
                        ctx.queue_event(LogScrollEvent::Status(0, String::default()));
                        self.find.set_invalid(false);
                    }

                    let cursor = RopeyCursor::new(self.logtext.rope().byte_slice(..));
                    let input = Input::new(cursor);
                    let mut matches = Vec::new();
                    self.find_matches.clear();
                    self.find_table.clear_offset();
                    self.find_table.clear_selection();
                    for m in find_iter(&re, input) {
                        self.find_matches.push(m.range());
                        matches.push((m.range(), 99));
                    }
                    self.logtext.set_styles(matches);
                }
                Err(err) => {
                    ctx.queue_event(LogScrollEvent::Status(0, format!("{:?}", err)));
                    self.find.set_invalid(true);
                    self.find_matches.clear();
                    self.find_table.clear_offset();
                    self.find_table.clear_selection();
                }
            }

            Ok(())
        }
    }

    impl AppState<GlobalState, LogScrollEvent, Error> for LogScrollState {
        fn init(
            &mut self,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, LogScrollEvent, Error>,
        ) -> Result<(), Error> {
            ctx.focus().first();

            self.load_file(&ctx.g.current)?;

            ctx.add_timer(
                TimerDef::new()
                    .repeat_forever()
                    .timer(Duration::from_millis(500)),
            );

            self.pollution = ctx.add_timer(
                TimerDef::new()
                    .repeat_forever()
                    .timer(Duration::from_millis(1000)),
            );

            Ok(())
        }

        #[allow(unused_variables)]
        fn event(
            &mut self,
            event: &LogScrollEvent,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<LogScrollEvent>, Error> {
            let r = match event {
                LogScrollEvent::Event(event) => {
                    try_flow!(match event {
                        ct_event!(keycode press F(2)) => {
                            if !self.split.is_focused() {
                                ctx.focus().focus(&self.split);
                            } else {
                                ctx.focus().next();
                            }
                            Control::Changed
                        }
                        ct_event!(key press ALT-'f') | ct_event!(keycode press F(3)) => {
                            if self.split.is_hidden(1) {
                                self.split.show_split(1);
                                ctx.focus().focus(&self.find);
                            } else {
                                self.split.hide_split(1);
                                ctx.focus().focus(&self.logtext);
                            }
                            Control::Changed
                        }
                        _ => Control::Continue,
                    });

                    try_flow!(self.split.handle(event, Regular));
                    try_flow!(self.logtext.handle(event, ReadOnly));
                    try_flow!(self.find_label.handle(event, ctx.focus()));
                    try_flow!(match self.find.handle(event, Regular) {
                        TextOutcome::TextChanged => {
                            self.run_search(ctx)?;
                            Control::Changed
                        }
                        r => r.into(),
                    });
                    try_flow!(match self.find_table.handle(event, Regular) {
                        TableOutcome::Selected => {
                            if let Some(selected) = self.find_table.selected() {
                                let range = self.find_matches[selected].clone();
                                let text_range = self.logtext.byte_range(range.clone());
                                self.logtext.set_cursor(text_range.start, false);

                                let old_style = self.logtext.styles().find(|(_, s)| *s == 100);
                                if let Some((range, style)) = old_style {
                                    self.logtext.remove_style(range, style);
                                }
                                self.logtext.add_style(range, 100);
                            }
                            Control::Changed
                        }
                        r => r.into(),
                    });

                    Control::Continue
                }
                LogScrollEvent::TimeOut(t) if t.handle == self.pollution => {
                    debug!("log-pollution {:?}", t);
                    Control::Continue
                }
                LogScrollEvent::TimeOut(t) if t.handle != self.pollution => {
                    if self.update_file(&ctx.g.current)? {
                        ctx.queue(Control::Event(LogScrollEvent::Status(
                            1,
                            format!("{}l", self.logtext.len_lines()),
                        )));
                        Control::Changed
                    } else {
                        Control::Continue
                    }
                }
                _ => Control::Continue,
            };

            // TODO

            Ok(r)
        }
    }
}

#[derive(Debug, Default)]
pub struct LogScrollConfig {}

#[derive(Debug)]
pub struct GlobalState {
    pub cfg: LogScrollConfig,
    pub theme: DarkTheme,
    pub current: PathBuf,
}

impl GlobalState {
    pub fn new(cfg: LogScrollConfig, theme: DarkTheme) -> Self {
        Self {
            cfg,
            theme,
            current: Default::default(),
        }
    }
}

fn setup_logging() -> Result<(), Error> {
    if let Some(_cache) = dirs::cache_dir() {
        // TODO:??
        // let log_path = if cfg!(debug_assertions) {
        //     PathBuf::from(".")
        // } else {
        //     let log_path = cache.join("rat-salsa");
        //     if !log_path.exists() {
        //         fs::create_dir_all(&log_path)?;
        //     }
        //     log_path
        // };
        let log_path = PathBuf::from(".");

        let log_file = log_path.join("logscroll.log");
        _ = fs::remove_file(&log_file);
        fern::Dispatch::new()
            .format(|out, message, _record| {
                out.finish(format_args!("{}", message)) //
            })
            .level(log::LevelFilter::Debug)
            .chain(fern::log_file(&log_file)?)
            .apply()?;
    }
    Ok(())
}
