use rat_salsa::number::{CurrencySym, Mode, NumberFormat, NumberSymbols, Token};
use std::mem::size_of;
use std::rc::Rc;

#[test]
pub fn size0() {
    println!("Token {}", size_of::<Token>());
    println!("Mode {}", size_of::<Mode>());
    println!("CurrencySym {}", size_of::<CurrencySym>());
    println!("NumberSymbols {}", size_of::<NumberSymbols>());
    println!("Rc<NumberSymbols> {}", size_of::<Rc<NumberSymbols>>());
    println!("NumberFormat {}", size_of::<NumberFormat>());
}
