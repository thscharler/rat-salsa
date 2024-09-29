
# Scroll

[Scroll](Scroll) adds support for widgets that want to scroll their
content.

Scroll works analogous to Block, it can be set on the widget
struct where it is supported. The widget can decide which 
scrolling it can do, horizontal vertical or both.

# Adding scroll to a widget

- Add [Scroll](Scroll) to the widget struct.
- Use [ScrollArea](ScrollArea) for the layout and rendering of
  the combination of Block+Scroll+Scroll your widget supports.
- Add [ScrollState](ScrollState) to the widget state struct. 
- Create a [ScrollAreaState](ScrollAreaState) for event-handling.
