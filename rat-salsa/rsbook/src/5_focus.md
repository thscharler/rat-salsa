# Focus

You can look up [Focus][refFocus] for yourselves. 

# Foundation

FocusFlag sits at the core of it all. It's the bit of state shared between
your widgets and the Focus system. 

The Focus system sees the world as a list of FocusFlags and some parameters 
how to treat each of them. 

Your widget sees 'Do I have it?'. 

# Widget trees

ratatui doesn't have a widget tree. 

So [FocusBuilder][refFocusBuilder] takes over, walks through
everything that [HasFocus][refHasFocus] and builds up one it
can use.

This has been decently optimized and usually takes a couple of
microseconds so it can be done with every event.


[refFocus]: https://docs.rs/rat-focus/latest/rat_focus/struct.Focus.html
[refHasFocus]: https://docs.rs/rat-focus/latest/rat_focus/trait.HasFocus.html
[refFocusBuilder]: https://docs.rs/rat-focus/latest/rat_focus/struct.FocusBuilder.html
