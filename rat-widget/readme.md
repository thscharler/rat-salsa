![semver](https://img.shields.io/badge/semver-☑-FFD700)
![stable](https://img.shields.io/badge/stability-stable-8A2BE2)
[![crates.io](https://img.shields.io/crates/v/rat-widget.svg)](https://crates.io/crates/rat-widget)
[![Documentation](https://docs.rs/rat-widget/badge.svg)](https://docs.rs/rat-widget)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
![](https://tokei.rs/b1/github/thscharler/rat-salsa)

This crate is a part of [rat-salsa][refRatSalsa].

For examples see [rat-widget GitHub][refGitHubWidget]

* [Changes](https://github.com/thscharler/rat-salsa/blob/master/rat-widget/changes.md)

# Rat Widgets

This crate tries to provide an extended set of widgets with

- *Event handling*
- *Focus handling*
- *Builtin scrolling*
- *Speed*

## Event handling

Uses the trait defined in [rat-event][refRatEvent] to implement
event-handling for crossterm. All widgets are designed with other
event-handlers in mind.

## Focus handling

Uses [FocusFlag][refRatFocusFlag] defined by [rat-focus][refRatFocus]
internally, to mark the focused widget. This is just a passive flag,
that probably can be used with other focus systems. Or you use
[rat-focus][refRatFocus].

## Scrolling

Where it makes sense the widgets implement internal scrolling.
They use [Scroll][refScroll] from [rat-scrolled][refRatScrolled]
to render the scroll bars.

## Speed

When rendering all the widgets try hard to do so without allocations
and extensive copying.

Special mentions:

- [rat-ftable::Table][refRatTable]: It uses an adapter for the
  data for rendering instead of creating Row/Cell structs.

# Widgets

All the widgets are plain ratatui widgets, and implement StatefulWidget.

Event handling uses [rat-event::HandleEvent][refRatEvent].
Currently, crossterm events are implemented.

## Layout

There are some layout calculators beyond ratatui's Layout.

* [layout](layout/index.html)

## Relocation

Widgets like View and Clipper move the widget-image after
rendering. This breaks any areas stored in the widget-states.

See [rat-reloc][refRatReloc]

[refRatSalsa]: https://docs.rs/rat-salsa/latest/rat_salsa/

[refRatEvent]: https://docs.rs/rat-event

[refRatFocus]: https://docs.rs/rat-focus

[refRatReloc]: https://docs.rs/rat-reloc

[refRatFocusFlag]: https://docs.rs/rat-focus/latest/rat_focus/struct.FocusFlag.html

[refScroll]: https://docs.rs/rat-scrolled/latest/rat_scrolled/struct.Scroll.html

[refRatScrolled]: https://docs.rs/rat-scrolled

[refRatTable]: https://docs.rs/rat-ftable

[refRatTextArea]: textarea/index.html

[refGitHubWidget]: https://github.com/thscharler/rat-salsa/blob/master/rat-widget/examples
