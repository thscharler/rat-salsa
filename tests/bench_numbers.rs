use crate::lib_test::{Timing, Unit};
use rat_salsa::number::{fmt_to, format_to, NumberFormat};
use std::fmt::Write;

mod lib_test;

#[test]
fn bench_num() -> Result<(), anyhow::Error> {
    let mut t = Timing::default()
        .skip(10)
        .runs(1000000)
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

    let fmt = NumberFormat::new("0.################")?;
    let mut out = String::new();
    t.run_proc("fmt", || {
        fmt_to(rand::random::<f64>(), &fmt, &mut out);
        out.clear();
    });

    let mut out = String::new();
    t.run_proc("fmt2", || {
        _ = format_to(rand::random::<f64>(), "0.################", &mut out);
        out.clear();
    });

    println!("{}", t);

    Ok(())
}
