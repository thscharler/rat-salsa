use rat_salsa::{tr, ControlUI};

#[derive(Debug, PartialEq, Eq)]
struct EE0;

#[derive(Debug, PartialEq, Eq)]
struct EE1;

#[derive(Debug, PartialEq, Eq)]
enum Error {
    E0(EE0),
    E1(EE1),
}

impl From<EE0> for Error {
    fn from(value: EE0) -> Self {
        Error::E0(value)
    }
}

impl From<EE1> for Error {
    fn from(value: EE1) -> Self {
        Error::E1(value)
    }
}

fn r0() -> Result<usize, EE0> {
    Err(EE0)
}

fn cr0() -> ControlUI<usize, Error> {
    tr!(r0());
    ControlUI::Continue
}

fn r1() -> ControlUI<usize, EE1> {
    ControlUI::Err(EE1)
}

fn cr1() -> ControlUI<usize, Error> {
    _ = tr!(r1());
    ControlUI::Continue
}

fn r2_0() -> Result<ControlUI<usize, EE1>, EE0> {
    Err(EE0)
}

fn cr2_0() -> ControlUI<usize, Error> {
    _ = tr!(r2_0());
    ControlUI::Continue
}

fn r2_1() -> Result<ControlUI<usize, EE1>, EE0> {
    Ok(ControlUI::Err(EE1))
}

fn cr2_1() -> ControlUI<usize, Error> {
    _ = tr!(r2_1());
    ControlUI::Continue
}

fn cr2_1b() -> ControlUI<usize, Error> {
    // this essentially works, but might be a bit of a trap...
    _ = tr!(tr!(r2_1()));
    ControlUI::Continue
}

fn cr2_expanded() -> ControlUI<usize, Error> {
    _ = {
        use ::rat_salsa::{ControlUI, SplitResult};
        let x = {
            use ::rat_salsa::{ControlUI, SplitResult};
            let x = (r2_1());
            let s = SplitResult::split(x);
            match s {
                (None, Some(e)) => {
                    let ee = e.into();
                    return ControlUI::Err(ee);
                }
                (Some(v), None) => v,
                _ => unreachable!(),
            }
        };
        let s = SplitResult::split(x);
        match s {
            (None, Some(e)) => {
                let ee = e.into();
                return ControlUI::Err(ee);
            }
            (Some(v), None) => v,
            _ => unreachable!(),
        }
    };
    ControlUI::Continue
}

fn cr2_1c() -> ControlUI<usize, Error> {
    // does not compile - ok
    // _ = check_break!(tr!(r2_1()));
    ControlUI::Continue
}

#[test]
fn test_0() {
    let rr = cr0();
    assert_eq!(rr, ControlUI::Err(Error::E0(EE0)));
}

#[test]
fn test_1() {
    let rr = cr1();
    assert_eq!(rr, ControlUI::Err(Error::E1(EE1)));
}

#[test]
fn test_2() {
    let rr = cr2_0();
    assert_eq!(rr, ControlUI::Err(Error::E0(EE0)));

    // this would require specialization to work.
    let rr = cr2_1();
    assert_ne!(rr, ControlUI::Err(Error::E1(EE1))); // should be assert_eq

    let rr = cr2_1b();
    assert_eq!(rr, ControlUI::Err(Error::E1(EE1)));

    // does not work this way. no conversions of the error type applied.
    // but this one is ok.
    let rr = cr2_1c();
    assert_ne!(rr, ControlUI::Err(Error::E1(EE1)));
}
