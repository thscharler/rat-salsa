use rat_salsa::number::{CurrencySym, Mode, NumberFormat, NumberSymbols, Token};
use std::mem::{offset_of, size_of};
use std::rc::Rc;

#[test]
pub fn size0() {
    println!("Token {}", size_of::<Token>());
    println!("Mode {}", size_of::<Mode>());
    println!("CurrencySym {}", size_of::<CurrencySym>());
    println!("NumberSymbols {}", size_of::<NumberSymbols>());
    println!("Rc<NumberSymbols> {}", size_of::<Rc<NumberSymbols>>());
    println!("NumberFormat {}", size_of::<NumberFormat>());

    println!("has_exp {}", offset_of!(NumberFormat, has_exp));
    println!("has_exp_0 {}", offset_of!(NumberFormat, has_exp_0));
    println!("has_frac_0 {}", offset_of!(NumberFormat, has_frac_0));
    println!("precision {}", offset_of!(NumberFormat, precision));
    println!("tok {}", offset_of!(NumberFormat, tok));
    println!("sym {}", offset_of!(NumberFormat, sym));
}
