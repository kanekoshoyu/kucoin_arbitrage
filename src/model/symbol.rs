use ordered_float::OrderedFloat;
/// symbol info that has its base, quote, base_min and base_increment, used for formatting the order placement
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymbolInfo {
    // e.g. BTC-USDT (name should be BASE-QUOTE, thus use symbol instead of name)
    pub symbol: String,
    // e.g. BTC
    pub base: String,
    // e.g. USDT
    pub quote: String,
    // e.g. 0.1
    pub base_min: OrderedFloat<f64>,
    // e.g. 0.001
    pub base_increment: OrderedFloat<f64>,
}
