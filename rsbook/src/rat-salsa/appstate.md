# AppState

AppState has some lifecycle functions:

- init: initialize before rendering the first frame.
- shutdown: shutdown on exit.

Error handling:

- error: is called for every error from rendering and event-handling.


Event handling:

- event: is called for every event. returns one of the Control values.

