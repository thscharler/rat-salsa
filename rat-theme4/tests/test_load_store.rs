use rat_theme4::palettes::dark::IMPERIAL;
use rat_theme4::{load_palette, store_palette};
use std::borrow::Cow;
use std::fs::{File, create_dir_all};

#[test]
fn store_g() {
    let mut p = IMPERIAL;

    create_dir_all("tmp").expect("tmp");

    p.generator = Cow::Borrowed("color-1");
    store_palette(&p, File::create("tmp/color-1.pal").expect("file")).expect("fine");
    p.generator = Cow::Borrowed("color-2");
    store_palette(&p, File::create("tmp/color-2.pal").expect("file")).expect("fine");
    p.generator = Cow::Borrowed("color-4");
    store_palette(&p, File::create("tmp/color-4.pal").expect("file")).expect("fine");
    p.generator = Cow::Borrowed("color-4-dark:63");
    store_palette(&p, File::create("tmp/color-4-dark.pal").expect("file")).expect("fine");
    p.generator = Cow::Borrowed("color-8");
    store_palette(&p, File::create("tmp/color-8.pal").expect("file")).expect("fine");

    dbg!(load_palette(File::open("tmp/color-1.pal").expect("file")).expect("fine"));
    dbg!(load_palette(File::open("tmp/color-2.pal").expect("file")).expect("fine"));
    dbg!(load_palette(File::open("tmp/color-4.pal").expect("file")).expect("fine"));
    dbg!(load_palette(File::open("tmp/color-4-dark.pal").expect("file")).expect("fine"));
    dbg!(load_palette(File::open("tmp/color-8.pal").expect("file")).expect("fine"));
}
