#![allow(dead_code)]

use anyhow::{Error, anyhow};
use configparser::ini::Ini;
use dirs::config_dir;
use log::{debug, warn};
use rat_salsa::poll::{PollCrossterm, PollTasks, PollTimers};
use rat_salsa::timer::TimeOut;
use rat_salsa::{Control, RunConfig, SalsaAppContext, SalsaContext, run_tui};
use rat_theme4::theme::SalsaTheme;
use rat_theme4::{StyleName, WidgetStyle, create_theme};
use rat_widget::event::{ConsumedEvent, Dialog, HandleEvent, Regular, ct_event};
use rat_widget::focus::FocusBuilder;
use rat_widget::layout::layout_middle;
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use rat_widget::statusline::{StatusLine, StatusLineState};
use rat_widget::text::HasScreenCursor;
use rat_widget::text::clipboard::{Clipboard, ClipboardError, set_global_clipboard};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::StatefulWidget;
use ropey::Rope;
use std::cell::RefCell;
use std::env::args;
use std::fs;
use std::fs::create_dir_all;
use std::ops::Range;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

fn main() -> Result<(), Error> {
    let mut args = args();
    args.next();
    let mut test = false;
    let mut current_file = None;
    for v in args {
        match v.as_str() {
            "--test" => test = true,
            "--help" => {
                eprintln!("usage: logscroll [--test] <file>");
                return Ok(());
            }
            s => current_file = Some(s.to_string()),
        }
    }
    let Some(current_file) = current_file else {
        eprintln!("usage: logscroll [--test] <file>");
        return Ok(());
    };
    let current = PathBuf::from(current_file);
    if !current.exists() {
        eprintln!("file {:?} does not exist.", current);
        return Ok(());
    }

    setup_logging(test)?;
    set_global_clipboard(CliClipboard::default());

    let config = load_config()?;
    let theme = config_theme(&config);

    let mut global = GlobalState::new(config, theme);
    global.file_path = current;
    global.test = test;

    let mut state = Scenery::default();

    run_tui(
        init,
        render,
        event,
        error,
        &mut global,
        &mut state,
        RunConfig::default()?
            .poll(PollCrossterm)
            .poll(PollTasks::new(2))
            .poll(PollTimers::new()),
    )?;

    Ok(())
}

#[derive(Debug, Default)]
pub struct LogScrollConfig {
    split_pos: u16,
    theme: String,
}

fn config_theme(config: &LogScrollConfig) -> SalsaTheme {
    create_theme(&config.theme)
}

fn load_config() -> Result<LogScrollConfig, Error> {
    if let Some(config) = config_dir() {
        let config = config.join("logscroll").join("logscroll.ini");
        if config.exists() {
            let mut ini = Ini::new();
            match ini.load(config) {
                Ok(_) => {}
                Err(e) => {
                    return Err(anyhow!(e));
                }
            };

            let split_pos: u16 = ini.get("default", "split").unwrap_or("25".into()).parse()?;
            let theme = ini.get("default", "theme").unwrap_or_default();

            return Ok(LogScrollConfig {
                split_pos,
                theme: theme.into(),
            });
        }
    };

    Ok(LogScrollConfig {
        split_pos: 25,
        theme: Default::default(),
    })
}

fn store_config(cfg: &LogScrollConfig) -> Result<(), Error> {
    if let Some(config_dir) = config_dir() {
        let config_path = config_dir.join("logscroll");
        create_dir_all(&config_path)?;
        let config = config_path.join("logscroll.ini");

        let mut ini = if config.exists() {
            let mut ini = Ini::new();
            match ini.load(&config) {
                Ok(_) => {}
                Err(e) => {
                    return Err(anyhow!(e));
                }
            };
            ini
        } else {
            Ini::default()
        };

        ini.set("default", "split", Some(cfg.split_pos.to_string()));
        ini.set("default", "theme", Some(cfg.theme.clone()));

        ini.write(config)?;

        Ok(())
    } else {
        Err(anyhow!("Can't save config."))
    }
}

pub struct GlobalState {
    pub ctx: SalsaAppContext<LogScrollEvent, Error>,
    pub cfg: LogScrollConfig,
    pub theme: SalsaTheme,
    pub file_path: PathBuf,
    pub test: bool,
}

impl SalsaContext<LogScrollEvent, Error> for GlobalState {
    fn set_salsa_ctx(&mut self, app_ctx: SalsaAppContext<LogScrollEvent, Error>) {
        self.ctx = app_ctx;
    }

    #[inline]
    fn salsa_ctx(&self) -> &SalsaAppContext<LogScrollEvent, Error> {
        &self.ctx
    }
}

impl GlobalState {
    pub fn new(cfg: LogScrollConfig, theme: SalsaTheme) -> Self {
        Self {
            ctx: Default::default(),
            cfg,
            theme,
            file_path: Default::default(),
            test: false,
        }
    }
}

#[derive(Debug)]
pub enum LogScrollEvent {
    Event(crossterm::event::Event),
    TimeOut(TimeOut),
    Message(String),
    Status(usize, String),

    Load(Rope),
    Append(String),
    Found(usize, usize, Vec<Range<usize>>),

    Cursor,
    StoreCfg,
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

#[derive(Debug, Default)]
pub struct Scenery {
    pub logscroll: logscroll::LogScroll,
    pub status: StatusLineState,
    pub error_dlg: MsgDialogState,
}

// scenery rendering.
pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut Scenery,
    ctx: &mut GlobalState,
) -> Result<(), Error> {
    let t0 = SystemTime::now();

    let layout = Layout::vertical([
        Constraint::Fill(1), //
        Constraint::Length(1),
    ])
    .split(area);

    logscroll::render(area, buf, &mut state.logscroll, ctx)?;
    ctx.set_screen_cursor(state.logscroll.screen_cursor());

    if state.error_dlg.active() {
        let err = MsgDialog::new().styles(ctx.theme.style(WidgetStyle::MSG_DIALOG));
        let err_area = layout_middle(
            area,
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(2),
            Constraint::Length(2),
        );
        err.render(err_area, buf, &mut state.error_dlg);
    }

    let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
    state
        .status
        .status(2, format!("R{} {:.0?}", ctx.count(), el).to_string());

    let status = StatusLine::new()
        .layout([
            Constraint::Fill(1),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(12),
        ])
        .styles(vec![
            ctx.theme.style::<Style>(Style::STATUS_BASE),
            ctx.theme.style::<Style>(Style::STATUS_BASE),
            ctx.theme.style::<Style>(Style::STATUS_BASE),
            ctx.theme.style::<Style>(Style::STATUS_BASE),
        ]);
    status.render(layout[1], buf, &mut state.status);

    Ok(())
}

pub fn init(state: &mut Scenery, ctx: &mut GlobalState) -> Result<(), Error> {
    ctx.set_focus(FocusBuilder::build_for(&state.logscroll));
    ctx.focus().enable_log();

    logscroll::init(&mut state.logscroll, ctx)?;

    Ok(())
}

pub fn event(
    event: &LogScrollEvent,
    state: &mut Scenery,
    ctx: &mut GlobalState,
) -> Result<Control<LogScrollEvent>, Error> {
    let t0 = SystemTime::now();

    let mut r = match event {
        LogScrollEvent::Event(event) => {
            ctx.set_focus(FocusBuilder::rebuild_for(
                &state.logscroll,
                ctx.take_focus(),
            ));
            ctx.focus().enable_log();

            let mut r = match &event {
                ct_event!(resized) => Control::Changed,
                ct_event!(key press CONTROL-'q') => {
                    state.logscroll.t_cancel.cancel();
                    Control::Quit
                }
                ct_event!(keycode press F(1)) => {
                    Control::Event(LogScrollEvent::Message(HELP_TEXT.to_string()))
                }
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
        LogScrollEvent::StoreCfg => {
            store_config(&ctx.cfg)?;
            Control::Continue
        }
        LogScrollEvent::Cursor => {
            let cursor = format!(
                "{}:{}",
                state.logscroll.logtext.cursor().x,
                state.logscroll.logtext.cursor().y
            );
            state.status.status(1, cursor);
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
    state: &mut Scenery,
    _ctx: &mut GlobalState,
) -> Result<Control<LogScrollEvent>, Error> {
    debug!("ERROR {:#?}", event);
    state.error_dlg.append(format!("{:?}", &*event).as_str());
    Ok(Control::Changed)
}

// main application code
mod logscroll {
    use crate::{GlobalState, LogScrollEvent};
    use anyhow::Error;
    use crossbeam::channel::Sender;
    use log::debug;
    use rat_focus::{FocusBuilder, FocusFlag};
    use rat_salsa::tasks::{Cancel, Liveness};
    use rat_salsa::timer::{TimerDef, TimerHandle};
    use rat_salsa::{Control, SalsaContext};
    use rat_theme4::theme::SalsaTheme;
    use rat_theme4::{StyleName, WidgetStyle, create_theme, salsa_themes};
    use rat_widget::event::{
        HandleEvent, Outcome, ReadOnly, Regular, TableOutcome, TextOutcome, ct_event, try_flow,
    };
    use rat_widget::focus::{HasFocus, Navigation};
    use rat_widget::paired::{PairSplit, Paired, PairedState, PairedWidget};
    use rat_widget::scrolled::Scroll;
    use rat_widget::splitter::{Split, SplitState, SplitType};
    use rat_widget::table::selection::RowSelection;
    use rat_widget::table::{Table, TableContext, TableData, TableState};
    use rat_widget::text::{TextPosition, impl_screen_cursor, upos_type};
    use rat_widget::text_input::{TextInput, TextInputState};
    use rat_widget::textarea::{TextArea, TextAreaState, TextWrap};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Layout, Rect};
    use ratatui::style::Style;
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{Block, BorderType, StatefulWidget, Widget};
    use regex_cursor::engines::dfa::{Regex, find_iter};
    use regex_cursor::{Input, RopeyCursor};
    use ropey::{Rope, RopeBuilder};
    use std::cmp::min;
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};
    use std::ops::Range;
    use std::path::Path;
    use std::thread::sleep;
    use std::time::Duration;
    use unicode_segmentation::UnicodeSegmentation;

    #[derive(Debug)]
    pub struct LogScroll {
        pub split: SplitState,
        pub start_line: upos_type,
        pub logtext: TextAreaState,
        pub find: TextInputState,

        pub logtext_id: usize,
        pub find_matches: Vec<(usize, usize, usize)>,
        pub select_match: (usize, usize, usize),
        pub find_table: TableState<RowSelection>,

        pub t_cancel: Cancel,
        pub t_live: Liveness,

        pub pollution: TimerHandle,
    }

    impl Default for LogScroll {
        fn default() -> Self {
            let mut zelf = Self {
                split: Default::default(),
                start_line: 0,
                logtext: Default::default(),
                find: Default::default(),
                logtext_id: 0,
                find_matches: Default::default(),
                select_match: (0, 0, 0),
                find_table: Default::default(),
                t_cancel: Default::default(),
                t_live: Default::default(),
                pollution: Default::default(),
            };
            zelf.find_table.set_scroll_selection(true);
            zelf
        }
    }

    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        state: &mut LogScroll,
        ctx: &mut GlobalState,
    ) -> Result<(), Error> {
        let l0 = Layout::vertical([
            Constraint::Fill(1), //
            Constraint::Length(1),
        ])
        .split(area);

        let h_scroll = Scroll::horizontal()
            .begin_symbol(Some("◀"))
            .end_symbol(Some("▶"))
            .track_symbol(Some("─"))
            .thumb_symbol("▄")
            .min_symbol(Some("─"));
        let v_scroll = Scroll::vertical()
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some("│"))
            .thumb_symbol("█")
            .min_symbol(Some("│"));

        let (split_layout, split) = Split::vertical()
            .styles(ctx.theme.style(WidgetStyle::SPLIT))
            .split_type(SplitType::Scroll)
            .mark_offset(1)
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1), //
                Constraint::Length(ctx.cfg.split_pos),
            ])
            .into_widgets();
        split_layout.render(l0[0], buf, &mut state.split);

        TextArea::new()
            .vscroll(v_scroll.clone().start_margin(2))
            .hscroll(h_scroll)
            .block(
                Block::bordered().border_type(BorderType::Rounded).title(
                    Line::from_iter([
                        Span::from("log/scroll "),
                        Span::from(format!("{:?}", ctx.file_path)),
                    ])
                    .style(ctx.theme.p.red(7)),
                ),
            )
            .styles(ctx.theme.style(WidgetStyle::TEXTVIEW))
            .text_style_idx(0, ctx.theme.p.orange(6))
            .text_style_idx(1, ctx.theme.p.yellow(6))
            .text_style_idx(2, ctx.theme.p.green(6))
            .text_style_idx(3, ctx.theme.p.bluegreen(6))
            .text_style_idx(4, ctx.theme.p.cyan(6))
            .text_style_idx(5, ctx.theme.p.blue(6))
            .text_style_idx(6, ctx.theme.p.deepblue(6))
            .text_style_idx(7, ctx.theme.p.purple(6))
            .text_style_idx(8, ctx.theme.p.magenta(6))
            .text_style_idx(9, ctx.theme.p.redpink(6))
            .text_style_idx(99, ctx.theme.p.secondary(2))
            .text_style_idx(101, ctx.theme.p.limegreen(2))
            .render(state.split.widget_areas[0], buf, &mut state.logtext);

        // right side
        buf.set_style(
            state.split.widget_areas[1],
            ctx.theme.style::<Style>(Style::CONTAINER_BASE),
        );

        let l2 = Layout::vertical([
            Constraint::Length(1), //
            Constraint::Length(1), //
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .horizontal_margin(1)
        .split(state.split.widget_areas[1]);

        let find_area = Rect::new(l2[1].x, l2[1].y, l2[1].width.saturating_sub(1), 1);
        Paired::new(
            PairedWidget::new(Span::from("Find")),
            TextInput::new().styles(ctx.theme.style(WidgetStyle::TEXT)),
        )
        .split(PairSplit::Fix1(5))
        .render(
            find_area,
            buf,
            &mut PairedState::new(&mut (), &mut state.find),
        );

        Table::new()
            .vscroll(v_scroll)
            .styles(ctx.theme.style(WidgetStyle::TABLE))
            .data(FindData {
                theme: &ctx.theme,
                text: &state.logtext,
                start_line: state.start_line,
                data: &state.find_matches,
            })
            .render(l2[2], buf, &mut state.find_table);

        let matches_area = Rect::new(l2[3].x, l2[3].y, l2[3].width.saturating_sub(1), 1);
        Line::from(format!("{} matches", state.find_matches.len()))
            .style(ctx.theme.p.red(7))
            .render(matches_area, buf);

        split.render(l0[0], buf, &mut state.split);

        Ok(())
    }

    struct FindData<'a> {
        theme: &'a SalsaTheme,
        text: &'a TextAreaState,
        start_line: upos_type,
        data: &'a [(usize, usize, usize)],
    }

    impl<'a> FindData<'a> {
        fn color(&self, n: usize) -> Style {
            match n {
                0 => self.theme.p.orange(6),
                1 => self.theme.p.yellow(6),
                2 => self.theme.p.green(6),
                3 => self.theme.p.bluegreen(6),
                4 => self.theme.p.cyan(6),
                5 => self.theme.p.blue(6),
                6 => self.theme.p.deepblue(6),
                7 => self.theme.p.purple(6),
                8 => self.theme.p.magenta(6),
                _ => self.theme.p.redpink(6),
            }
        }
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

            let line = self.text.byte_pos(data.0);

            let line_byte = self.text.byte_at(TextPosition::new(0, line.y));
            let data_start = data.0 - line_byte.start;
            let data_end = data.1 - line_byte.start;

            let line_text = self.text.line_at(line.y);
            let line_prefix = &line_text[..data_start];
            let line_match = &line_text[data_start..data_end];
            let line_suffix = &line_text[data_end..];

            let mut prefix_len = line_prefix.graphemes(true).count();
            let match_len = line_match.graphemes(true).count();
            let mut suffix_len = line_suffix.graphemes(true).count();
            let mut area_width = area.width as usize;

            if prefix_len + match_len + suffix_len > area_width {
                area_width = area_width.saturating_sub(match_len);

                if suffix_len > area_width / 2 && prefix_len > area_width / 2 {
                    prefix_len = area_width / 2;
                    suffix_len = area_width / 2;
                } else if prefix_len > area_width / 2 {
                    prefix_len = area_width - suffix_len;
                } else if suffix_len > area_width / 2 {
                    suffix_len = area_width - prefix_len;
                } else {
                    // nice
                }
            }

            let line_prefix = line_prefix
                .graphemes(true)
                .rev()
                .take(prefix_len)
                .collect::<String>();
            let line_prefix = line_prefix.graphemes(true).rev().collect::<String>();

            let line_suffix = line_suffix
                .graphemes(true)
                .take(suffix_len)
                .collect::<String>();

            let pos_txt = format!("\u{2316} {}:{}", line.x, self.start_line + line.y);

            let l0 = Line::from_iter([
                Span::from(pos_txt), //
            ]);
            let l1 = Line::from_iter([
                Span::from(line_prefix),
                Span::from(line_match).style(self.color(data.2)),
                Span::from(line_suffix),
            ])
            .style(Style::reset());

            l0.render(Rect::new(area.x, area.y, area.width, 1), buf);
            l1.render(Rect::new(area.x, area.y + 1, area.width, 1), buf);
        }
    }

    impl HasFocus for LogScroll {
        fn build(&self, builder: &mut FocusBuilder) {
            builder.widget_navigate(&self.logtext, Navigation::Regular);
            builder.widget(&self.split);
            builder.widget(&self.find);
            builder.widget(&self.find_table);
        }

        fn focus(&self) -> FocusFlag {
            unimplemented!("not defined")
        }

        fn area(&self) -> Rect {
            unimplemented!("not defined")
        }
    }

    impl_screen_cursor!(logtext, find for LogScroll);

    fn loop_file(
        path: &Path,
        can: Cancel,
        queue: &Sender<Result<Control<LogScrollEvent>, Error>>,
    ) -> Result<Control<LogScrollEvent>, Error> {
        // startup load
        let mut buf_len = load_file(path, &can, queue)?;

        if can.is_canceled() {
            return Ok(Control::Continue);
        }

        // loop and check
        loop {
            update_file(path, &can, queue, &mut buf_len)?;
            if can.is_canceled() {
                break;
            }
            sleep(Duration::from_millis(500));
        }

        Ok(Control::Continue)
    }

    fn update_file(
        path: &Path,
        can: &Cancel,
        queue: &Sender<Result<Control<LogScrollEvent>, Error>>,
        buf_len: &mut u64,
    ) -> Result<(), Error> {
        let new_len = path.metadata()?.len();
        if new_len < *buf_len {
            *buf_len = load_file(path, &can, queue)?;
        } else if new_len > *buf_len {
            let mut f = File::open(path)?;
            f.seek(SeekFrom::Start(*buf_len))?;

            let mut buf = String::new();
            *buf_len += f.read_to_string(&mut buf)? as u64;

            queue.send(Ok(Control::Event(LogScrollEvent::Append(buf))))?;
        }
        Ok(())
    }

    fn load_file(
        path: &Path,
        can: &Cancel,
        queue: &Sender<Result<Control<LogScrollEvent>, Error>>,
    ) -> Result<u64, Error> {
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

            if can.is_canceled() {
                return Ok(0);
            }
        }
        let txt = txt.finish();

        let buf_len = txt.len_bytes();

        queue.send(Ok(Control::Event(LogScrollEvent::Load(txt))))?;

        Ok(buf_len as u64)
    }

    fn append_log(state: &mut LogScroll, txt: &str) -> Result<Range<usize>, Error> {
        let old_len_bytes = state.logtext.rope().len_bytes();
        let cursor_at_end = state.logtext.cursor().y == state.logtext.len_lines()
            || state.logtext.cursor().y == state.logtext.len_lines() - 1;

        let pos = TextPosition::new(0, state.logtext.len_lines());
        state.logtext.value.insert_str(pos, txt).expect("fine");

        if cursor_at_end {
            state
                .logtext
                .value
                .set_cursor(TextPosition::new(0, state.logtext.len_lines()), false);
            state.logtext.set_vertical_offset(
                state
                    .logtext
                    .len_lines()
                    .saturating_sub(state.logtext.vertical_page()),
            );
        }

        let new_len_bytes = state.logtext.rope().len_bytes();

        Ok(old_len_bytes..new_len_bytes)
    }

    fn scan_regex_file(
        log_id: usize,
        id: usize,
        find: String,
        rope: Rope,
        slice: Range<usize>,
        can: Cancel,
        queue: &Sender<Result<Control<LogScrollEvent>, Error>>,
    ) -> Result<Control<LogScrollEvent>, Error> {
        // update find
        if !find.is_empty() {
            if let Ok(re) = Regex::new(&find) {
                let cursor = RopeyCursor::new(rope.byte_slice(slice.clone()));
                let input = Input::new(cursor);

                let mut find_matches = Vec::new();
                for m in find_iter(&re, input) {
                    find_matches.push(slice.start + m.start()..slice.start + m.end());

                    if find_matches.len() >= 10000 {
                        queue.send(Ok(Control::Event(LogScrollEvent::Found(
                            log_id,
                            id,
                            find_matches,
                        ))))?;
                        find_matches = Vec::new();
                    }

                    if can.is_canceled() {
                        break;
                    }
                }

                Ok(Control::Event(LogScrollEvent::Found(
                    log_id,
                    id,
                    find_matches,
                )))
            } else {
                Ok(Control::Continue)
            }
        } else {
            Ok(Control::Continue)
        }
    }

    fn do_search(state: &mut LogScroll, slice: Range<usize>, ctx: &mut GlobalState) {
        let text = state.find.text();
        let log_id = state.logtext_id;
        for (id, regex) in text.split('|').enumerate() {
            // TODO: |
            if !regex.is_empty() {
                let id = min(id, 9);
                let rope = state.logtext.rope().clone();
                let slice = slice.clone();
                let regex = regex.to_string();
                _ = ctx.spawn_ext(move |can, chan| {
                    scan_regex_file(log_id, id, regex, rope, slice, can, chan)
                });
            }
        }
    }

    pub fn init(state: &mut LogScroll, ctx: &mut GlobalState) -> Result<(), Error> {
        ctx.focus().first();

        let file_path = ctx.file_path.clone();
        (state.t_cancel, state.t_live) = ctx.spawn_ext(move |can, chan| {
            loop_file(&file_path, can, chan) //
        })?;

        if ctx.test {
            // fill the log
            state.pollution = ctx.add_timer(
                TimerDef::new()
                    .repeat_forever()
                    .timer(Duration::from_millis(400)),
            );
        }

        Ok(())
    }

    pub fn event(
        event: &LogScrollEvent,
        state: &mut LogScroll,
        ctx: &mut GlobalState,
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
                        if state.find.is_focused() {
                            if state.split.is_hidden(1) {
                                state.split.show_split(1);
                                ctx.focus().focus(&state.find);
                            } else {
                                state.split.hide_split(1);
                                ctx.focus().focus(&state.logtext);
                            }
                        } else {
                            state.split.show_split(1);
                            ctx.focus().focus(&state.find);
                        }
                        Control::Changed
                    }
                    ct_event!(key press ALT-'w') => {
                        let wrap = match state.logtext.text_wrap() {
                            TextWrap::Shift => TextWrap::Hard,
                            TextWrap::Hard => TextWrap::Word(8),
                            TextWrap::Word(_) => TextWrap::Shift,
                            _ => TextWrap::Shift,
                        };
                        state.logtext.set_text_wrap(wrap);
                        Control::Changed
                    }
                    ct_event!(keycode press F(4)) => {
                        state
                            .logtext
                            .set_cursor((0, state.logtext.len_lines()), false);
                        Control::Changed
                    }
                    ct_event!(keycode press F(5)) => {
                        state.start_line += state.logtext.len_lines().saturating_sub(1);
                        state.logtext.clear();
                        state.select_match = (0, 0, 0);
                        state.find_matches.clear();
                        state.find_table.clear_selection();
                        state.find_table.clear_offset();
                        Control::Changed
                    }
                    ct_event!(keycode press F(8)) => {
                        let themes = salsa_themes();
                        let pos = themes
                            .iter()
                            .position(|v| *v == ctx.theme.name())
                            .unwrap_or(0);
                        let pos = (pos + 1) % themes.len();
                        ctx.theme = create_theme(&themes[pos]);
                        ctx.cfg.theme = themes[pos].to_string();
                        ctx.queue_event(LogScrollEvent::StoreCfg);
                        ctx.queue_event(LogScrollEvent::Status(0, ctx.theme.name().into()));
                        Control::Changed
                    }

                    _ => Control::Continue,
                });

                try_flow!(match state.split.handle(event, Regular) {
                    Outcome::Changed => {
                        ctx.cfg.split_pos = state.split.area_len(1);
                        ctx.queue_event(LogScrollEvent::StoreCfg);
                        Control::Changed
                    }
                    r => r.into(),
                });
                try_flow!(match state.logtext.handle(event, ReadOnly) {
                    TextOutcome::Changed | TextOutcome::TextChanged => {
                        Control::Event(LogScrollEvent::Cursor)
                    }
                    r => r.into(),
                });
                try_flow!(match state.find.handle(event, Regular) {
                    TextOutcome::TextChanged => {
                        state.find.set_invalid(false);
                        state.find_matches.clear();
                        state.find_table.clear_offset();
                        state.find_table.select(None);
                        state.logtext_id += 1;
                        state.logtext.set_styles(Vec::default());

                        do_search(state, 0..state.logtext.rope().len_bytes(), ctx);
                        Control::Changed
                    }
                    r => r.into(),
                });
                if state.find.is_focused() {
                    if let ct_event!(keycode press Down) = event {
                        ctx.focus().focus(&state.find_table);
                    }
                }
                try_flow!(match state.find_table.handle(event, Regular) {
                    TableOutcome::Selected => {
                        if let Some(selected) = state.find_table.selected_checked() {
                            let range = state.find_matches[selected].clone();

                            // change highlight
                            state.logtext.remove_style(
                                state.select_match.0..state.select_match.1,
                                state.select_match.2,
                            );
                            state.logtext.add_style(range.0..range.1, 101);
                            state.select_match = (range.0, range.1, 101);

                            // scroll to find
                            let text_range = state.logtext.byte_range(range.0..range.1);
                            state.logtext.set_cursor(text_range.start, false);

                            ctx.queue_event(LogScrollEvent::Cursor);
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
            LogScrollEvent::Load(r) => {
                ctx.queue_event(LogScrollEvent::Status(0, String::default()));

                state.find.set_invalid(false);
                state.find_matches.clear();
                state.find_table.clear_offset();
                state.find_table.select(None);
                state.logtext_id += 1;

                state.start_line = 0;
                state.logtext.set_rope(r.clone());
                state.logtext.set_styles(Vec::default());
                state
                    .logtext
                    .set_cursor((0, state.logtext.len_lines()), false);
                state.logtext.scroll_cursor_to_visible();

                do_search(state, 0..r.len_bytes(), ctx);

                Control::Changed
            }
            LogScrollEvent::Append(s) => {
                ctx.queue_event(LogScrollEvent::Cursor);

                let r = append_log(state, &s)?;
                do_search(state, r, ctx);

                Control::Changed
            }
            LogScrollEvent::Found(log_id, id, found) => {
                if state.logtext_id == *log_id {
                    for r in found {
                        match state.find_matches.binary_search(&(r.start, r.end, *id)) {
                            Ok(i) => {
                                state.find_matches[i] = (r.start, r.end, *id);
                            }
                            Err(i) => {
                                state.find_matches.insert(i, (r.start, r.end, *id));
                            }
                        };
                        state.logtext.add_style(r.clone(), *id);
                    }
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

fn setup_logging(test: bool) -> Result<(), Error> {
    if let Some(cache) = dirs::cache_dir() {
        let log_path = if test {
            PathBuf::from(".")
        } else {
            let log_path = cache.join("rat-salsa");
            if !log_path.exists() {
                fs::create_dir_all(&log_path)?;
            }
            log_path
        };

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

#[derive(Debug, Default, Clone)]
struct CliClipboard {
    clip: RefCell<String>,
}

impl Clipboard for CliClipboard {
    fn get_string(&self) -> Result<String, ClipboardError> {
        match cli_clipboard::get_contents() {
            Ok(v) => Ok(v),
            Err(e) => {
                warn!("{:?}", e);
                Ok(self.clip.borrow().clone())
            }
        }
    }

    fn set_string(&self, s: &str) -> Result<(), ClipboardError> {
        let mut clip = self.clip.borrow_mut();
        *clip = s.to_string();

        match cli_clipboard::set_contents(s.to_string()) {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("{:?}", e);
                Err(ClipboardError)
            }
        }
    }
}

static HELP_TEXT: &str = r#"

### HELP ###

F1      show this
F2      change split position
F3      jump to find / toggle find
F4      jump to end of log
F5      truncate log
F8      next theme
Ctrl+Q  quit

### Navigation ###

Tab/
Shift-Tab   standard navigation

### Find ###

Use '|' to separate search terms.

### Log ###
Ctrl+End    jump to end of log and stick there
...         standard navigation with arrow-keys etc
Ctrl+C      copy to clipboard
Alt+W       toggle text-wrap

"#;
