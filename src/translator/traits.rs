use crate::model::orderbook::Orderbook;
use crate::model::symbol::SymbolInfo;
use crate::model::trade::TradeInfo;

pub trait ToOrderBook {
    fn to_internal(&self) -> Orderbook;
}

pub trait ToOrderBookChange {
    fn to_internal(&self, serial: u64) -> (String, Orderbook);
}

pub trait ToSymbolInfo {
    fn to_internal(&self) -> SymbolInfo;
}

pub trait ToTradeInfo {
    fn to_internal(&self) -> Result<TradeInfo, failure::Error>;
}
