use anyhow::Error;
use rat_theme4::palettes::dark::IMPERIAL;

#[test]
fn write_palette() -> Result<(), Error> {
    let p = IMPERIAL;
    let json = serde_json::to_string_pretty(&p)?;
    eprintln!("{}", json);
    let pp = serde_json::from_str(&json)?;
    assert_eq!(p, pp);
    Ok(())
}
