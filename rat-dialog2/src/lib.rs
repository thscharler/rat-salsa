use rat_event::{ConsumedEvent, HandleEvent, Outcome};
use rat_salsa2::{AppContext, Control, RenderContext};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::any::{type_name, Any, TypeId};
use std::cell::{Cell, RefCell};
use std::fmt::{Debug, Formatter};
use std::mem;
use std::rc::Rc;

/// Extends rat-salsa::Control with some dialog specific options.
#[derive(Debug, Clone, Copy)]
#[must_use]
#[non_exhaustive]
pub enum StackControl<Event> {
    /// Continue with event-handling.
    /// In the event-loop this waits for the next event.
    Continue,
    /// Break event-handling without repaint.
    /// In the event-loop this waits for the next event.
    Unchanged,
    /// Break event-handling and repaints/renders the application.
    /// In the event-loop this calls `render`.
    Changed,
    /// Eventhandling can cause secondary application specific events.
    /// One common way is to return this `Control::Message(my_event)`
    /// to reenter the event-loop with your own secondary event.
    ///
    /// This acts quite like a message-queue to communicate between
    /// disconnected parts of your application. And indeed there is
    /// a hidden message-queue as part of the event-loop.
    ///
    /// The other way is to call [AppContext::queue] to initiate such
    /// events.
    Event(Event),
    /// Quit the application.
    Quit,
    /// Close the dialog
    Pop,
    /// Move to front
    ToFront,
}

impl<Event> Eq for StackControl<Event> {}

impl<Event> PartialEq for StackControl<Event> {
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }
}

impl<Event> ConsumedEvent for StackControl<Event> {
    fn is_consumed(&self) -> bool {
        !matches!(self, StackControl::Continue)
    }
}

impl<Event> From<Outcome> for StackControl<Event> {
    fn from(value: Outcome) -> Self {
        match value {
            Outcome::Continue => StackControl::Continue,
            Outcome::Unchanged => StackControl::Unchanged,
            Outcome::Changed => StackControl::Changed,
        }
    }
}

impl<Event> From<Control<Event>> for StackControl<Event> {
    fn from(value: Control<Event>) -> Self {
        match value {
            Control::Continue => StackControl::Continue,
            Control::Unchanged => StackControl::Unchanged,
            Control::Changed => StackControl::Changed,
            Control::Event(evt) => StackControl::Event(evt),
            Control::Quit => StackControl::Quit,
            _ => {
                unreachable!()
            }
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
pub struct DialogStack<Global, Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    core: Rc<DialogStackCore<Global, Event, Error>>,
}

struct DialogStackCore<Global, Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    len: Cell<usize>,
    render: RefCell<
        Vec<
            Box<
                dyn Fn(
                        Rect,
                        &mut Buffer,
                        &mut dyn Any,
                        &'_ mut RenderContext<'_, Global>,
                    ) -> Result<(), Error>
                    + 'static,
            >,
        >,
    >,
    event: RefCell<
        Vec<
            Box<
                dyn Fn(
                    &Event, //
                    &mut dyn Any,
                    &'_ mut AppContext<'_, Global, Event, Error>,
                ) -> Result<StackControl<Event>, Error>,
            >,
        >,
    >,
    type_id: RefCell<Vec<TypeId>>,
    state: RefCell<Vec<Option<Box<dyn Any>>>>,
}

impl<Global, Event, Error> Clone for DialogStack<Global, Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    fn clone(&self) -> Self {
        Self {
            core: self.core.clone(),
        }
    }
}

impl<Global, Event, Error> Default for DialogStack<Global, Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
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

impl<Global, Event, Error> Debug for DialogStack<Global, Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
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

impl<Global, Event, Error> DialogStack<Global, Event, Error>
where
    Event: Send + 'static,
    Error: Send + 'static,
{
    /// Render all dialog-windows in stack-order.
    pub fn render(
        self,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderContext<'_, Global>,
    ) -> Result<(), Error>
    where
        Event: 'static + Send,
        Error: 'static + Send,
    {
        for n in 0..self.core.len.get() {
            let Some(mut state) = self.core.state.borrow_mut()[n].take() else {
                panic!("state is gone");
            };
            let render_fn = mem::replace(
                &mut self.core.render.borrow_mut()[n],
                Box::new(|_, _, _, _| Ok(())),
            );

            let r = render_fn(area, buf, state.as_mut(), ctx);

            self.core.render.borrow_mut()[n] = render_fn;
            self.core.state.borrow_mut()[n] = Some(state);

            r?
        }
        Ok(())
    }
}

impl<Global, Event, Error> DialogStack<Global, Event, Error>
where
    Event: Send + 'static,
    Error: Send + 'static,
{
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
        render: impl Fn(
                Rect,
                &mut Buffer,
                &mut dyn Any,
                &'_ mut RenderContext<'_, Global>,
            ) -> Result<(), Error>
            + 'static,
        event: impl Fn(
                &Event,
                &mut dyn Any,
                &'_ mut AppContext<'_, Global, Event, Error>,
            ) -> Result<StackControl<Event>, Error>
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
    /// Return StackControl::Pop instead of calling this function.
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
        let s = self
            .core
            .state
            .borrow_mut()
            .remove(n)
            .expect("state exists");
        s
    }

    /// Move the given dialog-window to the top of the stack.
    ///
    /// Panic
    ///
    /// This function is not reentrant. It will panic when called during
    /// rendering or event-handling of any dialog-window. Use StackControl::ToFront
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

    /// Run f for the given instance of S.
    ///
    /// Panic
    ///
    /// Panics when out-of-bounds.
    /// Panics when recursively accessing the same state. Accessing a
    /// *different* window-state is fine.
    /// Panics when the types don't match.
    pub fn apply<S: 'static, R>(&self, n: usize, f: impl Fn(&S) -> R) -> R {
        let Some(state) = self.core.state.borrow_mut()[n].take() else {
            panic!("state is gone");
        };

        let r = if let Some(state) = state.as_ref().downcast_ref::<S>() {
            f(state)
        } else {
            self.core.state.borrow_mut()[n] = Some(state);
            panic!("state is not {:?}", type_name::<S>());
        };

        self.core.state.borrow_mut()[n] = Some(state);
        r
    }

    /// Run f for the given instance of S with exclusive/mutabel access.
    ///
    /// Panic
    ///
    /// Panics when out-of-bounds.
    /// Panics when recursively accessing the same state. Accessing a
    /// *different* window-state is fine.
    /// Panics when the types don't match.
    pub fn apply_mut<S: 'static, R>(&mut self, n: usize, f: impl Fn(&mut S) -> R) -> R {
        let Some(mut state) = self.core.state.borrow_mut()[n].take() else {
            panic!("state is gone");
        };

        let r = if let Some(state) = state.as_mut().downcast_mut::<S>() {
            f(state)
        } else {
            self.core.state.borrow_mut()[n] = Some(state);
            panic!("state is not {:?}", type_name::<S>());
        };

        self.core.state.borrow_mut()[n] = Some(state);
        r
    }
}

/// Handle events from top to bottom of the stack.
///
/// Panic
///
/// This function is not reentrant, it will panic when called from within it's call-stack.
impl<Global, Event, Error>
    HandleEvent<Event, &mut AppContext<'_, Global, Event, Error>, Result<Control<Event>, Error>>
    for DialogStack<Global, Event, Error>
where
    Event: 'static + Send + Debug,
    Error: 'static + Send,
{
    fn handle(
        &mut self,
        event: &Event,
        ctx: &mut AppContext<'_, Global, Event, Error>,
    ) -> Result<Control<Event>, Error> {
        let mut rr = Control::Continue;

        for n in (0..self.core.len.get()).rev() {
            let Some(mut state) = self.core.state.borrow_mut()[n].take() else {
                panic!("state is gone");
            };
            let event_fn = mem::replace(
                &mut self.core.event.borrow_mut()[n],
                Box::new(|_, _, _| Ok(StackControl::Continue)),
            );

            let r = event_fn(event, state.as_mut(), ctx);

            self.core.event.borrow_mut()[n] = event_fn;
            self.core.state.borrow_mut()[n] = Some(state);

            match r {
                Ok(r) => {
                    match r {
                        StackControl::Continue => {
                            // noop, bottom of it
                        }
                        StackControl::Unchanged => {
                            if rr < Control::Unchanged {
                                rr = Control::Unchanged
                            }
                        }
                        StackControl::Changed => {
                            if rr < Control::Changed {
                                rr = Control::Changed
                            }
                        }
                        StackControl::Event(evt) => {
                            ctx.queue_event(evt);
                            // don't change rr. besides queuing the event
                            // this is Control::Continue anyway.
                        }
                        StackControl::Quit => {
                            if rr < Control::Quit {
                                rr = Control::Quit
                            }
                        }
                        StackControl::Pop => {
                            self.remove(n);
                            if rr < Control::Changed {
                                rr = Control::Changed
                            }
                        }
                        StackControl::ToFront => {
                            self.to_front(n);
                            if rr < Control::Changed {
                                rr = Control::Changed
                            }
                        }
                    }
                }
                Err(e) => return Err(e),
            }
        }

        Ok(rr)
    }
}
