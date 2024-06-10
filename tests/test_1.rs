use rat_salsa::AppContext;

#[test]
fn test_() {
    let a = AppContext {
        g: &mut (),
        ..AppContext::def
    };
}
