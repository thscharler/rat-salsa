//!
//! DialogStack adds support for dialog windows.
//!
//! ** unstable **
//!

use crate::{AppContext, AppState, AppWidget, Control, RenderContext};
use rat_widget::text::HasScreenCursor;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::any::{Any, TypeId};
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::fmt::{Debug, Formatter};
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

struct Inner<Global, Event, Error> {
    dialog: Vec<Box<DynStackedDialog<Global, Event, Error>>>,
    state: Vec<Box<dyn StackedDialogState<Global, Event, Error>>>,
}

pub type DynStackedDialog<Global, Event, Error> =
    dyn StackedDialog<Global, Event, Error, State = dyn StackedDialogState<Global, Event, Error>>;

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
    /// Calculate the area where the dialog will be rendered.
    fn layout(
        &self,
        area: Rect,
        buf: &Buffer,
        state: &mut Self::State,
        ctx: &mut RenderContext<'_, Global>,
    ) -> Result<Rect, Error>;
}

/// Trait for a dialogs state.
pub trait StackedDialogState<Global, Event, Error>
where
    Global: 'static,
    Event: Send + 'static,
    Error: Send + 'static,
    Self: Any + 'static,
    Self: AppState<Global, Event, Error>,
    Self: HasScreenCursor,
{
    /// Is the dialog closed.
    /// This will be checked after event-handling and will pop the
    /// dialog from the stack.
    fn closed(&self) -> bool;
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
        for (dialog, state) in inner.dialog.iter_mut().zip(inner.state.iter_mut()) {
            let area = dialog.layout(area, buf, state.as_mut(), ctx)?;
            dialog.render(area, buf, state.as_mut(), ctx)?;
        }
        state.inner.set(inner);
        Ok(())
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
        for state in inner.state.iter_mut().rev() {
            state.shutdown(ctx)?;
        }
        self.inner.set(inner);
        Ok(())
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

            self.inner.set(inner);

            (idx, dialog, state)
        };

        let r = state.event(event, ctx)?;

        {
            let mut inner = self.inner.replace(Inner::default());
            inner.dialog.insert(idx, dialog);
            inner.state.insert(idx, state);
            self.inner.set(inner);
        }

        Ok(r)
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
        }
    }
}

impl<Global, Event, Error> Debug for Inner<Global, Event, Error>
where
    Global: 'static,
    Event: Send + 'static,
    Error: Send + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
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

impl<Global, Event, Error> Debug for DialogStackState<Global, Event, Error>
where
    Global: 'static,
    Event: Send + 'static,
    Error: Send + 'static,
{
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!();
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
    ///
    /// It is popped again when it's closed() function returns true.
    /// This is checked during event handling.
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

    /// Test the type of the top dialog.
    pub fn top_is<T: 'static>(&self) -> bool {
        let inner = self.inner.replace(Inner::default());
        let r = if let Some(inner) = inner.dialog.last() {
            let dyn_transformed = &*inner.deref();
            dyn_transformed.type_id() == TypeId::of::<T>()
        } else {
            false
        };
        self.inner.set(inner);
        r
    }

    /// Test the type of the top dialog state.
    pub fn top_state_is<T: 'static>(&self) -> bool {
        let inner = self.inner.replace(Inner::default());
        let r = if let Some(inner) = inner.state.last() {
            let dyn_transformed = &*inner.deref();
            dyn_transformed.type_id() == TypeId::of::<T>()
        } else {
            false
        };
        self.inner.set(inner);
        r
    }
}

mod test {
    use crate::dialog_stack::{StackedDialog, StackedDialogState};
    use crate::{AppState, AppWidget, RenderContext};
    use rat_widget::text::HasScreenCursor;
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
        fn layout(
            &self,
            _area: Rect,
            _buf: &Buffer,
            _state: &mut Self::State,
            _ctx: &mut RenderContext<'_, Global>,
        ) -> Result<Rect, Error> {
            todo!()
        }
    }

    impl<Global, Event, Error> AppState<Global, Event, Error> for SampleState
    where
        Global: 'static,
        Event: Send + 'static,
        Error: Send + 'static,
    {
    }

    impl HasScreenCursor for SampleState {
        fn screen_cursor(&self) -> Option<(u16, u16)> {
            None
        }
    }

    impl<Global, Event, Error> StackedDialogState<Global, Event, Error> for SampleState
    where
        Global: 'static,
        Event: Send + 'static,
        Error: Send + 'static,
    {
        fn closed(&self) -> bool {
            true
        }
    }
}
