# RenderContext

```
pub struct RenderContext<'a, Global> {
    /// Some global state for the application.
    pub g: &'a mut Global,
   
    /// Frame counter.
    pub count: usize,
   
    /// Output cursor position. Set after rendering is complete.
    pub cursor: Option<(u16, u16)>,
}
```


