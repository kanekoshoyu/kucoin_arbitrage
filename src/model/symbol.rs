use ordered_float::OrderedFloat;
use std::collections::HashMap;

/// symbol as key, orderbook as value
pub type FullSymbolInfo = HashMap<String, SymbolInfo>; //Symbols to Orderbook

/// orderbook for each symbol, contains ask, bid, time and sequence
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymbolInfo {
    pub name: String,
    pub base_currency: String,
    pub quote_currency: String,
    pub base_min_size: OrderedFloat<f32>,
    pub base_max_size: OrderedFloat<f32>,
    pub quote_min_size: OrderedFloat<f32>,
    pub quote_max_size: OrderedFloat<f32>,
    pub base_increment: OrderedFloat<f32>,
    pub quote_increment: OrderedFloat<f32>,
    pub is_avaiable: bool,
}
