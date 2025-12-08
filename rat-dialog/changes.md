# 0.8.1

* fix dependencies

# 0.8.0

* move examples to rat-theme4

# 0.7.2

* sync crate versions

# 0.7.1

* sync crate versions

# 0.7.0

* break: break: Window needs a Context type parameter. Otherwise,
  setting the focus in set_top() is not possible. This adds Context
  to most of the function calls too.
* refactor: MacFrame and WindowFrame now use the same state struct WindowFrameState
* feature: WindowFrame without border.
* feature: add BaseDialog widget. It collects shared behaviour/layout of dialog windows.
* feature: WindowFrame: add no_fill()
* refactor: move decoration widgets to separate module.
* refactor: make DialogStack a StatefulWidget

# 0.6.0

* upgrade rat-text to 2.0.0

# 0.5.2

* add MacFrame

# 0.5.1

* keyboard move/resize

# 0.5.0

* breaks everything.

* simplified api for DialogStack.
* added WindowList for regular windows.
* add WindowFrame widget to render a border and handle move/resize.

- removed dependency on rat-salsa.
- remove all focus related stuff.

# 0.4.0

* break: rename StackControl to DialogControl. rename Pop to Close. add payload to Close.

# 0.3.0

* rebuilt with rat-salsa 2.0

# 0.2.0

* use a render Fn instead of creating and returning an AppWidget.
* intro DialogWidget and DialogState and detach them from
  AppWidget/AppState. They have different bounds and somewhat diverging
  functionality. And less impls necessary.

# 0.1.0

Initial release.
