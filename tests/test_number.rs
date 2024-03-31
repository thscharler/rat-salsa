use rat_salsa::number::{fmt_f64, parse_format};
use std::fmt;

#[test]
fn test_parse() -> Result<(), fmt::Error> {
    _ = dbg!(fmt_f64(1234, "#####", None));
    _ = dbg!(fmt_f64(1234, "####0.00", None));
    _ = dbg!(fmt_f64(1234, "###00.##", None));
    _ = dbg!(fmt_f64(1234, "#####e###", None));
    _ = dbg!(fmt_f64(1.234e14, "###,###,###,###,###f###", None));
    _ = dbg!(fmt_f64(
        1.234e-14,
        "###,###,###,###,###.##################f###",
        None
    ));
    _ = dbg!(fmt_f64(1234, "+####", None));
    _ = dbg!(fmt_f64(1234, "-####", None));
    _ = dbg!(fmt_f64(-1234, "####-", None));
    Ok(())
}
