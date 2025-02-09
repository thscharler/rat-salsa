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
use std::cell::{Ref, RefCell, RefMut};
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
    inner: Rc<RefCell<Vec<Inner<Global, Event, Error>>>>,
}

struct Inner<Global, Event, Error> {
    dialog: Box<DynStackedDialog<Global, Event, Error>>,
    state: Box<dyn StackedDialogState<Global, Event, Error>>,
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
        let mut inner = state.inner.borrow_mut();
        for v in inner.iter_mut() {
            let area = v.dialog.layout(area, buf, v.state.as_mut(), ctx)?;
            v.dialog.render(area, buf, v.state.as_mut(), ctx)?;
        }
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
        let mut inner = self.inner.borrow_mut();
        for v in inner.iter_mut().rev() {
            v.state.shutdown(ctx)?;
        }
        Ok(())
    }

    fn event(
        &mut self,
        event: &Event,
        ctx: &mut AppContext<'_, Global, Event, Error>,
    ) -> Result<Control<Event>, Error> {
        let (idx, dialog, mut state) = {
            let mut inner = self.inner.borrow_mut();

            // only the top dialog gets any events.
            let Some(Inner { dialog, state }) = inner.pop() else {
                return Ok(Control::Continue);
            };
            let idx = inner.len();

            (idx, dialog, state)
        };

        let r = state.event(event, ctx)?;

        let closed = state.closed();

        {
            let mut inner = self.inner.borrow_mut();
            if !closed {
                inner.insert(idx, Inner { dialog, state });
            }
        }

        Ok(r)
    }
}

impl<Global, Event, Error> HasScreenCursor for DialogStackState<Global, Event, Error> {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        let inner = self.inner.borrow();
        if let Some(last) = inner.last() {
            last.state.screen_cursor()
        } else {
            None
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
        let mut inner = self.inner.borrow_mut();
        inner.push(Inner {
            dialog: Box::new(dialog),
            state: Box::new(state),
        });
    }

    /// Test the type of the top dialog.
    pub fn top_is<T: 'static>(&self) -> bool {
        let inner = self.inner.borrow();
        if let Some(inner) = inner.last() {
            let dyn_transformed = &*inner.dialog.deref();
            dyn_transformed.type_id() == TypeId::of::<T>()
        } else {
            false
        }
    }

    /// Test the type of the top dialog state.
    pub fn top_state_is<T: 'static>(&self) -> bool {
        let inner = self.inner.borrow();
        if let Some(inner) = inner.last() {
            let dyn_transformed = &*inner.state.deref();
            dyn_transformed.type_id() == TypeId::of::<T>()
        } else {
            false
        }
    }

    /// Get a Ref to the top dialog state.
    pub fn top_state<T>(&self) -> Option<Ref<T>>
    where
        T: StackedDialogState<Global, Event, Error> + 'static,
    {
        let inner = self.inner.borrow();
        if inner.is_empty() {
            return None;
        }
        if !self.top_state_is::<T>() {
            return None;
        }

        let dcr = Ref::map(inner, |inner| {
            let inner = inner.last().expect("last");
            let dyn_transformed = &*inner.state.deref();
            let dc = dyn_transformed.downcast_ref::<T>().expect("T");
            dc
        });
        Some(dcr)
    }

    /// Get a RefMut to the top dialog state.
    pub fn top_state_mut<T>(&self) -> Option<RefMut<T>>
    where
        T: StackedDialogState<Global, Event, Error> + 'static,
    {
        let inner = self.inner.borrow_mut();
        if inner.is_empty() {
            return None;
        }
        if !self.top_state_is::<T>() {
            return None;
        }

        let dcr = RefMut::map(inner, |inner| {
            let inner = inner.last_mut().expect("last");
            let dyn_transformed = &mut *inner.state.deref_mut();
            let dc = dyn_transformed.downcast_mut::<T>().expect("T");
            dc
        });
        Some(dcr)
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
