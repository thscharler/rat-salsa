![semver](https://img.shields.io/badge/semver-☑-FFD700)
![stable](https://img.shields.io/badge/stability-stable-8A2BE2)
[![crates.io](https://img.shields.io/crates/v/rat-text.svg)](https://crates.io/crates/rat-text)
[![Documentation](https://docs.rs/rat-text/badge.svg)](https://docs.rs/rat-text)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
![](https://tokei.rs/b1/github/thscharler/rat-salsa)

This crate is a part of [rat-salsa][refRatSalsa].

For examples see [rat-text GitHub][refGitHubText] or an extended example `mdedit.rs` in
[rat-salsa GitHub][refGitHubSalsa].

* [Changes](https://github.com/thscharler/rat-salsa/blob/master/rat-text/changes.md)

# Text widgets for ratatui

Features for all widgets:

* Undo/redo
* Sync another widget
* Support double-width characters
* Range based text styling
* Wrapped text
* Clipboard trait to link to some clipboard implementation.

  > There is no general solution for clipboards but this way you
  > can integrate one of the many crates that try to do this.

* Builtin scrolling. Uses [rat-scrolled][refRatScrolled].

* Lots of text manipulation functions.
    * Line/TextRange/byte indexes supported.
    * Grapheme iterator/cursor
    * byte-pos to TextPosition and vice versa.
    * Screen position to text position.

## [TextInput](crate::text_input::TextInput)

Single line text input widget.

```ignore
// actual text is held in the state:
state.textinput.set_value("sample");

// render
TextInput::new()
    .style(Style::default().white().on_dark_gray())
    .select_style(Style::default().black().on_yellow())
    .render(txt_area, frame.buffer_mut(), &mut state.textinput);
    
// during event-handling
match text_input::handle_events(&mut state.textinput, true /*focused*/, event) {
    TextOutcome::Continue => { /* no handling */ }
    TextOutcome::Unchanged => { /* event recognized, but no changes */ }
    TextOutcome::Changed => { /* render required */ }
    TextOutcome::TextChanged => { /* actual edit */ }
}
```

## [TextArea](crate::text_area::TextArea)

Textarea with tendencies to being an editor.

Uses [ropey][refRopey] as backend and [iset][refIset] to
manage the text-styles.

* Tab width/Tab expand to space.
* Indent/dedent selection.
* Newline starts at indent.
* Mouse selection can work word-wise.
* Decent speed even for large text (millons of lines and text-width ~100_000 tested).
* Word-wrap mode.
* Add Quotes/Braces/Brackets to selection.

```ignore
// text is stored in the state
state.textarea.set_text("some text");

// render
TextArea::new()
  .block(Block::bordered())
  .vscroll(
      Scroll::new()
          .scroll_by(1)
          .policy(ScrollbarPolicy::Always),
  )
  .hscroll(Scroll::new().policy(ScrollbarPolicy::Always))
  .styles(istate.theme.textarea_style())
  .text_style([
      Style::new().red(),
      Style::new().underlined(),
      Style::new().green(),
      Style::new().on_yellow(),
  ])
  .render(layout[2], frame.buffer_mut(), &mut state.textarea);
  
// event-handling
match text_area::handle_events(&mut state.textarea, true /* focused */, event) {
    TextOutcome::Continue => { /* no handling */ }
    TextOutcome::Unchanged => { /* event recognized, but no changes */ }
    TextOutcome::Changed => { /* render required */ }
    TextOutcome::TextChanged => { /* actual edit */ }
}
```

There is an extended example `mdedit.rs` for TextArea in
[rat-salsa][refRatSalsa]

## [MaskedInput](crate::text_input_mask::MaskedInput)

Single line text input with a text-mask for allowed input.

* Numeric
* Decimal/Hexadecimal/Octal digits
* Character/Character+Digits
* Text separators

Nice to have for structured text input.

The widgets

* DateInput and
* NumberInput

use this as base.

## [DateInput](crate::date_input::DateInput)

DateInput with [chrono][refChrono] format patterns.

## [NumberInput](crate::number_input::NumberInput)

NumberInput with [format_num_pattern][refFormatNumPattern]
backend. A bit similar to javas DecimalFormat.

## [LineNumbers](crate::line_number::LineNumbers)

Line numbers widget that can be combined with TextArea.


[refRatSalsa]: https://docs.rs/rat-salsa/latest/rat_salsa/

[refRatScrolled]: https://docs.rs/rat-scrolled/latest/rat_scrolled/

[refRopey]: https://docs.rs/ropey/

[refIset]: https://docs.rs/iset/

[refChrono]: https://docs.rs/chrono

[refFormatNumPattern]: https://docs.rs/format_num_pattern

[refGitHubText]: https://github.com/thscharler/rat-salsa/blob/master/rat-text/examples

[refGitHubSalsa]: https://github.com/thscharler/rat-salsa/tree/master/examples
