use crate::model::orderbook::Orderbook;

pub trait OrderBookTranslator {
    fn to_internal(&self) -> Orderbook;
}

pub trait OrderBookChangeTranslator {
    fn to_internal(&self, serial: u64) -> (String, Orderbook);
}
