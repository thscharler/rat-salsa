//!
//! DialogStack adds support for dialog windows.
//!
//! ** unstable **
//!

use crate::{AppContext, AppState, AppWidget, Control, RenderContext};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::any::{Any, TypeId};
use std::cell::Cell;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

/// DialogStack.
///
/// Call render() for this as the last action when rendering your
/// application.
///
/// TODO: usage
///
#[derive(Debug)]
pub struct DialogStack;

/// State of the dialog stack.
///
/// Add this to your global state.
///
/// TODO: usage
///
/// ** unstable **
pub struct DialogStackState<Global, Event, Error> {
    inner: Rc<Cell<Inner<Global, Event, Error>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogStackError {
    InvalidDuringEventHandling,
    StackIsEmpty,
    TypeMismatch,
}

impl Display for DialogStackError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for DialogStackError {}

struct Inner<Global, Event, Error> {
    dialog: Vec<
        Box<
            dyn StackedDialog<
                Global,
                Event,
                Error,
                State = dyn StackedDialogState<Global, Event, Error>,
            >,
        >,
    >,
    state: Vec<Box<dyn StackedDialogState<Global, Event, Error>>>,

    // top has been detached.
    detached: bool,
    // top will be popped later, if it is currently detached.
    pop_top: bool,
}

/// Trait for a dialog window.
pub trait StackedDialog<Global, Event, Error>
where
    Global: 'static,
    Event: Send + 'static,
    Error: Send + 'static,
    Self: Any + 'static,
    Self: AppWidget<Global, Event, Error>,
    Self::State: StackedDialogState<Global, Event, Error>,
{
}

/// Trait for a dialogs state.
pub trait StackedDialogState<Global, Event, Error>
where
    Global: 'static,
    Event: Send + 'static,
    Error: Send + 'static,
    Self: Any + 'static,
    Self: AppState<Global, Event, Error>,
{
}

impl<Global, Event, Error>
    dyn StackedDialog<Global, Event, Error, State = dyn StackedDialogState<Global, Event, Error>>
where
    Global: 'static,
    Event: Send + 'static,
    Error: Send + 'static,
{
    /// down cast Any style.
    pub fn downcast_ref<
        R: StackedDialog<
                Global,
                Event,
                Error,
                State = dyn StackedDialogState<Global, Event, Error>,
            > + 'static,
    >(
        &self,
    ) -> Option<&R> {
        if self.type_id() == TypeId::of::<R>() {
            let p: *const dyn StackedDialog<
                Global,
                Event,
                Error,
                State = dyn StackedDialogState<Global, Event, Error>,
            > = self;
            Some(unsafe { &*(p as *const R) })
        } else {
            None
        }
    }

    /// down cast Any style.
    pub fn downcast_mut<
        R: StackedDialog<
                Global,
                Event,
                Error,
                State = dyn StackedDialogState<Global, Event, Error>,
            > + 'static,
    >(
        &'_ mut self,
    ) -> Option<&'_ mut R> {
        if (*self).type_id() == TypeId::of::<R>() {
            let p: *mut dyn StackedDialog<
                Global,
                Event,
                Error,
                State = dyn StackedDialogState<Global, Event, Error>,
            > = self;
            Some(unsafe { &mut *(p as *mut R) })
        } else {
            None
        }
    }
}

impl<Global, Event, Error> dyn StackedDialogState<Global, Event, Error>
where
    Global: 'static,
    Event: Send + 'static,
    Error: Send + 'static,
{
    /// down cast Any style.
    pub fn downcast_ref<R: StackedDialogState<Global, Event, Error> + 'static>(
        &self,
    ) -> Option<&R> {
        if self.type_id() == TypeId::of::<R>() {
            let p: *const dyn StackedDialogState<Global, Event, Error> = self;
            Some(unsafe { &*(p as *const R) })
        } else {
            None
        }
    }

    /// down cast Any style.
    pub fn downcast_mut<R: StackedDialogState<Global, Event, Error> + 'static>(
        &'_ mut self,
    ) -> Option<&'_ mut R> {
        if (*self).type_id() == TypeId::of::<R>() {
            let p: *mut dyn StackedDialogState<Global, Event, Error> = self;
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
            for (dialog, state) in inner.dialog.iter().zip(inner.state.iter_mut()) {
                let r = dialog.render(area, buf, state.as_mut(), ctx);
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
    Global: 'static,
    Event: Send + 'static,
    Error: Send + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inner").field("..dyn..", &()).finish()
    }
}

impl<Global, Event, Error> Default for Inner<Global, Event, Error>
where
    Global: 'static,
    Event: Send + 'static,
    Error: Send + 'static,
{
    fn default() -> Self {
        Self {
            dialog: Default::default(),
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
        // no applicable
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

            if inner.dialog.is_empty() {
                self.inner.set(inner);
                return Ok(Control::Continue);
            }

            // only the top dialog gets any events.
            let dialog = inner.dialog.pop().expect("dialog");
            let state = inner.state.pop().expect("state");
            let idx = inner.dialog.len();

            inner.detached = true;
            self.inner.set(inner);

            (idx, dialog, state)
        };

        let r = state.event(event, ctx)?;

        {
            let mut inner = self.inner.replace(Inner::default());
            if !inner.pop_top {
                inner.detached = false;
                inner.dialog.insert(idx, dialog);
                inner.state.insert(idx, state);
            } else {
                inner.detached = false;
                inner.pop_top = false;
            }
            self.inner.set(inner);
        }

        Ok(r)
    }
}

impl<Global, Event, Error> Debug for DialogStackState<Global, Event, Error>
where
    Global: 'static,
    Event: Send + 'static,
    Error: Send + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DialogStackState").finish()
    }
}

impl<Global, Event, Error> Default for DialogStackState<Global, Event, Error>
where
    Global: 'static,
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
    Global: 'static,
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
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a new dialog window on the stack.
    pub fn push_dialog(
        &mut self,
        dialog: impl StackedDialog<
                Global,
                Event,
                Error,
                State = dyn StackedDialogState<Global, Event, Error>,
            > + 'static,
        state: impl StackedDialogState<Global, Event, Error>,
    ) {
        let mut inner = self.inner.replace(Inner::default());
        inner.dialog.push(Box::new(dialog));
        inner.state.push(Box::new(state));
        self.inner.set(inner);
    }

    /// Pop the top dialog from the stack.
    ///
    /// This can be called repeatedly if necessary.
    /// It can even be called during event-handling of the dialog itself.
    pub fn pop_dialog(&mut self) {
        let mut inner = self.inner.replace(Inner::default());
        if inner.detached && !inner.pop_top {
            inner.pop_top = true;
            self.inner.set(inner);
        } else {
            inner.dialog.pop().expect("dialog");
            inner.state.pop().expect("state");
            self.inner.set(inner);
        }
    }

    /// Is the dialog stack empty?
    pub fn is_empty(&self) -> bool {
        let inner = self.inner.replace(Inner::default());
        let r = inner.state.is_empty() && !inner.detached;
        self.inner.set(inner);
        r
    }

    /// Test the type of the top dialog.
    ///
    /// __Note__
    ///
    /// This will not work during event-handling of the dialog itself.
    pub fn top_is<T: 'static>(&self) -> Result<bool, DialogStackError> {
        let inner = self.inner.replace(Inner::default());

        if inner.detached {
            self.inner.set(inner);
            return Err(DialogStackError::InvalidDuringEventHandling);
        }
        if inner.dialog.is_empty() {
            self.inner.set(inner);
            return Err(DialogStackError::StackIsEmpty);
        }

        let dialog = inner.dialog.last().expect("dialog");
        let dyn_transformed = &*dialog.deref();
        let r = dyn_transformed.type_id() == TypeId::of::<T>();

        self.inner.set(inner);

        Ok(r)
    }

    /// Test the type of the top dialog state.
    ///
    /// __Note__
    ///
    /// This will not work during event-handling of the dialog itself.
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
    /// This will not work during event-handling of the dialog itself.
    pub fn map_top_state_if<S, MAP, R>(&self, map: MAP) -> Result<R, DialogStackError>
    where
        MAP: FnOnce(&mut S) -> R,
        S: StackedDialogState<Global, Event, Error>,
    {
        if !self.top_state_is::<S>()? {
            return Err(DialogStackError::TypeMismatch);
        }

        let mut inner = self.inner.replace(Inner::default());

        let dialog = inner.dialog.pop().expect("dialog");
        let mut state = inner.state.pop().expect("state");
        let dyn_state = &mut *state.deref_mut();
        let state_t = dyn_state.downcast_mut::<S>().expect("state");

        let r = map(state_t);

        inner.dialog.push(dialog);
        inner.state.push(state);

        self.inner.set(inner);

        Ok(r)
    }
}

mod test {
    use crate::dialog_stack::{StackedDialog, StackedDialogState};
    use crate::{AppState, AppWidget, RenderContext};
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    #[derive(Debug)]
    pub struct Sample;
    #[derive(Debug)]
    pub struct SampleState {}

    impl<Global, Event, Error> AppWidget<Global, Event, Error> for Sample
    where
        Global: 'static,
        Event: Send + 'static,
        Error: Send + 'static,
    {
        type State = dyn StackedDialogState<Global, Event, Error>;

        fn render(
            &self,
            _area: Rect,
            _buf: &mut Buffer,
            _state: &mut Self::State,
            _ctx: &mut RenderContext<'_, Global>,
        ) -> Result<(), Error> {
            Ok(())
        }
    }

    impl<Global, Event, Error> StackedDialog<Global, Event, Error> for Sample
    where
        Global: 'static,
        Event: Send + 'static,
        Error: Send + 'static,
    {
    }

    impl<Global, Event, Error> AppState<Global, Event, Error> for SampleState
    where
        Global: 'static,
        Event: Send + 'static,
        Error: Send + 'static,
    {
    }

    impl<Global, Event, Error> StackedDialogState<Global, Event, Error> for SampleState
    where
        Global: 'static,
        Event: Send + 'static,
        Error: Send + 'static,
    {
    }
}
