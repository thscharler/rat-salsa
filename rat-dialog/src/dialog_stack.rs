//!
//! A stack of modal dialog windows.
//!
use crate::WindowControl;
use rat_event::HandleEvent;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::widgets::StatefulWidget;
use std::any::{Any, TypeId};
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::fmt::{Debug, Formatter};
use std::mem;
use std::rc::Rc;
use try_as::traits::TryAsRef;

/// Hold a stack of dialog-widgets.
///
/// Renders them and integrates them into event-handling.
///
/// Hold the dialog-stack in your global state,
/// call render() at the very end of rendering and
/// handle() near the start of event-handling.
///
/// The event-handler will consume all crossterm events
/// and block any widget interaction later in event-handling.
/// It will pass through any application level events.
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
                dyn Fn(&Event, &mut dyn Any, &mut Context) -> Result<WindowControl<Event>, Error>
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

impl<Event, Context, Error> StatefulWidget for DialogStack<Event, Context, Error> {
    type State = Context;

    fn render(self, area: Rect, buf: &mut Buffer, ctx: &mut Self::State) {
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
        event: impl Fn(&Event, &mut dyn Any, &'_ mut Context) -> Result<WindowControl<Event>, Error>
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
    /// Return [WindowControl::Close] instead of calling this function.
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
    /// Return [WindowControl::Close] instead of calling this function.
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

    /// Find top state with this type.
    #[allow(clippy::manual_find)]
    pub fn top<S: 'static>(&self) -> Option<usize> {
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
    pub fn get_mut<'a, S: 'static>(&'a self, n: usize) -> Option<RefMut<'a, S>> {
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
    pub fn get<'a, S: 'static>(&'a self, n: usize) -> Option<Ref<'a, S>> {
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
/// crossterm events will only be passed on to the first
/// dialog window. Other application events will go through
/// all the windows on the stack until they are consumed.
///
/// Panic
///
/// This function is not reentrant, it will panic when called from within it's call-stack.
impl<Event, Context, Error> HandleEvent<Event, &mut Context, Result<WindowControl<Event>, Error>>
    for DialogStack<Event, Context, Error>
where
    Event: TryAsRef<ratatui_crossterm::crossterm::event::Event>,
    Error: 'static,
    Event: 'static,
{
    fn handle(&mut self, event: &Event, ctx: &mut Context) -> Result<WindowControl<Event>, Error> {
        for n in (0..self.core.len.get()).rev() {
            let Some(mut state) = self.core.state.borrow_mut()[n].take() else {
                panic!("state is gone");
            };
            let event_fn = mem::replace(
                &mut self.core.event.borrow_mut()[n],
                Box::new(|_, _, _| Ok(WindowControl::Continue)),
            );

            let r = event_fn(event, state.as_mut(), ctx);

            self.core.event.borrow_mut()[n] = event_fn;
            self.core.state.borrow_mut()[n] = Some(state);

            match r {
                Ok(r) => match r {
                    WindowControl::Close(_) => {
                        self.remove(n);
                        return Ok(r);
                    }
                    WindowControl::Event(_) => {
                        return Ok(r);
                    }
                    WindowControl::Unchanged => {
                        return Ok(r);
                    }
                    WindowControl::Changed => {
                        return Ok(r);
                    }
                    WindowControl::Continue => {
                        // next
                    }
                },
                Err(e) => return Err(e),
            }

            // Block all crossterm events.
            let event: Option<&ratatui_crossterm::crossterm::event::Event> = event.try_as_ref();
            if event.is_some() {
                return Ok(WindowControl::Unchanged);
            }
        }

        Ok(WindowControl::Continue)
    }
}

/// Handle events from top to bottom of the stack.
///
/// Panic
///
/// This function is not reentrant, it will panic when called from within it's call-stack.
pub fn handle_dialog_stack<Event, Context, Error>(
    mut dialog_stack: DialogStack<Event, Context, Error>,
    event: &Event,
    ctx: &mut Context,
) -> Result<WindowControl<Event>, Error>
where
    Event: TryAsRef<ratatui_crossterm::crossterm::event::Event>,
    Error: 'static,
    Event: 'static,
{
    dialog_stack.handle(event, ctx)
}
