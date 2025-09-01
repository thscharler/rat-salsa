#![allow(dead_code)]

use crate::event::LogScrollEvent;
use crate::scenery::Scenery;
use anyhow::Error;
use rat_salsa2::poll::{PollCrossterm, PollTimers};
use rat_salsa2::{run_tui, RunConfig};
use rat_theme2::palettes::IMPERIAL;
use rat_theme2::DarkTheme;
use std::env::args;
use std::fs;
use std::path::PathBuf;

type AppContext<'a> = rat_salsa2::AppContext<'a, GlobalState, LogScrollEvent, Error>;
type RenderContext<'a> = rat_salsa2::RenderContext<'a, GlobalState>;

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

    let mut state = Scenery::default();

    run_tui(
        scenery::init,
        scenery::render,
        scenery::event,
        scenery::error,
        &mut global,
        &mut state,
        RunConfig::default()?
            .poll(PollCrossterm)
            .poll(PollTimers::new()),
    )?;

    Ok(())
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

mod event {
    use rat_salsa2::timer::TimeOut;

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
    use crate::logscroll::LogScroll;
    use crate::{logscroll, AppContext, RenderContext};
    use anyhow::Error;
    use log::debug;
    use rat_salsa2::Control;
    use rat_widget::event::{ct_event, ConsumedEvent, Dialog, HandleEvent, Regular};
    use rat_widget::focus::FocusBuilder;
    use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
    use rat_widget::statusline::{StatusLine, StatusLineState};
    use rat_widget::text::HasScreenCursor;
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Layout, Rect};
    use ratatui::widgets::StatefulWidget;
    use std::time::{Duration, SystemTime};

    #[derive(Debug, Default)]
    pub struct Scenery {
        pub logscroll: LogScroll,
        pub status: StatusLineState,
        pub error_dlg: MsgDialogState,
    }

    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        state: &mut Scenery,
        ctx: &mut RenderContext<'_>,
    ) -> Result<(), Error> {
        let t0 = SystemTime::now();

        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1), //
            Constraint::Length(1),
        ])
        .split(area);

        logscroll::render(area, buf, &mut state.logscroll, ctx)?;

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

    pub fn init(state: &mut Scenery, ctx: &mut AppContext<'_>) -> Result<(), Error> {
        ctx.focus = Some(FocusBuilder::build_for(&state.logscroll));
        ctx.focus().enable_log();
        logscroll::init(&mut state.logscroll, ctx)?;
        Ok(())
    }

    pub fn event(
        event: &LogScrollEvent,
        state: &mut Scenery,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<LogScrollEvent>, Error> {
        let t0 = SystemTime::now();

        let mut r = match event {
            LogScrollEvent::Event(event) => {
                ctx.focus = Some(FocusBuilder::rebuild_for(
                    &state.logscroll,
                    ctx.focus.take(),
                ));
                ctx.focus().enable_log();

                let mut r = match &event {
                    ct_event!(resized) => Control::Changed,
                    ct_event!(key press CONTROL-'q') => Control::Quit,
                    _ => Control::Continue,
                };

                r = r.or_else(|| {
                    if state.error_dlg.active() {
                        state.error_dlg.handle(event, Dialog).into()
                    } else {
                        Control::Continue
                    }
                });

                let f = ctx.focus_mut().handle(event, Regular);
                ctx.queue(f);

                r
            }
            LogScrollEvent::Message(s) => {
                state.error_dlg.append(s.as_str());
                Control::Changed
            }
            LogScrollEvent::Status(n, s) => {
                state.status.status(*n, s);
                Control::Changed
            }
            _ => Control::Continue,
        };

        r = r.or_else_try(|| logscroll::event(event, &mut state.logscroll, ctx))?;

        let el = t0.elapsed()?;
        state.status.status(3, format!("E {:.0?}", el).to_string());

        Ok(r)
    }

    pub fn error(
        event: Error,
        _state: &mut Scenery,
        _ctx: &mut AppContext<'_>,
    ) -> Result<Control<LogScrollEvent>, Error> {
        debug!("ERROR {:#?}", event);
        // self.error_dlg.append(format!("{:?}", &*event).as_str());
        Ok(Control::Changed)
    }
}

mod logscroll {
    use crate::event::LogScrollEvent;
    use crate::{AppContext, RenderContext};
    use anyhow::Error;
    use log::debug;
    use rat_salsa2::timer::{TimerDef, TimerHandle};
    use rat_salsa2::Control;
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
    pub struct LogScroll {
        split: SplitState,
        logtext: TextAreaState,
        find_label: CaptionState,
        find: TextInputState,
        find_matches: Vec<Range<usize>>,
        find_table: TableState<RowSelection>,
        pollution: TimerHandle,
    }

    impl Default for LogScroll {
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

    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        state: &mut LogScroll,
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

    impl_has_focus!(logtext, split, find, find_table for LogScroll);
    impl_screen_cursor!(logtext, find for LogScroll);

    fn load_file(state: &mut LogScroll, path: &Path) -> Result<(), Error> {
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
        state.logtext.set_rope(txt.finish());
        state.logtext.set_styles(Vec::default());
        state
            .logtext
            .set_cursor((0, state.logtext.len_lines()), false);
        state.logtext.scroll_cursor_to_visible();

        Ok(())
    }

    fn log_grows(state: &mut LogScroll, path: &Path) -> Result<bool, Error> {
        let rope = state.logtext.rope().clone();
        let old_len_bytes = rope.len_bytes();

        if path.metadata()?.len() > old_len_bytes as u64 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn log_shrinks(state: &mut LogScroll, path: &Path) -> Result<bool, Error> {
        let rope = state.logtext.rope().clone();
        let old_len_bytes = rope.len_bytes();

        if path.metadata()?.len() < old_len_bytes as u64 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn update_file(state: &mut LogScroll, path: &Path) -> Result<bool, Error> {
        if log_shrinks(state, path)? {
            unreachable!("log shrink");
        }
        if !log_grows(state, path)? {
            return Ok(false);
        }

        let rope = state.logtext.rope().clone();
        let old_len_bytes = rope.len_bytes();

        let mut f = File::open(path)?;
        f.seek(SeekFrom::Start(old_len_bytes as u64))?;

        let cursor_at_end = state.logtext.cursor().y == state.logtext.len_lines()
            || state.logtext.cursor().y == state.logtext.len_lines() - 1;

        let mut buf = String::with_capacity(4096);
        loop {
            match f.read_to_string(&mut buf) {
                Ok(0) => {
                    break;
                }
                Ok(_) => {
                    let pos = TextPosition::new(0, state.logtext.len_lines());
                    state.logtext.value.insert_str(pos, &buf).expect("fine");
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }

        if cursor_at_end {
            state
                .logtext
                .value
                .set_cursor(TextPosition::new(0, state.logtext.len_lines()), false);
            state.logtext.set_vertical_offset(
                (state.logtext.len_lines() as usize).saturating_sub(state.logtext.vertical_page()),
            );
        }

        // update find
        let find = state.find.text();
        if !find.is_empty() {
            if let Ok(re) = Regex::new(find) {
                let new_len_bytes = state.logtext.rope().len_bytes();

                let cursor = RopeyCursor::new(
                    state
                        .logtext
                        .rope()
                        .byte_slice(old_len_bytes..new_len_bytes),
                );
                let input = Input::new(cursor);

                for m in find_iter(&re, input) {
                    state
                        .find_matches
                        .push(old_len_bytes + m.start()..old_len_bytes + m.end());
                }
                for m in &state.find_matches {
                    state.logtext.add_style(m.clone(), 99);
                }
            }
        }

        Ok(true)
    }

    fn run_search(state: &mut LogScroll, ctx: &mut AppContext<'_>) -> Result<(), Error> {
        let text = state.find.text();

        if text.is_empty() {
            ctx.queue_event(LogScrollEvent::Status(0, String::default()));
            state.find.set_invalid(false);
            state.find_matches.clear();
            state.logtext.set_styles(Vec::default());
            state.find_table.clear_offset();
            state.find_table.clear_selection();
            return Ok(());
        }

        match Regex::new(text) {
            Ok(re) => {
                if state.find.invalid() {
                    ctx.queue_event(LogScrollEvent::Status(0, String::default()));
                    state.find.set_invalid(false);
                }

                let cursor = RopeyCursor::new(state.logtext.rope().byte_slice(..));
                let input = Input::new(cursor);
                let mut matches = Vec::new();
                state.find_matches.clear();
                state.find_table.clear_offset();
                state.find_table.clear_selection();
                for m in find_iter(&re, input) {
                    state.find_matches.push(m.range());
                    matches.push((m.range(), 99));
                }
                state.logtext.set_styles(matches);
            }
            Err(err) => {
                ctx.queue_event(LogScrollEvent::Status(0, format!("{:?}", err)));
                state.find.set_invalid(true);
                state.find_matches.clear();
                state.find_table.clear_offset();
                state.find_table.clear_selection();
            }
        }

        Ok(())
    }

    pub fn init(state: &mut LogScroll, ctx: &mut AppContext<'_>) -> Result<(), Error> {
        ctx.focus().first();

        load_file(state, &ctx.g.current)?;

        ctx.add_timer(
            TimerDef::new()
                .repeat_forever()
                .timer(Duration::from_millis(500)),
        );

        state.pollution = ctx.add_timer(
            TimerDef::new()
                .repeat_forever()
                .timer(Duration::from_millis(1000)),
        );

        Ok(())
    }

    pub fn event(
        event: &LogScrollEvent,
        state: &mut LogScroll,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<LogScrollEvent>, Error> {
        let r = match event {
            LogScrollEvent::Event(event) => {
                try_flow!(match event {
                    ct_event!(keycode press F(2)) => {
                        if !state.split.is_focused() {
                            ctx.focus().focus(&state.split);
                        } else {
                            ctx.focus().next();
                        }
                        Control::Changed
                    }
                    ct_event!(key press ALT-'f') | ct_event!(keycode press F(3)) => {
                        if state.split.is_hidden(1) {
                            state.split.show_split(1);
                            ctx.focus().focus(&state.find);
                        } else {
                            state.split.hide_split(1);
                            ctx.focus().focus(&state.logtext);
                        }
                        Control::Changed
                    }
                    _ => Control::Continue,
                });

                try_flow!(state.split.handle(event, Regular));
                try_flow!(state.logtext.handle(event, ReadOnly));
                try_flow!(state.find_label.handle(event, ctx.focus()));
                try_flow!(match state.find.handle(event, Regular) {
                    TextOutcome::TextChanged => {
                        run_search(state, ctx)?;
                        Control::Changed
                    }
                    r => r.into(),
                });
                try_flow!(match state.find_table.handle(event, Regular) {
                    TableOutcome::Selected => {
                        if let Some(selected) = state.find_table.selected() {
                            let range = state.find_matches[selected].clone();
                            let text_range = state.logtext.byte_range(range.clone());
                            state.logtext.set_cursor(text_range.start, false);

                            let old_style = state.logtext.styles().find(|(_, s)| *s == 100);
                            if let Some((range, style)) = old_style {
                                state.logtext.remove_style(range, style);
                            }
                            state.logtext.add_style(range, 100);
                        }
                        Control::Changed
                    }
                    r => r.into(),
                });

                Control::Continue
            }
            LogScrollEvent::TimeOut(t) if t.handle == state.pollution => {
                debug!("log-pollution {:?}", t);
                Control::Continue
            }
            LogScrollEvent::TimeOut(t) if t.handle != state.pollution => {
                if log_grows(state, &ctx.g.current)? {
                    ctx.queue(Control::Event(LogScrollEvent::Status(
                        0,
                        "grows".to_string(),
                    )));
                    if update_file(state, &ctx.g.current)? {
                        let file_size = format!("{}l", state.logtext.len_lines());
                        ctx.queue(Control::Event(LogScrollEvent::Status(1, file_size)));
                        Control::Changed
                    } else {
                        Control::Continue
                    }
                } else if log_shrinks(state, &ctx.g.current)? {
                    ctx.queue(Control::Event(LogScrollEvent::Status(
                        0,
                        "shrinks".to_string(),
                    )));
                    load_file(state, &ctx.g.current)?;
                    run_search(state, ctx)?;
                    let file_size = format!("{}l", state.logtext.len_lines());
                    ctx.queue(Control::Event(LogScrollEvent::Status(1, file_size)));
                    Control::Changed
                } else {
                    Control::Continue
                }
            }
            _ => Control::Continue,
        };

        Ok(r)
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
