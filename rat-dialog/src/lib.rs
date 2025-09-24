#![allow(clippy::question_mark)]
#![allow(clippy::type_complexity)]

use rat_event::{ConsumedEvent, HandleEvent, Outcome};
use rat_salsa::{Control, SalsaContext};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::any::{Any, TypeId};
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::fmt::{Debug, Formatter};
use std::mem;
use std::rc::Rc;

/// Extends rat-salsa::Control with some dialog specific options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[must_use]
#[non_exhaustive]
pub enum DialogControl<Event> {
    /// Continue with event-handling.
    /// In the event-loop this waits for the next event.
    Continue,
    /// Break event-handling without repaint.
    /// In the event-loop this waits for the next event.
    Unchanged,
    /// Break event-handling and repaints/renders the application.
    /// In the event-loop this calls `render`.
    Changed,
    /// Return back some application event.
    Event(Event),
    /// Close the dialog
    Close(Event),
    /// Quit
    Quit,
}

impl<Event> ConsumedEvent for DialogControl<Event> {
    fn is_consumed(&self) -> bool {
        !matches!(self, DialogControl::Continue)
    }
}

impl<Event, T: Into<Outcome>> From<T> for DialogControl<Event> {
    fn from(value: T) -> Self {
        let r = value.into();
        match r {
            Outcome::Continue => DialogControl::Continue,
            Outcome::Unchanged => DialogControl::Unchanged,
            Outcome::Changed => DialogControl::Changed,
        }
    }
}

/// Hold a stack of widgets.
///
/// Renders the widgets and can handle events.
///
/// Hold the dialog-stack in your global state,
/// call render() at the very end of rendering and
/// handle() near the start of event-handling.
///
/// This will not handle modality, so make sure
/// to consume all events you don't want to propagate.
///
pub struct DialogStack<Event, Context, Error> {
    core: Rc<DialogStackCore<Event, Context, Error>>,
}

struct DialogStackCore<Event, Context, Error> {
    len: Cell<usize>,
    render: RefCell<Vec<Box<dyn Fn(Rect, &mut Buffer, &mut dyn Any, &mut Context) + 'static>>>,
    event: RefCell<
        Vec<
            Box<
                dyn Fn(&Event, &mut dyn Any, &mut Context) -> Result<DialogControl<Event>, Error>
                    + 'static,
            >,
        >,
    >,
    type_id: RefCell<Vec<TypeId>>,
    state: RefCell<Vec<Option<Box<dyn Any>>>>,
}

impl<Event, Context, Error> Clone for DialogStack<Event, Context, Error> {
    fn clone(&self) -> Self {
        Self {
            core: self.core.clone(),
        }
    }
}

impl<Event, Context, Error> Default for DialogStack<Event, Context, Error> {
    fn default() -> Self {
        Self {
            core: Rc::new(DialogStackCore {
                len: Cell::new(0),
                render: Default::default(),
                event: Default::default(),
                type_id: Default::default(),
                state: Default::default(),
            }),
        }
    }
}

impl<Event, Context, Error> Debug for DialogStack<Event, Context, Error> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let state = self.core.state.borrow();
        let is_proxy = state.iter().map(|v| v.is_none()).collect::<Vec<_>>();
        let type_id = self.core.type_id.borrow();

        f.debug_struct("DialogStackCore")
            .field("len", &self.core.len.get())
            .field("type_id", &type_id)
            .field("is_proxy", &is_proxy)
            .finish()
    }
}

impl<Event, Context, Error> DialogStack<Event, Context, Error> {
    /// Render all dialog-windows in stack-order.
    pub fn render(self, area: Rect, buf: &mut Buffer, ctx: &mut Context) {
        for n in 0..self.core.len.get() {
            let Some(mut state) = self.core.state.borrow_mut()[n].take() else {
                panic!("state is gone");
            };
            let render_fn = mem::replace(
                &mut self.core.render.borrow_mut()[n],
                Box::new(|_, _, _, _| {}),
            );

            render_fn(area, buf, state.as_mut(), ctx);

            self.core.render.borrow_mut()[n] = render_fn;
            self.core.state.borrow_mut()[n] = Some(state);
        }
    }
}

impl<Event, Context, Error> DialogStack<Event, Context, Error> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a dialog-window on the stack.
    /// - render is called in reverse stack order, to render bottom to top.
    /// - event is called in stack-order to handle events.
    ///   if you don't want events to propagate to dialog-windows in the
    ///   background, you must consume them by returning StackControl::Unchanged.
    /// - state as Any
    pub fn push(
        &self,
        render: impl Fn(Rect, &mut Buffer, &mut dyn Any, &'_ mut Context) + 'static,
        event: impl Fn(&Event, &mut dyn Any, &'_ mut Context) -> Result<DialogControl<Event>, Error>
        + 'static,
        state: impl Any,
    ) {
        self.core.len.update(|v| v + 1);
        self.core.type_id.borrow_mut().push(state.type_id());
        self.core.state.borrow_mut().push(Some(Box::new(state)));
        self.core.event.borrow_mut().push(Box::new(event));
        self.core.render.borrow_mut().push(Box::new(render));
    }

    /// Pop the top dialog-window from the stack.
    ///
    /// It will return None if the stack is empty.
    ///
    /// Panic
    ///
    /// This function is partially reentrant. When called during rendering/event-handling
    /// it will panic when trying to pop your current dialog-window.
    /// Return [DialogControl::Close] instead of calling this function.
    pub fn pop(&self) -> Option<Box<dyn Any>> {
        self.core.len.update(|v| v - 1);
        self.core.type_id.borrow_mut().pop();
        self.core.event.borrow_mut().pop();
        self.core.render.borrow_mut().pop();
        let Some(s) = self.core.state.borrow_mut().pop() else {
            return None;
        };
        if s.is_none() {
            panic!("state is gone");
        }
        s
    }

    /// Remove some dialog-window.
    ///
    /// Panic
    ///
    /// This function is not reentrant. It will panic when called during
    /// rendering or event-handling of any dialog-window.
    /// Return [DialogControl::Close] instead of calling this function.
    ///
    /// Panics when out-of-bounds.
    pub fn remove(&self, n: usize) -> Box<dyn Any> {
        for s in self.core.state.borrow().iter() {
            if s.is_none() {
                panic!("state is gone");
            }
        }

        self.core.len.update(|v| v - 1);
        self.core.type_id.borrow_mut().remove(n);
        _ = self.core.event.borrow_mut().remove(n);
        _ = self.core.render.borrow_mut().remove(n);

        self.core
            .state
            .borrow_mut()
            .remove(n)
            .expect("state exists")
    }

    /// Move the given dialog-window to the top of the stack.
    ///
    /// Panic
    ///
    /// This function is not reentrant. It will panic when called during
    /// rendering or event-handling of any dialog-window. Use [DialogControl::ToFront]
    /// for this.
    ///
    /// Panics when out-of-bounds.
    pub fn to_front(&self, n: usize) {
        for s in self.core.state.borrow().iter() {
            if s.is_none() {
                panic!("state is gone");
            }
        }

        let type_id = self.core.type_id.borrow_mut().remove(n);
        let state = self.core.state.borrow_mut().remove(n);
        let event = self.core.event.borrow_mut().remove(n);
        let render = self.core.render.borrow_mut().remove(n);

        self.core.type_id.borrow_mut().push(type_id);
        self.core.state.borrow_mut().push(state);
        self.core.event.borrow_mut().push(event);
        self.core.render.borrow_mut().push(render);
    }

    /// No windows.
    pub fn is_empty(&self) -> bool {
        self.core.type_id.borrow().is_empty()
    }

    /// Number of dialog-windows.
    pub fn len(&self) -> usize {
        self.core.len.get()
    }

    /// Typecheck the state.
    pub fn state_is<S: 'static>(&self, n: usize) -> bool {
        self.core.type_id.borrow()[n] == TypeId::of::<S>()
    }

    /// Find first state with this type.
    #[allow(clippy::manual_find)]
    pub fn first<S: 'static>(&self) -> Option<usize> {
        for n in (0..self.core.len.get()).rev() {
            if self.core.type_id.borrow()[n] == TypeId::of::<S>() {
                return Some(n);
            }
        }
        None
    }

    /// Find all states with this type.
    pub fn find<S: 'static>(&self) -> Vec<usize> {
        self.core
            .type_id
            .borrow()
            .iter()
            .enumerate()
            .rev()
            .filter_map(|(n, v)| {
                if *v == TypeId::of::<S>() {
                    Some(n)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get a reference to the state at index n.
    ///
    /// Panic
    ///
    /// Panics when out-of-bounds.
    /// Panics when recursively accessing the same state. Accessing a
    /// *different* window-state is fine.
    /// Panics when the types don't match.
    pub fn get<'a, S: 'static>(&'a self, n: usize) -> Ref<'a, S> {
        self.try_get(n).expect("recursion or wrong type")
    }

    /// Get a mutable reference to the state at index n.
    ///
    /// Panic
    ///
    /// Panics when out-of-bounds.
    /// Panics when recursively accessing the same state. Accessing a
    /// *different* window-state is fine.
    /// Panics when the types don't match.
    pub fn get_mut<'a, S: 'static>(&'a self, n: usize) -> RefMut<'a, S> {
        self.try_get_mut(n).expect("recursion or wrong type")
    }

    /// Get a mutable reference to the state at index n.
    ///
    /// Panic
    ///
    /// Panics when out-of-bounds.
    ///
    /// Fails
    ///
    /// Fails when recursively accessing the same state. Accessing a
    /// *different* window-state is fine.
    /// Fails when the types don't match.
    pub fn try_get_mut<'a, S: 'static>(&'a self, n: usize) -> Option<RefMut<'a, S>> {
        let state = self.core.state.borrow_mut();

        RefMut::filter_map(state, |v| {
            let state = &mut v[n];
            if let Some(state) = state.as_mut() {
                if let Some(state) = state.downcast_mut::<S>() {
                    Some(state)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .ok()
    }

    /// Get a reference to the state at index n.
    ///
    /// Panic
    ///
    /// Panics when out-of-bounds.
    ///
    /// Fails
    ///
    /// Fails when recursively accessing the same state. Accessing a
    /// *different* window-state is fine.
    /// Fails when the types don't match.
    pub fn try_get<'a, S: 'static>(&'a self, n: usize) -> Option<Ref<'a, S>> {
        let state = self.core.state.borrow();

        Ref::filter_map(state, |v| {
            let state = &v[n];
            if let Some(state) = state.as_ref() {
                if let Some(state) = state.downcast_ref::<S>() {
                    Some(state)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .ok()
    }
}

/// Handle events from top to bottom of the stack.
///
/// Panic
///
/// This function is not reentrant, it will panic when called from within it's call-stack.
impl<Event, Context, Error> HandleEvent<Event, &mut Context, Result<Control<Event>, Error>>
    for DialogStack<Event, Context, Error>
where
    Context: SalsaContext<Event, Error>,
    Error: 'static,
    Event: 'static,
{
    fn handle(&mut self, event: &Event, ctx: &mut Context) -> Result<Control<Event>, Error> {
        for n in (0..self.core.len.get()).rev() {
            let Some(mut state) = self.core.state.borrow_mut()[n].take() else {
                panic!("state is gone");
            };
            let event_fn = mem::replace(
                &mut self.core.event.borrow_mut()[n],
                Box::new(|_, _, _| Ok(DialogControl::Continue)),
            );

            let r = event_fn(event, state.as_mut(), ctx);

            self.core.event.borrow_mut()[n] = event_fn;
            self.core.state.borrow_mut()[n] = Some(state);

            match r {
                Ok(r) => match r {
                    DialogControl::Close(event) => {
                        self.remove(n);
                        return Ok(Control::Event(event));
                    }
                    DialogControl::Event(event) => {
                        return Ok(Control::Event(event));
                    }
                    DialogControl::Unchanged => {
                        return Ok(Control::Unchanged);
                    }
                    DialogControl::Changed => {
                        return Ok(Control::Changed);
                    }
                    DialogControl::Quit => {
                        return Ok(Control::Quit);
                    }
                    DialogControl::Continue => {
                        // next
                    }
                },
                Err(e) => return Err(e),
            }
        }

        Ok(Control::Continue)
    }
}
