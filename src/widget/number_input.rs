///
/// Symbols for numbers
///
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct NumberSymbols {
    /// Decimal separator
    pub decimal_sep: String,
    /// Decimal grouping
    pub decimal_grp: String,
    /// Minus sign
    pub negative_sym: String,
}
