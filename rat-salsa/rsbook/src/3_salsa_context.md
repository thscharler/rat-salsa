# SalsaContext

This is designed as an extension trait to your global state. 

It allows run_tui() to set the initialized context and gives you
direct access to its functionality.

Some of it's functions depend on you adding a specific Pollxxx 
to RunConfig and will panic if they can't find it. 

For details see the [documentation](https://docs.rs/rat-salsa/latest/rat_salsa/trait.SalsaContext.html)
