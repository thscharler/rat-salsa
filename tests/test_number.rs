use rat_salsa::number;
use rat_salsa::number::{parse_sym, FormatNumber, NumberFmtError, NumberFormat, NumberSymbols};
use std::fmt;
use std::rc::Rc;

#[test]
fn test_extra() -> Result<(), fmt::Error> {
    dbg!(number::format(1234, "#\\-#\\-#\\-#\\-#\\-#"));

    Ok(())
}

#[test]
fn test_fail() {
    assert_eq!(
        NumberFormat::new("##0.00.00"),
        Err(NumberFmtError::ParseInvalidDecimalSep)
    );
    assert_eq!(
        NumberFormat::new("##0e00e00"),
        Err(NumberFmtError::ParseInvalidExp)
    );
}

#[test]
fn test_exp() -> Result<(), fmt::Error> {
    assert_eq!(
        number::format(1.234e14, "###,###,###,###,###e###"),
        Ok("                  1e14 ".to_string())
    );

    let fmt = NumberFormat::new("###e##00").unwrap();
    assert_eq!(fmt.fmt(1), Ok("  1e00  ".to_string()));
    assert_eq!(fmt.fmt(1e1), Ok("  1e01  ".to_string()));
    assert_eq!(fmt.fmt(1e-1), Ok("  1e-01 ".to_string()));
    assert_eq!(fmt.fmt(1e12), Ok("  1e12  ".to_string()));
    assert_eq!(fmt.fmt(1e-12), Ok("  1e-12 ".to_string()));

    let fmt = NumberFormat::new("###e###0").unwrap();
    assert_eq!(fmt.fmt(1), Ok("  1e0   ".to_string()));
    assert_eq!(fmt.fmt(1e1), Ok("  1e1   ".to_string()));
    assert_eq!(fmt.fmt(1e-1), Ok("  1e-1  ".to_string()));
    assert_eq!(fmt.fmt(1e12), Ok("  1e12  ".to_string()));
    assert_eq!(fmt.fmt(1e-12), Ok("  1e-12 ".to_string()));

    let fmt = NumberFormat::new("###e####").unwrap();
    assert_eq!(fmt.fmt(1), Ok("  1e0   ".to_string()));
    assert_eq!(fmt.fmt(1e1), Ok("  1e1   ".to_string()));
    assert_eq!(fmt.fmt(1e-1), Ok("  1e-1  ".to_string()));
    assert_eq!(fmt.fmt(1e12), Ok("  1e12  ".to_string()));
    assert_eq!(fmt.fmt(1e-12), Ok("  1e-12 ".to_string()));

    let fmt = NumberFormat::new("###e##").unwrap();
    assert_eq!(fmt.fmt(1), Ok("  1e0 ".to_string()));
    assert_eq!(fmt.fmt(1e1), Ok("  1e1 ".to_string()));
    assert_eq!(fmt.fmt(1e-1), Ok("  1e-1".to_string()));
    assert_eq!(fmt.fmt(1e12), Ok("  1e12".to_string()));
    assert_eq!(fmt.fmt(1e-12), Err(NumberFmtError::FmtLenExp));
    Ok(())
}

#[test]
fn test_grouping() {
    assert_eq!(number::format(-123, "##,###"), Ok("  -123".to_string()));
    assert_eq!(number::format(-1234, "##,###"), Ok("-1,234".to_string()));
}

#[test]
fn test_sign() {
    let fmt = NumberFormat::new("####").expect("x");
    let mut str = String::new();
    number::core::map_num::<_, false>("-.", &fmt, fmt.sym(), &mut str).expect("x");
    assert_eq!(str, "   -");

    assert_eq!(number::format(-1, "####"), Ok("  -1".to_string()));
    assert_eq!(number::format(-1, "###0"), Ok("  -1".to_string()));
    assert_eq!(number::format(-1, "##00"), Ok(" -01".to_string()));
    assert_eq!(number::format(-1, "#000"), Ok("-001".to_string()));
}

#[test]
fn test_format() {
    assert_eq!(number::format(1234, "#####"), Ok(" 1234".to_string()));
    assert_eq!(number::format(1234, "####0.00"), Ok(" 1234.00".to_string()));
    assert_eq!(number::format(1234, "###00.##"), Ok(" 1234   ".to_string()));
    assert_eq!(
        number::format(1234, "#####e###"),
        Ok("    1e3  ".to_string())
    );
    assert_eq!(
        number::format(1.234e14, "###,###,###,###,###e###"),
        Ok("                  1e14 ".to_string())
    );
    assert_eq!(
        number::format(1.234e-14, "###,###,###,###,###.##################e###"),
        Ok("                  1.233999999999999936e-14".to_string())
    );
    assert_eq!(number::format(1234, "-####"), Ok(" 1234".to_string()));
    assert_eq!(number::format(-1234, "#####"), Ok("-1234".to_string()));
    assert_eq!(number::format(-1234, "####-"), Ok("1234-".to_string()));
    assert_eq!(number::format(-1234, "#####-"), Ok(" 1234-".to_string()));
    assert_eq!(number::format(-1, "###00"), Ok("  -01".to_string()));
}

#[test]
fn test_fmt() {
    assert_eq!(
        format!("{}", 32.format("####", &NumberSymbols::new()).expect("x")).to_string(),
        "  32"
    );
    assert_eq!(
        format!(
            "{}",
            32.23f64
                .format("0000.00", &NumberSymbols::new())
                .expect("x")
        )
        .to_string(),
        "0032.23"
    );
    assert_eq!(
        format!(
            "{}",
            32.23f64
                .format("0000.00e-000", &NumberSymbols::new())
                .expect("x")
        )
        .to_string(),
        "0003.22e+001"
    );
    assert_eq!(
        format!(
            "{}",
            32.23f64
                .format("###0.00e###", &NumberSymbols::new())
                .expect("x")
        )
        .to_string(),
        "   3.22e1  "
    );
    assert_eq!(
        format!(
            "{}",
            0.003223f64
                .format("###0.00e###", &NumberSymbols::new())
                .expect("x")
        )
        .to_string(),
        "   3.22e-3 "
    );
}

#[test]
fn test_parse() {
    assert_eq!(parse_sym::<f32>("111", &Default::default()), Ok(111f32));
}

#[test]
fn test_currency() {
    let sym = Rc::new(NumberSymbols {
        decimal_sep: ',',
        decimal_grp: Some('.'),
        currency_sym: "€".into(),
        ..Default::default()
    });

    let sym2 = Rc::new(NumberSymbols {
        decimal_sep: ',',
        decimal_grp: Some('.'),
        currency_sym: "Rub".into(),
        ..Default::default()
    });

    assert_eq!(
        number::formats(112, "$ ###0", &sym),
        Ok("€  112".to_string())
    );
    assert_eq!(
        number::formats(112, "$ ###0", &sym2),
        Ok("Rub  112".to_string())
    );

    let fmt = NumberFormat::news("$ ###0", &sym).expect("x");
    assert_eq!(number::parse_fmt("€  112", &fmt), Ok(112));
    let fmt2 = NumberFormat::news("$ ###0", &sym2).expect("x");
    assert_eq!(number::parse_fmt("Rub  112", &fmt2), Ok(112));

    assert_eq!(
        number::parse_fmt::<u32>("Ru  112", &fmt2),
        Err(NumberFmtError::ParseInvalidCurrency)
    );
    assert_eq!(
        number::parse_fmt::<u32>("Ru", &fmt2),
        Err(NumberFmtError::ParseInvalidCurrency)
    );
}
