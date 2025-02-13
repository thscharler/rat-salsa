# AppWidget


```
pub trait AppWidget<Global, Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    /// Type of the State.
    type State: AppState<Global, Event, Error> + ?Sized;

    /// Renders an application widget.
    fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut Self::State,
        ctx: &mut RenderContext<'_, Global>,
    ) -> Result<(), Error>;
}
```





