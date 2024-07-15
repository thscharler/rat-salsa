# Rendering overlays

ratatui itself has no builtin facilities for widgets that render
as overlay over other widgets.

For widgets that are only rendered as overlay, the solution
is straight forward: render them after all widgets that
should be below have been rendered.

That leaves widget that are only partial overlays, such as Menubar
and Split. They solve this, by not implementing any widget trait,
instead they act as widget-builders, and have a method `into_widgets()`
that return two widgets. One for the base-rendering and one for
the popup. Only those are ratatui-widgets, and they have no
further configuration methods.

        The advantage of this is, that there is only one configuration
        necessary. The method into_widgets can distribute the 
        parameters to both as necessary. 


