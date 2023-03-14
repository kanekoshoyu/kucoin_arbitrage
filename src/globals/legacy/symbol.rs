extern crate lazy_static;
use kucoin_rs::kucoin::model::market::SymbolList;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/*
please refer to
https://docs.kucoin.com/#get-full-order-book-aggregated
https://docs.kucoin.com/#level-2-market-data
*/

pub type SymbolMap = HashMap<String, SymbolList>;
lazy_static::lazy_static! {
    pub static ref SYMBOLMAP: Arc<Mutex<SymbolMap>> = Arc::new(Mutex::new(HashMap::new()));
}

// return None when new value added
// return Some when there was a value beforehand
pub fn insert_symbol_list(symbol: String, symbol_list: SymbolList) -> Option<SymbolList> {
    let mut p = SYMBOLMAP.lock().unwrap();
    (*p).insert(symbol, symbol_list)
}

pub fn insert_symbolmap(symbolmap: SymbolMap) -> Result<(), &'static str> {
    for (symbol_name, symbol_list) in symbolmap.into_iter() {
        let res = insert_symbol_list(symbol_name, symbol_list);
        if res.is_some() {
            return Err("duplicating data");
        }
    }
    Ok(())
}

pub fn get_clone(symbol: String) -> Option<SymbolList> {
    // TODO: make it return none if it is actually none
    let mut p = SYMBOLMAP.lock().unwrap();
    let res = (*p).get_mut(&symbol).unwrap();
    let res = SymbolList {
        symbol: res.symbol.clone(),
        name: res.name.clone(),
        base_currency: res.base_currency.clone(),
        quote_currency: res.quote_currency.clone(),
        base_min_size: res.base_min_size.clone(),
        base_max_size: res.base_max_size.clone(),
        quote_max_size: res.quote_max_size.clone(),
        base_increment: res.base_increment.clone(),
        quote_increment: res.quote_increment.clone(),
        price_increment: res.price_increment.clone(),
        fee_currency: res.fee_currency.clone(),
        enable_trading: res.enable_trading,
        is_margin_enabled: res.is_margin_enabled,
    };
    Some(res)
}
