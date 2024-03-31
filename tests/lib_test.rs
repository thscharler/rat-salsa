#![allow(dead_code, unreachable_pub)]

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hint::black_box;
use std::io::{stdout, Write};
use std::time::Instant;
use std::{fs, mem};

pub fn init_test() -> Result<(), anyhow::Error> {
    fs::create_dir_all("test_out")?;
    Ok(())
}

#[derive(Clone, Copy, Debug)]
pub enum Unit {
    Nanosecond,
    Microsecond,
    Millisecond,
    Second,
}

impl Unit {
    pub fn conv(&self, nanos: f64) -> f64 {
        match self {
            Unit::Nanosecond => nanos,
            Unit::Microsecond => nanos / 1000.0,
            Unit::Millisecond => nanos / 1000000.0,
            Unit::Second => nanos / 1000000000.0,
        }
    }
}

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let v = match self {
            Unit::Nanosecond => "ns",
            Unit::Microsecond => "µs",
            Unit::Millisecond => "ms",
            Unit::Second => "s",
        };
        write!(f, "{}", v)
    }
}

#[derive(Clone, Debug)]
pub struct Timing {
    pub skip: usize,
    pub runs: usize,
    pub divider: u64,
    pub unit: Unit,

    /// Samples for the current run.
    pub samples: Vec<f64>,

    /// Collected samples in ns. already divided by divider.
    pub summed: HashMap<&'static str, Vec<f64>>,
}

impl Timing {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn skip(mut self, skip: usize) -> Self {
        self.skip = skip;
        self
    }

    pub fn runs(mut self, runs: usize) -> Self {
        self.runs = runs;
        self
    }

    pub fn divider(mut self, divider: u64) -> Self {
        self.divider = divider;
        self
    }

    pub fn unit(mut self, unit: Unit) -> Self {
        self.unit = unit;
        self
    }

    /// Runs the measurement of bench().
    ///
    /// After each bench-run collect() is called with the result which can collect additional
    /// information based on the result.
    ///
    /// After one run summarize() is called to compress and additional information.
    /// The samples themselves are automatically collected into `self.summed`.
    ///
    pub fn run_bench<E, R>(
        &mut self,
        name: &str,
        mut bench: impl FnMut() -> Result<R, E>,
        mut collect: impl FnMut(&mut Timing, Result<R, E>) -> Result<R, E>,
        mut summarize: impl FnMut(&mut Timing, &str),
    ) -> Result<R, E> {
        assert!(self.runs > 0);
        assert!(self.divider > 0);

        print!("run {} ", name);

        let mut run_bench = move || {
            let now = Instant::now();
            let result = bench();
            (now.elapsed(), result)
        };

        let mut n = 1;
        let result = loop {
            let (elapsed, result) = black_box(run_bench());

            let result = if n > self.skip {
                let sample = elapsed.as_nanos() as f64 / self.divider as f64;
                self.samples.push(sample);
                collect(self, result)
            } else {
                result
            };
            if n > self.skip + self.runs {
                break result;
            }

            n += 1;

            let d = 10usize.pow(n.ilog10());
            if n % d == 0 {
                print!(".");
            }
            let _ = stdout().flush();
        };

        summarize(self, name);

        self.summed
            .insert(name.to_string().leak(), mem::take(&mut self.samples));

        println!();

        result
    }

    /// Run the bench with a procedure instead of a function.
    pub fn run_proc(&mut self, name: &str, mut bench: impl FnMut()) {
        _ = self.run_bench::<(), ()>(
            name,
            || {
                bench();
                Ok(())
            },
            |_t, result| result,
            |_t, _name| {},
        );
    }

    /// Run with only the basic bench-function.
    pub fn run_basic<R, E>(
        &mut self,
        name: &str,
        bench: impl FnMut() -> Result<R, E>,
    ) -> Result<R, E> {
        self.run_bench(name, bench, |_t, result| result, |_t, _name| {})
    }

    pub fn n(&self, name: &str) -> usize {
        self.summed.get(name).expect("sample").len()
    }

    pub fn sum(&self, name: &str) -> f64 {
        self.summed.get(name).expect("sample").iter().sum()
    }

    pub fn mean(&self, name: &str) -> f64 {
        self.summed.get(name).expect("sample").iter().sum::<f64>()
            / self.summed.get(name).expect("sample").len() as f64
    }

    pub fn median(&self, name: &str) -> (f64, f64, f64) {
        let mut s = self.summed.get(name).expect("sample").clone();
        s.sort_by(|v, w| v.total_cmp(w));
        let m0 = s.len() * 1 / 10;
        let m5 = s.len() / 2;
        let m9 = s.len() * 9 / 10;

        (s[m0], s[m5], s[m9])
    }

    pub fn lin_dev(&self, name: &str) -> f64 {
        let mean = self.mean(name);
        let lin_sum = self
            .summed
            .get(name)
            .expect("sample")
            .iter()
            .map(|v| (*v - mean).abs())
            .sum::<f64>();
        lin_sum / self.summed.len() as f64
    }

    pub fn std_dev(&self, name: &str) -> f64 {
        let mean = self.mean(name);
        let std_sum = self
            .summed
            .get(name)
            .expect("sample")
            .iter()
            .map(|v| (*v - mean) * (*v - mean))
            .sum::<f64>();
        (std_sum / self.summed.len() as f64).sqrt()
    }
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            skip: 0,
            runs: 1,
            divider: 1,
            unit: Unit::Nanosecond,
            samples: Default::default(),
            summed: Default::default(),
        }
    }
}

impl Display for Timing {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f,)?;
        writeln!(
            f,
            "| name | n | sum | 1/10 | median | 9/10 | mean | lin_dev | std_dev |"
        )?;
        writeln!(f, "|:---|:---|:---|:---|:---|:---|:---|:---|:---|")?;

        for name in self.summed.keys() {
            let n = self.n(name);
            let sum = self.sum(name);
            let (m0, m5, m9) = self.median(name);
            let mean = self.mean(name);
            let lin = self.lin_dev(name);
            let std = self.std_dev(name);

            writeln!(
                f,
                "| {} | {} | {:.2}{} | {:.2}{} | {:.2}{} | {:.2}{} | {:.2}{} | {:.2}{} | {:.2}{} |",
                name,
                n,
                self.unit.conv(sum),
                self.unit,
                self.unit.conv(m0),
                self.unit,
                self.unit.conv(m5),
                self.unit,
                self.unit.conv(m9),
                self.unit,
                self.unit.conv(mean),
                self.unit,
                self.unit.conv(lin),
                self.unit,
                self.unit.conv(std),
                self.unit,
            )?;
        }

        // write all the data
        if f.alternate() {
            for name in self.summed.keys() {
                for (n, sample) in self.summed.get(name).expect("sample").iter().enumerate() {
                    writeln!(f, "{}|{}|{}", name, n, sample)?;
                }
            }
        }

        Ok(())
    }
}
