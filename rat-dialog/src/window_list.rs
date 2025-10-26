//!
//! A list of modeless application windows.
//!
use crate::WindowControl;
use rat_event::util::mouse_trap;
use rat_event::{ConsumedEvent, HandleEvent, Outcome, ct_event};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::any::{Any, TypeId};
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::fmt::{Debug, Formatter};
use std::mem;
use std::rc::Rc;
use try_as::traits::TryAsRef;

/// Necessary trait for your window-state.
///
/// This enables reordering the application windows.
pub trait Window<Context>: Any
where
    Context: 'static,
{
    /// Set as top window.
    fn set_top(&mut self, top: bool, ctx: &mut Context);
    /// Window area.
    fn area(&self) -> Rect;
}

impl<Context> dyn Window<Context> {
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref::<T>()
    }

    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        (self as &mut dyn Any).downcast_mut::<T>()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WindowFrameOutcome {
    /// The given event was not handled at all.
    Continue,
    /// The event was handled, no repaint necessary.
    Unchanged,
    /// The event was handled, repaint necessary.
    Changed,
    /// Request close.
    ShouldClose,
    /// Has moved.
    Moved,
    /// Has resized.
    Resized,
}

impl ConsumedEvent for WindowFrameOutcome {
    fn is_consumed(&self) -> bool {
        *self != WindowFrameOutcome::Continue
    }
}

impl From<Outcome> for WindowFrameOutcome {
    fn from(value: Outcome) -> Self {
        match value {
            Outcome::Continue => WindowFrameOutcome::Continue,
            Outcome::Unchanged => WindowFrameOutcome::Unchanged,
            Outcome::Changed => WindowFrameOutcome::Changed,
        }
    }
}

impl From<WindowFrameOutcome> for Outcome {
    fn from(value: WindowFrameOutcome) -> Self {
        match value {
            WindowFrameOutcome::Continue => Outcome::Continue,
            WindowFrameOutcome::Unchanged => Outcome::Unchanged,
            WindowFrameOutcome::Changed => Outcome::Changed,
            WindowFrameOutcome::Moved => Outcome::Changed,
            WindowFrameOutcome::Resized => Outcome::Changed,
            WindowFrameOutcome::ShouldClose => Outcome::Continue,
        }
    }
}

///
/// A list of [Window]s.
///
pub struct WindowList<Event, Context, Error> {
    core: Rc<WindowListCore<Event, Context, Error>>,
}

struct WindowListCore<Event, Context, Error> {
    len: Cell<usize>,
    render: RefCell<
        Vec<Box<dyn Fn(Rect, &mut Buffer, &mut dyn Window<Context>, &mut Context) + 'static>>,
    >,
    event: RefCell<
        Vec<
            Box<
                dyn Fn(
                        &Event,
                        &mut dyn Window<Context>,
                        &mut Context,
                    ) -> Result<WindowControl<Event>, Error>
                    + 'static,
            >,
        >,
    >,
    type_id: RefCell<Vec<TypeId>>,
    state: RefCell<Vec<Option<Box<dyn Window<Context>>>>>,
}

impl<Event, Context, Error> Clone for WindowList<Event, Context, Error> {
    fn clone(&self) -> Self {
        Self {
            core: self.core.clone(),
        }
    }
}

impl<Event, Context, Error> Default for WindowList<Event, Context, Error> {
    fn default() -> Self {
        Self {
            core: Rc::new(WindowListCore {
                len: Cell::new(0),
                render: Default::default(),
                event: Default::default(),
                type_id: Default::default(),
                state: Default::default(),
            }),
        }
    }
}

impl<Event, Context, Error> Debug for WindowList<Event, Context, Error> {
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

impl<Event, Context, Error> WindowList<Event, Context, Error> {
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

impl<Event, Context, Error> WindowList<Event, Context, Error>
where
    Context: 'static,
{
    pub fn new() -> Self {
        Self::default()
    }

    /// Show a window.
    ///
    /// This shows the window on top of any other.
    pub fn show(
        &self,
        render: impl Fn(Rect, &mut Buffer, &mut dyn Window<Context>, &'_ mut Context) + 'static,
        event: impl Fn(
            &Event,
            &mut dyn Window<Context>,
            &'_ mut Context,
        ) -> Result<WindowControl<Event>, Error>
        + 'static,
        state: impl Window<Context>,
        ctx: &mut Context,
    ) {
        self.core.len.update(|v| v + 1);
        self.core.type_id.borrow_mut().push(state.type_id());
        self.core.state.borrow_mut().push(Some(Box::new(state)));
        self.core.event.borrow_mut().push(Box::new(event));
        self.core.render.borrow_mut().push(Box::new(render));

        self.set_top_window(ctx);
    }

    /// Remove the window from the list.
    ///
    /// Panic
    ///
    /// This function is not reentrant. It will panic when called during
    /// rendering or event-handling of any dialog-window.
    /// Return [WindowControl::Close] instead of calling this function.
    ///
    /// Panics when out-of-bounds.
    pub fn close(&self, n: usize, ctx: &mut Context) -> Box<dyn Window<Context>> {
        for s in self.core.state.borrow().iter() {
            if s.is_none() {
                panic!("state is gone");
            }
        }

        self.core.len.update(|v| v - 1);
        self.core.type_id.borrow_mut().remove(n);
        _ = self.core.event.borrow_mut().remove(n);
        _ = self.core.render.borrow_mut().remove(n);

        let r = self
            .core
            .state
            .borrow_mut()
            .remove(n)
            .expect("state exists");

        self.set_top_window(ctx);

        r
    }

    /// Move the given window to the top.
    ///
    /// Panic
    ///
    /// This function is not reentrant. It will panic when called during
    /// rendering or event-handling of any window. It is not necessary
    /// to call this during event handling as this happens automatically.
    ///
    /// Panics when out-of-bounds.
    pub fn to_front(&self, n: usize, ctx: &mut Context) {
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

        self.set_top_window(ctx);
    }

    /// Move the given window to the bottom.
    ///
    /// Panic
    ///
    /// This function is not reentrant. It will panic when called during
    /// rendering or event-handling of any dialog-window.
    ///
    /// Panics when out-of-bounds.
    pub fn to_back(&self, n: usize, ctx: &mut Context) {
        for s in self.core.state.borrow().iter() {
            if s.is_none() {
                panic!("state is gone");
            }
        }

        let type_id = self.core.type_id.borrow_mut().remove(n);
        let state = self.core.state.borrow_mut().remove(n);
        let event = self.core.event.borrow_mut().remove(n);
        let render = self.core.render.borrow_mut().remove(n);

        self.core.type_id.borrow_mut().insert(0, type_id);
        self.core.state.borrow_mut().insert(0, state);
        self.core.event.borrow_mut().insert(0, event);
        self.core.render.borrow_mut().insert(0, render);

        self.set_top_window(ctx);
    }

    fn set_top_window(&self, ctx: &mut Context) {
        let len = self.len();

        if len > 0 {
            let mut states = self.core.state.borrow_mut();
            for i in 0..len - 1 {
                if let Some(s) = &mut states[i] {
                    s.set_top(false, ctx);
                } else {
                    panic!("state is gone");
                }
            }
            if let Some(s) = &mut states[len - 1] {
                s.set_top(true, ctx);
            } else {
                panic!("state is gone");
            }
        }
    }

    /// No windows.
    pub fn is_empty(&self) -> bool {
        self.core.type_id.borrow().is_empty()
    }

    /// Number of windows.
    pub fn len(&self) -> usize {
        self.core.len.get()
    }

    /// Typecheck the window.
    pub fn state_is<S: 'static>(&self, n: usize) -> bool {
        self.core.type_id.borrow()[n] == TypeId::of::<S>()
    }

    /// Find top window with this type.
    #[allow(clippy::manual_find)]
    pub fn top<S: 'static>(&self) -> Option<usize> {
        for n in (0..self.core.len.get()).rev() {
            if self.core.type_id.borrow()[n] == TypeId::of::<S>() {
                return Some(n);
            }
        }
        None
    }

    /// Find all windows with this type.
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

    /// Get a reference to the window at index n.
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

    /// Get a mutable reference to the window at index n.
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

    /// Get a mutable reference to the window at index n.
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
                if let Some(state) = (state.as_mut() as &mut dyn Any).downcast_mut::<S>() {
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

    /// Get a reference to the window at index n.
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
                if let Some(state) = (state.as_ref() as &dyn Any).downcast_ref::<S>() {
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

/// Handle events from top to bottom.
///
/// Panic
///
/// This function is not reentrant, it will panic when called from within it's call-stack.
impl<Event, Context, Error> HandleEvent<Event, &mut Context, Result<WindowControl<Event>, Error>>
    for WindowList<Event, Context, Error>
where
    Event: TryAsRef<crossterm::event::Event>,
    Error: 'static,
    Event: 'static,
    Context: 'static,
{
    fn handle(&mut self, event: &Event, ctx: &mut Context) -> Result<WindowControl<Event>, Error> {
        let mut r_front = WindowControl::Continue;

        for n in (0..self.core.len.get()).rev() {
            let (to_front, area) = {
                let state = self.core.state.borrow();
                let state = state[n].as_ref().expect("state is gone");
                match event.try_as_ref() {
                    Some(ct_event!(mouse down Left for x,y))
                        if state.area().contains((*x, *y).into()) =>
                    {
                        (true, state.area())
                    }
                    _ => (false, state.area()),
                }
            };

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

            if to_front {
                self.to_front(n, ctx);
                r_front = WindowControl::Changed;
            };

            match r {
                Ok(r) => match r {
                    WindowControl::Close(_) => {
                        self.close(n, ctx);
                        return Ok(max(r, r_front));
                    }
                    WindowControl::Event(_) => {
                        return Ok(max(r, r_front));
                    }
                    WindowControl::Unchanged => {
                        return Ok(max(r, r_front));
                    }
                    WindowControl::Changed => {
                        return Ok(max(r, r_front));
                    }
                    WindowControl::Continue => match event.try_as_ref() {
                        Some(event) => {
                            if mouse_trap(event, area).is_consumed() {
                                return Ok(max(WindowControl::Unchanged, r_front));
                            }
                        }
                        _ => {}
                    },
                },
                Err(e) => return Err(e),
            }
        }

        Ok(r_front)
    }
}

fn max<Event>(
    primary: WindowControl<Event>,
    secondary: WindowControl<Event>,
) -> WindowControl<Event> {
    let s = match &primary {
        WindowControl::Continue => 0,
        WindowControl::Unchanged => 1,
        WindowControl::Changed => 2,
        WindowControl::Event(_) => 3,
        WindowControl::Close(_) => 4,
    };
    let t = match &secondary {
        WindowControl::Continue => 0,
        WindowControl::Unchanged => 1,
        WindowControl::Changed => 2,
        WindowControl::Event(_) => 3,
        WindowControl::Close(_) => 4,
    };
    if s > t { primary } else { secondary }
}

/// Handle events from top to bottom of the stack.
///
/// Panic
///
/// This function is not reentrant, it will panic when called from within it's call-stack.
pub fn handle_window_list<Event, Context, Error>(
    mut window_list: WindowList<Event, Context, Error>,
    event: &Event,
    ctx: &mut Context,
) -> Result<WindowControl<Event>, Error>
where
    Event: TryAsRef<crossterm::event::Event>,
    Error: 'static,
    Event: 'static,
    Error: Debug,
    Event: Debug,
    Context: 'static,
{
    window_list.handle(event, ctx)
}
