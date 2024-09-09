# Text-widgets for ratatui

Features for all widgets:

* Undo/redo.
  
* Sync another widget.
  
* Support double-width characters.
  
* Range based text styling.
  
* Clipboard trait to link to some clipboard implementation.
  
  > There is no general solution for clipboards but this way you
  > can integrate one of the many crates that try to do this.
  
* Builtin scrolling. Uses rat-scroll.

* Lot's of text manipulation functions. 
  * Line/TextRange/byte indexes supported.
  * Glyph iteration (glyph=grapheme+display information)
  * Grapheme iterator/cursor
  * byte-pos to TextPosition and vice versa.
  * Screen position to text position.
  
  
## TextInput

Single line text input widget. 

## TextArea

Textarea with tendencies to being an editor.

Uses [Rope](https://docs.rs/ropey/) backend for a good editing
experience for longer text. Starts lagging a bit if you have
more than 10,000 style ranges or so (wip). 

* Tab width/Tab expand to space.
* Indent/dedent selection.
* Newline starts at indent.
* Mouse selection can work word-wise. 
* Renders this text in ~400µs
* Quote/Brace/Bracket selection.

There is an extended example `mdedit.rs` for TextArea in 
<https://github.com/thscharler/rat-salsa>


## MaskedInput

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

## DateInput

DateInput with [chrono](https://docs.rs/chrono) format patterns.

## NumberInput

NumberInput with
[format_num_pattern](https://docs.rs/format_num_pattern) backend.
A bit similar to javas DecimalFormat.

## LineNumbers

Line numbers widget that can be combined with TextArea. 



























