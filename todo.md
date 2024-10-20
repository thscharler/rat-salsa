* Clipper: analog to pager, but with scroll moving window?
    * own Buffer for temp rendering
    * resizing the buffer if necessary?
    * pre-sizing the buffer bigger and blobbing.
    * as an alternative to View and Viewport.

---

If you are considering profound breaking changes here too, I want
to add:

* Make Rect and alike use __i32__ for the indexes. This would make
  it possible to partially scroll widgets left/top off-screen.

This can be partially emulated by creating a temporary Buffer and
shifting it during copying. This works reasonably well for Widgets,
but becomes unfeasable for StatefulWidgets that want to store areas
for later interactions. Without very detailed knowledge of the
internals of each such widget its almost impossible to track what
the widget will do with the area given to render. As a result
matching up a mouse position with the widget becomes hard to
impossible.

* If i32 is too big, i16 might do as well. 

