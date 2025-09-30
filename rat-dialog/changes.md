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
