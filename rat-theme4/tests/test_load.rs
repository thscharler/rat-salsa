use rat_theme4::load_palette;

#[test]
fn test_load() {
    let b = include_bytes!("everforest.pal");

    match load_palette(b.as_slice()) {
        Ok(p) => {
            dbg!(p);
        }
        Err(e) => {
            dbg!(e);
            panic!()
        }
    };
}
