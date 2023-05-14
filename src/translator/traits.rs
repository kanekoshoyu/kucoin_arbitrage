use crate::model::orderbook::Orderbook;
use crate::model::symbol::SymbolInfo;
pub trait OrderBookTranslator {
    fn to_internal(&self) -> Orderbook;
}

pub trait OrderBookChangeTranslator {
    fn to_internal(&self, serial: u64) -> (String, Orderbook);
}

pub trait SymbolInfoTranslator {
    fn to_internal(&self) -> SymbolInfo;
}
