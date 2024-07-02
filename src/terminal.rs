//!
//! Defines the trait RenderUI to hide the different rendering backends.
//!

use crate::{AppContext, AppWidget, RenderContext};
use crossterm::cursor::{DisableBlinking, EnableBlinking, SetCursorStyle};
use crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use std::fmt::Debug;
use std::io;
use std::io::{stdout, Stdout};

/// Encapsulates Terminal and Backend.
///
/// This is used as dyn Trait to hide the Background type parameter.
///
/// If you want to send other than the default Commands to the backend,
/// implement this trait.
pub trait Terminal<App, Global, Action, Error>
where
    App: AppWidget<Global, Action, Error>,
    Action: 'static + Send + Debug,
    Error: 'static + Send + Debug,
{
    /// Terminal init.
    fn init(&mut self) -> Result<(), Error>
    where
        Error: From<io::Error>;

    /// Terminal shutdown.
    fn shutdown(&mut self) -> Result<(), Error>
    where
        Error: From<io::Error>;

    /// Render the app widget.
    ///
    /// Creates the render-context, fetches the frame and calls render.
    #[allow(clippy::needless_lifetimes)]
    fn render<'a, 'b>(
        &mut self,
        app: &mut App,
        state: &mut App::State,
        ctx: &'b mut AppContext<'a, Global, Action, Error>,
    ) -> Result<(), Error>
    where
        Error: From<io::Error>;
}

/// Default RenderUI for crossterm.
#[derive(Debug)]
pub struct CrosstermTerminal {
    term: ratatui::Terminal<CrosstermBackend<Stdout>>,
}

impl CrosstermTerminal {
    pub fn new() -> Result<Self, io::Error> {
        Ok(Self {
            term: ratatui::Terminal::new(CrosstermBackend::new(stdout()))?,
        })
    }
}

impl<App, Global, Action, Error> Terminal<App, Global, Action, Error> for CrosstermTerminal
where
    App: AppWidget<Global, Action, Error>,
    Action: 'static + Send + Debug,
    Error: 'static + Send + Debug,
{
    fn init(&mut self) -> Result<(), Error>
    where
        Error: From<io::Error>,
    {
        stdout().execute(EnterAlternateScreen)?;
        stdout().execute(EnableMouseCapture)?;
        stdout().execute(EnableBracketedPaste)?;
        stdout().execute(EnableBlinking)?;
        stdout().execute(SetCursorStyle::BlinkingBar)?;
        enable_raw_mode()?;

        self.term.clear()?;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), Error>
    where
        Error: From<io::Error>,
    {
        disable_raw_mode()?;
        stdout().execute(SetCursorStyle::DefaultUserShape)?;
        stdout().execute(DisableBlinking)?;
        stdout().execute(DisableBracketedPaste)?;
        stdout().execute(DisableMouseCapture)?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }

    #[allow(clippy::needless_lifetimes)]
    fn render<'a, 'b>(
        &mut self,
        app: &mut App,
        state: &mut App::State,
        ctx: &'b mut AppContext<'a, Global, Action, Error>,
    ) -> Result<(), Error>
    where
        App: AppWidget<Global, Action, Error>,
        Error: From<io::Error>,
    {
        let mut res = Ok(());

        _ = self.term.hide_cursor();

        self.term.draw(|frame| {
            let mut ctx = RenderContext {
                g: ctx.g,
                timeout: ctx.timeout,
                counter: frame.count(),
                area: frame.size(),
                cursor: None,
            };

            let frame_area = frame.size();
            res = app.render(frame_area, frame.buffer_mut(), state, &mut ctx);

            if let Some((cursor_x, cursor_y)) = ctx.cursor {
                frame.set_cursor(cursor_x, cursor_y);
            }
        })?;

        res
    }
}
