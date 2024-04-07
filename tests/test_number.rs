use rat_salsa::number;
use rat_salsa::number::{FormatNumber, parse_fmt, parse_sym};
use std::fmt;

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
fn test_fmt() {
    println!("{}", 32.format("####"));
    println!("{}", 32.23f64.format("0000.00"));
    println!("{}", 32.23f64.format("0000.00e+000"));
    println!("{}", 32.23f64.format("###0.00e888"));
    println!("{}", 0.003223f64.format("###0.00e888"));
}

#[test]
fn test_parse() {
    println!("{:?}", parse_sym("111", Default::default()))

    println!("{:?}", parse_fmt())
}
