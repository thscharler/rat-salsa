use anyhow::Error;
use log::debug;
use rat_event::{HandleEvent, Regular, ct_event, try_flow};
use rat_focus::{FocusBuilder, HasFocus, impl_has_focus};
use rat_salsa::event::RenderedEvent;
use rat_salsa::poll::{PollCrossterm, PollRendered};
use rat_salsa::{Control, RunConfig, SalsaAppContext, SalsaContext, run_tui};
use rat_theme4::theme::SalsaTheme;
use rat_theme4::{StyleName, WidgetStyle, create_theme};
use rat_widget::button::{Button, ButtonState};
use rat_widget::event::ButtonOutcome;
use rat_widget::layout::simple_grid;
use rat_widget::list::selection::RowSelection;
use rat_widget::list::{List, ListState};
use rat_widget::reloc::RelocatableState;
use rat_widget::scrolled::Scroll;
use rat_widget::text::HasScreenCursor;
use rat_widget::text_input::{TextInput, TextInputState};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::style::Style;
use ratatui_core::text::Text;
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_crossterm::crossterm::event::Event as CrosstermEvent;
use ratatui_widgets::block::Block;
use ratatui_widgets::borders::BorderType;
use ratatui_widgets::list::ListItem;

fn main() -> Result<(), Error> {
    setup_logging()?;

    let theme = create_theme("Imperial Dark");
    let mut global = Global::new(theme);
    let mut state = Todos::default();

    run_tui(
        init,
        render,
        event,
        error,
        &mut global,
        &mut state,
        RunConfig::default()? //
            .poll(PollCrossterm)
            .poll(PollRendered),
    )?;

    Ok(())
}

#[derive(Clone, Debug)]
pub struct Todo {
    _id: usize,
    text: String,
}

impl Todo {
    pub fn new(id: usize, text: String) -> Self {
        Self { _id: id, text }
    }
}

/// Globally accessible data/state.
pub struct Global {
    ctx: SalsaAppContext<AppEvent, Error>,
    pub theme: SalsaTheme,
}

impl SalsaContext<AppEvent, Error> for Global {
    fn set_salsa_ctx(&mut self, app_ctx: SalsaAppContext<AppEvent, Error>) {
        self.ctx = app_ctx;
    }

    fn salsa_ctx(&self) -> &SalsaAppContext<AppEvent, Error> {
        &self.ctx
    }
}

impl Global {
    pub fn new(theme: SalsaTheme) -> Self {
        Self {
            ctx: Default::default(),
            theme,
        }
    }
}

#[derive(Debug)]
pub enum AppEvent {
    Event(CrosstermEvent),
    Rendered,
}

impl From<RenderedEvent> for AppEvent {
    fn from(_: RenderedEvent) -> Self {
        Self::Rendered
    }
}

impl From<CrosstermEvent> for AppEvent {
    fn from(value: CrosstermEvent) -> Self {
        Self::Event(value)
    }
}

#[derive(Debug, Default)]
pub struct Todos {
    pub todos: Vec<Todo>,
    pub input: TextInputState,
    pub add: ButtonState,
    pub remove: ButtonState,
    pub list: ListState<RowSelection>,
    pub next_id: usize,
}

impl_has_focus!(input, list for Todos);

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut Todos,
    ctx: &mut Global,
) -> Result<(), Error> {
    let grid = simple_grid(
        area,
        [
            Constraint::Length(5), //
            Constraint::Fill(1),
            Constraint::Length(5),
        ],
        [
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ],
    );

    Text::from("📝 Simple Todo App").render(grid[1][1], buf);

    let l1 = Layout::horizontal([Constraint::Fill(1), Constraint::Length(7)])
        .spacing(1)
        .split(grid[1][3]);

    TextInput::new() //
        .styles(ctx.theme.style(WidgetStyle::TEXT))
        .render(l1[0], buf, &mut state.input);

    Button::new("Add")
        .styles(ctx.theme.style(WidgetStyle::BUTTON))
        .hover_style(ctx.theme.p.limegreen(0))
        .render(l1[1], buf, &mut state.add);

    List::new(state.todos.iter().map(|v| ListItem::new(v.text.as_str())))
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(ctx.theme.style::<Style>(Style::CONTAINER_BORDER_FG)),
        )
        .scroll(Scroll::new())
        .styles(ctx.theme.style(WidgetStyle::LIST))
        .render(grid[1][5], buf, &mut state.list);

    // show inline remove button
    state.remove.relocate_hidden();
    if let Some(row) = state.list.selected() {
        if let Some(area) = state.list.row_area(row) {
            if state.list.is_focused() {
                Button::new("Remove")
                    .styles(ctx.theme.style(WidgetStyle::BUTTON))
                    .hover_style(ctx.theme.p.limegreen(0))
                    .render(
                        Rect::new(area.right().saturating_sub(9), area.y, 8, 1),
                        buf,
                        &mut state.remove,
                    );
            }
        }
    }

    ctx.set_screen_cursor(state.input.screen_cursor());

    Ok(())
}

pub fn init(state: &mut Todos, ctx: &mut Global) -> Result<(), Error> {
    ctx.set_focus(FocusBuilder::build_for(state));
    ctx.focus().first();
    Ok(())
}

pub fn event(
    event: &AppEvent,
    state: &mut Todos,
    ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    match event {
        AppEvent::Event(event) => {
            try_flow!(match &event {
                ct_event!(resized) => Control::Changed,
                ct_event!(key press CONTROL-'q') => Control::Quit,
                _ => Control::Continue,
            });

            ctx.handle_focus(event);

            try_flow!(handle_ui(event, state, ctx)?);

            Ok(Control::Continue)
        }
        AppEvent::Rendered => {
            ctx.set_focus(FocusBuilder::rebuild_for(state, ctx.take_focus()));
            Ok(Control::Continue)
        }
    }
}

fn handle_ui(
    event: &CrosstermEvent,
    state: &mut Todos,
    ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    try_flow!(state.input.handle(event, Regular));

    try_flow!(match state.add.handle(event, Regular) {
        ButtonOutcome::Pressed => {
            add_todo(state, ctx)?
        }
        r => r.into(),
    });
    try_flow!(match state.remove.handle(event, Regular) {
        ButtonOutcome::Pressed => {
            remove_todo(state, ctx)?
        }
        r => r.into(),
    });

    try_flow!(state.list.handle(event, Regular));

    try_flow!(match event {
        ct_event!(keycode press Enter) if state.input.is_focused() => {
            add_todo(state, ctx)?
        }
        ct_event!(keycode press Esc) if state.input.is_focused() => {
            clear_input(state, ctx)?
        }
        ct_event!(keycode press Delete) if state.list.is_focused() => {
            remove_todo(state, ctx)?
        }
        _ => Control::Continue,
    });

    Ok(Control::Continue)
}

fn remove_todo(state: &mut Todos, _ctx: &mut Global) -> Result<Control<AppEvent>, Error> {
    if let Some(row) = state.list.selected() {
        if row < state.todos.len() {
            state.todos.remove(row);
            state.remove.mouse.hover.set(false);

            if row >= state.todos.len() {
                state.list.select(Some(state.todos.len().saturating_sub(1)));
            }
        }
    }
    Ok(Control::Changed)
}

fn clear_input(state: &mut Todos, _ctx: &mut Global) -> Result<Control<AppEvent>, Error> {
    state.input.clear();
    Ok(Control::Changed)
}

fn add_todo(state: &mut Todos, _ctx: &mut Global) -> Result<Control<AppEvent>, Error> {
    let text = state.input.text();
    if !text.trim().is_empty() {
        state.todos.push(Todo {
            _id: state.next_id,
            text: text.trim().to_string(),
        });
        state.list.select(Some(state.todos.len().saturating_sub(1)));
        state.list.scroll_to_selected();
        state.input.clear();
        state.next_id += 1;

        Ok(Control::Changed)
    } else {
        Ok(Control::Continue)
    }
}

pub fn error(
    event: Error,
    _state: &mut Todos,
    _ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    debug!("{:#?}", event);
    Ok(Control::Changed)
}

fn setup_logging() -> Result<(), Error> {
    use std::{fs, path};
    let log_path = path::PathBuf::from(".");
    let log_file = log_path.join("log.log");
    _ = fs::remove_file(&log_file);
    fern::Dispatch::new()
        .format(|out, message, _record| {
            out.finish(format_args!("{}", message)) //
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file(&log_file)?)
        .apply()?;
    Ok(())
}
