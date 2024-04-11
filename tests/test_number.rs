use pure_rust_locales::Locale;
use rat_salsa::number;
use rat_salsa::number::{parse_sym, FormatNumber, NumberFormat, NumberSymbols, ParseNumber};
use std::fmt;
use std::rc::Rc;

#[test]
fn test_format_struct() -> Result<(), fmt::Error> {
    let fmt = NumberFormat::new("####")?;
    let mut str = String::new();
    number::core::map_num("-.", &fmt, fmt.sym(), &mut str)?;
    assert_eq!(str, "   -");

    assert_eq!(number::format(-1, "####")?, "  -1");
    assert_eq!(number::format(-1, "###0")?, "  -1");
    assert_eq!(number::format(-1, "##00")?, " -01");
    assert_eq!(number::format(-1, "#000")?, "-001");

    Ok(())
}

#[test]
fn test_format() -> Result<(), fmt::Error> {
    assert_eq!(number::format(1234, "#####")?, " 1234");
    assert_eq!(number::format(1234, "####0.00")?, " 1234.00");
    assert_eq!(number::format(1234, "###00.##")?, " 1234   ");
    assert_eq!(number::format(1234, "#####e###")?, "    1e  3");
    assert_eq!(
        number::format(1.234e14, "###,###,###,###,###f###")?,
        "                  1e 14"
    );
    assert_eq!(
        number::format(1.234e-14, "###,###,###,###,###.##################f###")?,
        "                  1.233999999999999936e-14"
    );
    assert_eq!(number::format(1234, "+####")?, "+1234");
    assert_eq!(number::format(1234, "-####")?, " 1234");
    assert_eq!(number::format(-1234, "#####")?, "-1234");
    assert_eq!(number::format(-1234, "####-")?, "1234-");
    assert_eq!(number::format(-1234, "#####-")?, " 1234-");
    assert_eq!(number::format(-1, "###00")?, "  -01");
    Ok(())
}

#[test]
fn test_fmt() -> Result<(), std::fmt::Error> {
    assert_eq!(
        format!("{}", 32.format("####", &NumberSymbols::new())?).to_string(),
        "  32"
    );
    assert_eq!(
        format!("{}", 32.23f64.format("0000.00", &NumberSymbols::new())?).to_string(),
        "0032.23"
    );
    assert_eq!(
        format!(
            "{}",
            32.23f64.format("0000.00e+000", &NumberSymbols::new())?
        )
        .to_string(),
        "0003.22e+001"
    );
    assert_eq!(
        format!("{}", 32.23f64.format("###0.00e888", &NumberSymbols::new())?).to_string(),
        "   3.22e1"
    );
    assert_eq!(
        format!(
            "{}",
            0.003223f64.format("###0.00e888", &NumberSymbols::new())?
        )
        .to_string(),
        "   3.22e-3"
    );

    Ok(())
}

#[test]
fn test_parse() {
    assert_eq!(parse_sym::<f32>("111", &Default::default()), Ok(111f32));
}

#[test]
fn test_currency() -> Result<(), std::fmt::Error> {
    let sym = Rc::new(NumberSymbols {
        decimal_sep: ',',
        decimal_grp: '.',
        currency_sym: "€".into(),
        ..Default::default()
    });

    let sym2 = Rc::new(NumberSymbols {
        decimal_sep: ',',
        decimal_grp: '.',
        currency_sym: "Rub".into(),
        ..Default::default()
    });

    assert_eq!(number::formats(112, "$ ###0", &sym)?, "€  112");
    assert_eq!(number::formats(112, "$ ###0", &sym2)?, "Rub  112");

    let fmt = NumberFormat::news("$ ###0", &sym)?;
    assert_eq!(number::parse_fmt("€  112", &fmt), Ok(112));
    let fmt2 = NumberFormat::news("$ ###0", &sym2)?;
    assert_eq!(number::parse_fmt("Rub  112", &fmt2), Ok(112));

    Ok(())
}

#[test]
fn test_loc() {
    dbg!(NumberSymbols::monetary(Locale::es_ES));
}
