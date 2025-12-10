use anyhow::Error;
use rat_theme4::palette::Palette;
use rat_theme4::palettes::dark::IMPERIAL;
use std::borrow::Cow;
use std::fs;
use std::fs::create_dir_all;

#[test]
fn write_palette() -> Result<(), Error> {
    let p = IMPERIAL;
    let json = serde_json::to_string_pretty(&p)?;
    eprintln!("{}", json);
    let pp = serde_json::from_str(&json)?;
    assert_eq!(p, pp);
    Ok(())
}

#[test]
fn store_g() {
    let mut p = IMPERIAL;

    create_dir_all("tmp").expect("tmp");

    p.generator = Cow::Borrowed("color-1");
    fs::write(
        "tmp/color-1.json",
        serde_json::to_string_pretty(&p).expect("json"),
    )
    .expect("fine");

    p.generator = Cow::Borrowed("color-2");
    fs::write(
        "tmp/color-2.json",
        serde_json::to_string_pretty(&p).expect("json"),
    )
    .expect("fine");

    p.generator = Cow::Borrowed("color-4");
    fs::write(
        "tmp/color-4.json",
        serde_json::to_string_pretty(&p).expect("json"),
    )
    .expect("fine");

    p.generator = Cow::Borrowed("color-4-dark:63");
    fs::write(
        "tmp/color-4-dark.json",
        serde_json::to_string_pretty(&p).expect("json"),
    )
    .expect("fine");

    p.generator = Cow::Borrowed("color-8");
    fs::write(
        "tmp/color-8.json",
        serde_json::to_string_pretty(&p).expect("json"),
    )
    .expect("fine");

    let json = fs::read_to_string("tmp/color-1.json").expect("fine");
    dbg!(serde_json::from_str::<Palette>(&json).expect("json"));
    let json = fs::read_to_string("tmp/color-2.json").expect("fine");
    dbg!(serde_json::from_str::<Palette>(&json).expect("json"));
    let json = fs::read_to_string("tmp/color-4.json").expect("fine");
    dbg!(serde_json::from_str::<Palette>(&json).expect("json"));
    let json = fs::read_to_string("tmp/color-4-dark.json").expect("fine");
    dbg!(serde_json::from_str::<Palette>(&json).expect("json"));
    let json = fs::read_to_string("tmp/color-8.json").expect("fine");
    dbg!(serde_json::from_str::<Palette>(&json).expect("json"));
}
