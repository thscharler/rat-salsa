# 0.10.0

Moved the text-widgets from rat-widgets to this crate.
This was not a simple migration, but a start from scratch
with the goal to use one backend for all text-widgets.

This introduces the TextStore trait which acts as backend
for the backend and does the text manipulation and mapping
from graphemes to bytes. There is a String based implementation
which supports only a single line of text and a rope based
implementation for the full textarea.

The api of the widgets stays more or less the same, but
everything is re-implemented based on text-store. 