#![doc = include_str!("../readme.md")]

use rat_salsa::{AppContext, AppState, AppWidget, Control, RenderContext};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::any::{Any, TypeId};
use std::cell::Cell;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

pub mod widgets;

/// DialogStack.
///
/// Call render() for this as the last action when rendering your
/// application.
///
#[derive(Debug)]
pub struct DialogStack;

/// State of the dialog stack.
///
/// Add this to your global state.
///

///
/// ** unstable **
pub struct DialogStackState<Global, Event, Error> {
    inner: Rc<Cell<Inner<Global, Event, Error>>>,
}

/// Errors for some operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogStackError {
    /// During event-handling the top dialog is taken from the
    /// stack to avoid problems if this ever recurses.
    /// Some operations are not available during this time.
    InvalidDuringEventHandling,
    /// No dialogs on the stack.
    StackIsEmpty,
    /// Downcasting error.
    TypeMismatch,
}

impl Display for DialogStackError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for DialogStackError {}

/// Behind the scenes.
struct Inner<Global, Event, Error> {
    // don't hold the Widgets just a constructor.
    render: Vec<
        Box<
            dyn Fn(
                    Rect,
                    &mut Buffer,
                    &mut dyn DialogState<Global, Event, Error>,
                    &'_ mut RenderContext<'_, Global>,
                ) -> Result<(), Error>
                + 'static,
        >,
    >,
    // dialog states
    state: Vec<Box<dyn DialogState<Global, Event, Error>>>,

    // top has been detached.
    detached: bool,
    // top will be popped later, if it is currently detached.
    pop_top: bool,
}

/// Trait for a dialogs state.
///
/// A separate trait is necessary because this needs Any in the vtable.
pub trait DialogState<Global, Event, Error>: AppState<Global, Event, Error> + Any
where
    Global: 'static,
    Event: Send + 'static,
    Error: Send + 'static,
{
    /// Is the dialog still active.
    ///
    /// Whenever this goes to false during event handling the dialog
    /// will be popped from the stack.
    fn active(&self) -> bool;
}

impl<Global, Event, Error> dyn DialogState<Global, Event, Error>
where
    Global: 'static,
    Event: Send + 'static,
    Error: Send + 'static,
{
    /// down cast Any style.
    pub fn downcast_ref<R: DialogState<Global, Event, Error>>(&self) -> Option<&R> {
        if self.type_id() == TypeId::of::<R>() {
            let p: *const dyn DialogState<Global, Event, Error> = self;
            Some(unsafe { &*(p as *const R) })
        } else {
            None
        }
    }

    /// down cast Any style.
    pub fn downcast_mut<R: DialogState<Global, Event, Error>>(&'_ mut self) -> Option<&'_ mut R> {
        if (*self).type_id() == TypeId::of::<R>() {
            let p: *mut dyn DialogState<Global, Event, Error> = self;
            Some(unsafe { &mut *(p as *mut R) })
        } else {
            None
        }
    }
}

impl<Global, Event, Error> AppWidget<Global, Event, Error> for DialogStack
where
    Global: 'static,
    Event: Send + 'static,
    Error: Send + 'static,
{
    type State = DialogStackState<Global, Event, Error>;

    fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut Self::State,
        ctx: &mut RenderContext<'_, Global>,
    ) -> Result<(), Error> {
        // render in order. last is top.
        let mut inner = state.inner.replace(Inner::default());

        let r = 'l: {
            // render in order. last is top.
            for (render, state) in inner.render.iter().zip(inner.state.iter_mut()) {
                let r = render(area, buf, state.as_mut(), ctx);
                if r.is_err() {
                    break 'l r;
                }
            }
            Ok(())
        };

        state.inner.set(inner);

        Ok(r?)
    }
}

impl<Global, Event, Error> Debug for Inner<Global, Event, Error>
where
    Event: Send + 'static,
    Error: Send + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inner").field("..dyn..", &()).finish()
    }
}

impl<Global, Event, Error> Default for Inner<Global, Event, Error>
where
    Event: Send + 'static,
    Error: Send + 'static,
{
    fn default() -> Self {
        Self {
            render: Default::default(),
            state: Default::default(),
            detached: Default::default(),
            pop_top: Default::default(),
        }
    }
}

impl<Global, Event, Error> AppState<Global, Event, Error> for DialogStackState<Global, Event, Error>
where
    Global: 'static,
    Event: Send + 'static,
    Error: Send + 'static,
{
    fn init(&mut self, _ctx: &mut AppContext<'_, Global, Event, Error>) -> Result<(), Error> {
        // no special init
        Ok(())
    }

    fn shutdown(&mut self, ctx: &mut AppContext<'_, Global, Event, Error>) -> Result<(), Error> {
        let mut inner = self.inner.replace(Inner::default());
        let r = 'l: {
            for state in inner.state.iter_mut().rev() {
                let r = state.shutdown(ctx);
                if r.is_err() {
                    break 'l r;
                }
            }
            Ok(())
        };
        self.inner.set(inner);
        Ok(r?)
    }

    fn event(
        &mut self,
        event: &Event,
        ctx: &mut AppContext<'_, Global, Event, Error>,
    ) -> Result<Control<Event>, Error> {
        let (idx, dialog, mut state) = {
            let mut inner = self.inner.replace(Inner::default());

            if inner.render.is_empty() {
                self.inner.set(inner);
                return Ok(Control::Continue);
            }

            // only the top dialog gets any events.
            let dialog = inner.render.pop().expect("dialog");
            let state = inner.state.pop().expect("state");
            let idx = inner.render.len();

            inner.detached = true;
            self.inner.set(inner);

            (idx, dialog, state)
        };

        let r = state.event(event, ctx);
        let active = state.active();

        {
            let mut inner = self.inner.replace(Inner::default());
            if inner.pop_top || !active {
                inner.detached = false;
                inner.pop_top = false;
            } else {
                inner.detached = false;
                inner.render.insert(idx, dialog);
                inner.state.insert(idx, state);
            }
            self.inner.set(inner);
        }

        r
    }
}

impl<Global, Event, Error> Debug for DialogStackState<Global, Event, Error>
where
    Event: Send + 'static,
    Error: Send + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DialogStackState").finish()
    }
}

impl<Global, Event, Error> Default for DialogStackState<Global, Event, Error>
where
    Event: Send + 'static,
    Error: Send + 'static,
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<Global, Event, Error> Clone for DialogStackState<Global, Event, Error>
where
    Event: Send + 'static,
    Error: Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<Global, Event, Error> DialogStackState<Global, Event, Error>
where
    Global: 'static,
    Event: Send + 'static,
    Error: Send + 'static,
{
    /// New dialog stack state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a new dialog window on the stack.
    ///
    /// This takes
    /// * a constructor for the AppWidget. This is called for every render
    ///   and can adjust construction to the environment.
    /// * the dialogs state.
    ///
    /// __Note__
    ///
    /// This can be called during event handling of a dialog.
    pub fn push_dialog<Render, State>(&mut self, render: Render, state: State)
    where
        Render: Fn(
                Rect,
                &mut Buffer,
                &mut dyn DialogState<Global, Event, Error>,
                &'_ mut RenderContext<'_, Global>,
            ) -> Result<(), Error>
            + 'static,
        State: DialogState<Global, Event, Error> + 'static,
    {
        let mut inner = self.inner.replace(Inner::default());

        inner.render.push(Box::new(render));
        inner.state.push(Box::new(state));

        self.inner.set(inner);
    }

    /// Pop the top dialog from the stack.
    ///
    /// This can be called repeatedly if necessary.
    ///
    /// __Note__
    ///
    /// This can be called during event handling of a dialog.
    pub fn pop_dialog(&mut self) {
        let mut inner = self.inner.replace(Inner::default());

        if inner.detached && !inner.pop_top {
            inner.pop_top = true;

            self.inner.set(inner);
        } else {
            _ = inner.render.pop().expect("render");
            _ = inner.state.pop().expect("state");

            self.inner.set(inner);
        }
    }

    /// Is the dialog stack empty?
    ///
    /// __Note__
    ///
    /// This can be called during event handling of a dialog.
    pub fn is_empty(&self) -> bool {
        let inner = self.inner.replace(Inner::default());

        let r = inner.state.is_empty() && !inner.detached;

        self.inner.set(inner);

        r
    }

    /// Test the type of the top dialog state.
    ///
    /// __Note__
    ///
    /// This will not work during event-handling of a dialog.
    pub fn top_state_is<S: 'static>(&self) -> Result<bool, DialogStackError> {
        let inner = self.inner.replace(Inner::default());

        if inner.detached {
            self.inner.set(inner);
            return Err(DialogStackError::InvalidDuringEventHandling);
        }
        if inner.state.is_empty() {
            self.inner.set(inner);
            return Err(DialogStackError::StackIsEmpty);
        }

        let state = inner.state.last().expect("state");
        let dyn_transformed = &*state.deref();
        let r = dyn_transformed.type_id() == TypeId::of::<S>();

        self.inner.set(inner);

        Ok(r)
    }

    /// Calls the closure with the top state of the stack if the type matches.
    ///
    /// __Note__
    ///
    /// This will not work during event-handling of a dialog.
    pub fn map_top_state_if<S, MAP, R>(&self, map: MAP) -> Result<R, DialogStackError>
    where
        MAP: FnOnce(&mut S) -> R,
        S: DialogState<Global, Event, Error>,
    {
        if !self.top_state_is::<S>()? {
            return Err(DialogStackError::TypeMismatch);
        }

        let mut inner = self.inner.replace(Inner::default());

        let dialog = inner.render.pop().expect("render");
        let mut state = inner.state.pop().expect("state");

        let dyn_state = &mut *state.deref_mut();
        let state_t = dyn_state.downcast_mut::<S>().expect("state");
        let r = map(state_t);

        inner.render.push(dialog);
        inner.state.push(state);

        self.inner.set(inner);

        Ok(r)
    }
}
