# 2.4.0

* add example palette_edit

* add wrapper for FileDialog and MessageDialog to use with DialogStack.

* handle_focus() now returns the Outcome.

# 2.3.1

* minor fixes and documentation

# 2.3.0

* feature: Add support for autonomous dialog windows.

  This copies part of the work done in rat-dialog to rat-salsa itself.
  It is behind the feature 'dialog' for now.

  It is not included in the SalsaContext for now, I couldn't get
  the type parameters working satisfactorily. You have to include
  DialogStack into your Global state manually and call rendering
  and event-handling for the dialogs too.

  But it is integrated into the Control-enum and doesn't need
  special handling any longer.

## rat-theme4

New theme crate rat-theme4. Not using defined functions or a trait any longer.
Just a map style-name -> style. But with some niceties that make life easier.

# 2.2.0

* move render/event timing into the framework.
  accessible as SalsaContext::last_event() and SalsaContext::last_render().
* move main templates to "/templates"
* use StatusLineStacked as standard status-line.

# 2.1.2

* upgrade rat-text to 2.0. only for examples.

# 2.1.1

* feature: give access to Terminal via the SalsaContext.
* perf: Filter out double Control::Change. No need to render twice.

* update examples

# 2.1.0

break: extend Terminal trait to mirror ratatui::Terminal.

feature: add SalsaContext::clear_terminal() and SalsaContext::insert_before().
feature: add RunConfig::manual_mode() to do custom init/shutdown of the terminal.

# 2.0.2

* fix: for linux

# 2.0.1

* fix: for linux

# 2.0.0

Remove AppWidget and AppState and replace it with plain functions.
Prime example for traits rooted in old habits and too much object-oriented
thinking.

- break: remove the difference between AppContext and RenderContext.
- break: remove AppContext as a separate unit and make it part of the Global
  struct of the application. This removes the obnoxious '.g' when accessing
  global data and brings AppContext and Global to the same level.
  Renamed to SalsaAppContext and add a trait SalsaContext that can do
  everything the current AppContext can. Easy to plug it to the Global
  struct this way. And less lifetime annotations this way.
- break: simplify spawn() and add spawn_ext() with the full functionality.
- break: rename focus_event() to handle_focus()

- book: there is a new one.
- feature: add a Liveness flag that can track a background task. Not
  very useful for short-lived task, but if you want to run a permanent
  background worker it might be useful.
- feature: add mock::init() and mock::error() to use with run_tui() if
  you don't need init or error handling.
- feature: add PollQuit that will send a message immediately before
  quitting. If you return anything but Quit it will cancel the quit.
- feature: remove the Send bound where it is not absolutely necessary.
  Now your Event and Error type don't need to be Send if you don't
  use thread-tasks or future-tasks.

- feature: really minimize down the minimal.rs example. And add a
  nominal.rs that shows some internal structuring.
- feature: make ultra.rs nicer. and still <= 100loc
-

- fix: don't use all of rat-widget, just rat-event and rat-focus are needed.

# 1.0.1

concerns only the examples ...

* fix: clean unused crates
* fix: replace directories-next with dirs.
* fix: upgrade to rat-theme2

# 1.0.0

... jump ...

# 0.32.2

# 0.32.1

* bump version of rat-widget

# 0.32.0

* BREAK: radically simplify PollEvents traits.
    * Move any configuration out to the PollEvents impls.
    * read_exec transformed to simple read which produces an event
      without the need to send it to the application directly.
    * Control::Event can take over that part now.
* break: Rename Control::Message to Control::Event to better match the new semantics.
* break: The Pollxxx structs now all do their own initialization.
* break: RunConfig has less type parameters now :)

* feature: add AppContext::focus_event(). runs the focus related-handling.
* examples: some fixes.

## Related work

* Choice widget popup handling.
* Rebuild of the calendar widget. It's grown up now.

# 0.31.0

* BREAK: module structure work-over. No changed functionality but
  import paths changed. PollEvent impls moved to their own poll module.
  Makes docs more readable.

# 0.30.2

* update rat-widget

# 0.30.1

* moved all rat-crates to one repo

# 0.30.0

* feature: Add RenderedEvent. This can be activated by adding
  PollRendered and will send one event after each successful render.
* feature: Add an optional async runtime. Currently, tokio.
  Start async tasks via the AppContext and get the result as event.

# 0.29.0

* major change: Replaced the current Message type variable with Event.

  This offloads the complete event-type to the application.

    * event distribution goes only through one channel now,
      the newly added `event()` method. This makes it easier not
      to miss some event-forwarding to different parts of the app.
    * crossterm goes from 'requirement' to 'supported'.
      There still is `PollCrossterm`, but it can be replaced with
      something else entirely.
    * A custom Pollxx can be added and distribute events via the standard `event()`.
      It can require support for its own events with trait bounds.
      Implementing one stays the same, and if configuration is needed
      it still has to go in the Global struct and has to be shared
      with the Pollxx.
    * Timers and ThreadPool can be deactivated completely.
      They are still kept in the AppContext and panic when they are
      accessed and not initialized but otherwise are simply absent.

  The change was surprisingly easy though. Most of the changes come from the optional
  timer and threading. The AppState trait is much cleaner now. It retains init()
  and gains the lost shutdown(). It is probably the best to keep those out of regular
  event-handling. The error() handling functions stays too, but that's unused for
  everything but the main AppState anyway.

  *** Upgrading ***
  Is copypasta more or less. Move the current crossterm(), message() and timer()
  functions out of the trait impl and call them from event(). Or inline their
  contents.

* fix: Key Release events are not universal. Set a static flag during
  terminal init to allow widgets to query this behaviour.

# 0.28.2

* doc fixes

# 0.28.1

* remove unnecessary Debug bounds everywhere.

# 0.28.0

** upgrade to ratatui 0.29 **

* examples changed due to upstream changes.

# 0.27.2

* fix: upstream changes

# 0.27.1

* docs: small fixes

# 0.27.0

* break: Make Control non_exhaustive.

* feature: Change the sleep strategy. Longer idle sleeps and separate
  and faster backoff after changing to fast sleeps.

# 0.26.0

* break: final renames in rat-focus.

# 0.25.6

* fix: update some docs.

# 0.25.5

* fix: docs

# 0.25.4

* fix: changes in rat-menu affect the examples.

# 0.25.2 and 0.25.3

fixed some docs

# 0.25.1

* mention the book.

# 0.25.0

Sync version for beta.

* feat: write rsbook.
* feat: Replace all conversions OutcomeXX to Control with
  one `From<Into<Outcome>>`. All OutcomeXX should be convertible to
  base Outcome anyway.
* refactor: Cancel is now an Arc<AtomicBool>.
* fix: Define Ord for Control without using Message.
* example: Add life.rs
* example: Add turbo.rs

# 0.24.2

* minor fixes for examples/mdedit
* add gifs but don't publish them.

# 0.24.1

* extensive work at the mdedit example. might even publish this
  separately sometime.

* cleanup minimal example. make it more minimal.

# 0.24.0

* update ratatui to 0.28

# 0.23.0

* Start example mdedit.
* Update examples files.

* break: remove timeout from AppContext and add to Terminal::render() instead.
* break: rename AppEvents to AppState to be more in sync with ratatui.

* feature: add replace_timer() to both contexts.
* feature: addd set_screen_cursor() to RenderContext.
* fix: Timer must use next from TimerDef if it exists.

# 0.22.2

* refactor: adaptations for new internal Scroll<'a> instead of Scrolled widget.

# 0.22.1

* add files.rs example
* DarkTheme adds methods to create styles directly from scheme colors.

# 0.22.0

* Restart the loop once more:
    * Remove RepaintEvent. Move the TimeOut to RenderContext instead.
    * Add trait EventPoll. This abstracts away all event-handling,
      and allows adding custom event-sources.
        * Implement PollTimers, PollCrossterm, PollTasks this way,
          and just make those the default set.
    * Add trait Terminal. This encapsulates the ratatui::Terminal.
        * Make the terminal init sequences customizable.
        * Allows other Backends, while not adding to the type variables.
    * Extend RunConfig with
        * render - RenderUI impl
        * events - List of EventPoll.
    * Remove functions add_timer(), remove_timer(), spawn() and queue() from RenderContext.
      This is not needed while rendering.

# 0.21.2

* refactor: AppWidget::render() removes mut from self parameter.
  this matches better with ratatui. should be good enough usually.
* add theme example

# 0.21.1

Fixed several future problems with ordering the events in the presence
of AppContext::queue(). Changed to use a single queue for external events
and results. External events are only polled again after all internal
results have been processed. This way there is a well-defined order
for the internal results and a guarantee that no external interference
can occur between processing two internal results. Which probably
would provide food for some headaches.

# 0.21.0

Moved everything from rat-salsa2 back to rat-salsa, now that it is no
longer in use.

# 0.20.2

* complete refactor:
    * throw away TuiApp completely. It got fat&ugly lately.
    * Drastically reduce the number of used types, don't need
      Data and Theme, those can go into Global as an implementation detail.

  With everything down to three types Global, Action and Error use them directly.
  Everything is still tied together via AppContext and RenderContext.

* refactor: hide timer in the context structs and add the necessary access
  functions, add and remove.
* refactor: make Timers private and add a TimerHandle for deletion.
* AppContext and RenderContext: queue and tasks need not be public.

# 0.20.1

* Extend tasks with cancellation support.
* Add queue for extra result values from event-handling.
  Used for accurate repaint after focus changes.

* fix missing conversion from ScrollOutcome.
* fix missing conversions for DoubleClickOutcome.

* simplified the internal machinery of event-handling a bit.
  Simpler type variables are a thing.

# 0.15.1

was the wrong crate committed

# 0.20.0

* Split AppWidgets into AppWidgets and AppEvents. One for the
  widget side for render, the other for the state side for all
  event handling. This better aligns with the split seen
  in ratatui stateful widgets.
    - The old mono design goes too much in the direction of a widget tree,
      which is not the intent.
    - It seems that AppWidget now very much mimics the StatefulWidget trait,
      event if that was not the initial goal. Curious.
    - I think I'm quite content with the tree-way split that now exists.
    - I had originally intended to use the rat-event::HandleEvent trait
      instead of some AppEvents, but that proved to limited. It still is
      very fine for creating widgets, that's why I don't want to change
      it anymore. Can live well with this current state.

# 0.19.0

First release that I consider as BETA ready.

* reorg from rat-event down. built in some niceties there.

# 0.18.0

Start from scratch as rat-salsa2. The old rat-salsa now is
mostly demolished and split up in

* rat-event
* rat-focus
* rat-ftable
* rat-input
* rat-scrolled
* rat-widget

and the rest is not deemed worth it. 
