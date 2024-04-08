use crate::lib_test::{Timing, Unit};
use rat_salsa::number::core::parse_number_format;
use rat_salsa::number::fmt_to;
use std::fmt::Write;

mod lib_test;

#[test]
fn bench_num() -> Result<(), anyhow::Error> {
    let mut t = Timing::default()
        .skip(10)
        .runs(100000)
        .unit(Unit::Microsecond);

    let mut out = String::new();
    t.run_proc("std", || {
        let _s = write!(&mut out, "{:.16}", rand::random::<f64>());
        out.clear();
    });

    let mut ryu = ryu::Buffer::new();
    t.run_proc("ryu", || {
        _ = ryu.format(rand::random::<f64>());
    });

    let fmt = parse_number_format("0.################")?;
    let mut out = String::new();
    t.run_proc("fmt", || {
        fmt_to(rand::random::<f64>(), &fmt, &mut out);
        out.clear();
    });

    println!("{}", t);

    Ok(())
}
