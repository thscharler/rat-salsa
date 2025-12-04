use anyhow::Error;
use rat_theme3::palettes::IMPERIAL;
use rat_theme3::{DarkTheme, create_theme};

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
fn write_theme() -> Result<(), Error> {
    let t = DarkTheme::new("Imperial Dark", IMPERIAL);
    let json = serde_json::to_string_pretty(&t)?;
    eprintln!("{}", json);
    let tt = serde_json::from_str(&json)?;
    assert_eq!(t, tt);
    Ok(())
}
