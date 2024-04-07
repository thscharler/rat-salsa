use rat_salsa::number;
use rat_salsa::number::{parse_sym, FormatNumber, NumberFormat, NumberSymbols, ParseNumber};
use std::fmt;
use std::rc::Rc;

#[test]
fn test_format() -> Result<(), fmt::Error> {
    _ = dbg!(number::format(1234, "#####"));
    _ = dbg!(number::format(1234, "####0.00"));
    _ = dbg!(number::format(1234, "###00.##"));
    _ = dbg!(number::format(1234, "#####e###"));
    _ = dbg!(number::format(1.234e14, "###,###,###,###,###f###"));
    _ = dbg!(number::format(
        1.234e-14,
        "###,###,###,###,###.##################f###"
    ));
    _ = dbg!(number::format(1234, "+####"));
    _ = dbg!(number::format(1234, "-####"));
    _ = dbg!(number::format(-1234, "####-"));
    Ok(())
}

#[test]
fn test_fmt() -> Result<(), std::fmt::Error> {
    println!("{}", 32.format("####", &NumberSymbols::new())?);
    println!("{}", 32.23f64.format("0000.00", &NumberSymbols::new())?);
    println!(
        "{}",
        32.23f64.format("0000.00e+000", &NumberSymbols::new())?
    );
    println!("{}", 32.23f64.format("###0.00e888", &NumberSymbols::new())?);
    println!(
        "{}",
        0.003223f64.format("###0.00e888", &NumberSymbols::new())?
    );

    Ok(())
}

#[test]
fn test_parse() {
    println!("{:?}", parse_sym::<f32>("111", &Default::default()));
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

    println!("{}", 112.format("$ ###0", &sym)?);
    println!("{}", 112.format("$ ###0", &sym2)?);

    let fmt = NumberFormat::news("$ ###0", &sym)?;
    println!("{:?}", "€  112".parse_fmt::<f64>(&fmt));

    let fmt2 = NumberFormat::news("$ ###0", &sym2)?;
    println!("{:?}", "Rub  112".parse_fmt::<f64>(&fmt2));

    Ok(())
}
