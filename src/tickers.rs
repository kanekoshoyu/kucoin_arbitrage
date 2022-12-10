use crate::strings::symbol_to_tuple;
use kucoin_rs::failure;
use kucoin_rs::kucoin::client::Kucoin;
use kucoin_rs::kucoin::model::market::SymbolList;
use std::collections::HashMap;

pub async fn bases_with_quotes(
    api: Kucoin,
    quote1: &str,
    quote2: &str,
) -> Result<Vec<String>, failure::Error> {
    let res = api.get_all_tickers().await?;
    let all_ticker = res.data.expect("connection failure");
    let tickers = all_ticker.ticker;
    // let total = tickers.len();
    // info!("Total symbols: {:#?}", total); //1300
    let mut dict: HashMap<String, bool> = HashMap::new();
    let mut vec: Vec<String> = Vec::new();
    for tick in tickers.into_iter() {
        // log::info!("{tick:#?}");
        let symbol = tick.symbol.as_str();
        if symbol_is_match(&mut dict, symbol, quote1, quote2) {
            let (a, _b) = symbol_to_tuple(symbol).expect("wrong format");
            vec.push(a.to_string())
        }
    }
    Ok(vec)
}

pub type SymbolMap = HashMap<String, SymbolList>;
use log::info;
pub async fn symbols_selected(
    api: Kucoin,
    list_str_symbol: Vec<String>,
) -> Result<SymbolMap, failure::Error> {
    let res = api.get_symbol_list(None).await?;
    let lists_symbol = res.data.expect("connection failure");
    let mut res: SymbolMap = HashMap::new();
    for list_symbol in lists_symbol.into_iter() {
        let symbol = list_symbol.symbol.to_owned();
        // info!("symbol:{symbol:#?}");

        for base in list_str_symbol.to_owned().into_iter() {

            // if symbol.contains(&base) {
            //     res.insert(symbol, list_symbol);
            // }
        }

        // if list_str_symbol.contains(&symbol) {
        //     res.insert(symbol, list_symbol);
        // }
    }
    Ok(res)
}

pub fn symbol_string(base: &str, quote: &str) -> String {
    let mut n = String::from(base);
    n.push('-');
    n.push_str(quote);
    return n;
}

pub async fn symbol_whitelist(
    api: Kucoin,
    quote1: &str,
    quote2: &str,
) -> Result<Vec<String>, failure::Error> {
    let bases = bases_with_quotes(api, quote1, quote2).await?;
    let mut res: Vec<String> = Vec::new();
    let n = symbol_string(quote1, quote2);
    res.push(n);

    for base in bases.into_iter() {
        let base = base.as_str();
        res.push(symbol_string(base, quote1));
        res.push(symbol_string(base, quote2));
        // TODO: there is a subscription limit
        if res.len() > 100 {
            return Ok(res);
        }
    }
    Ok(res)
}

// rough way of reading.
fn symbol_is_match(
    dict: &mut HashMap<String, bool>,
    ticker: &str,
    quote1: &str,
    quote2: &str,
) -> bool {
    // no match
    // let bases = ("BTC", "USDT");
    let (temp_base, temp_quote) = symbol_to_tuple(ticker).expect("wrong format");
    if temp_quote.ne(quote1) && temp_quote.ne(quote2) {
        return false;
    }
    // first match
    if let Some(n) = dict.get_mut(temp_base) {
        (*n) = true;
        true
    } else {
        dict.insert(temp_base.to_string(), false);
        false
    }
}

#[cfg(test)]
mod tests {
    use core::panic;
    use kucoin_rs::tokio;

    #[test]
    fn test_insert_and_read() {
        let ticker_name = "BTC-USDT".to_string();
        let def_ticker = crate::mirror::TickerInfo::default();
        let ticker_name_clone = ticker_name.clone();

        let mir = crate::mirror::MIRROR.clone();
        {
            let mut mir = mir.lock().unwrap();
            let mir = &mut (*mir);
            let cloned = def_ticker.clone();
            mir.insert(ticker_name, cloned);
        }
        {
            let mut mir = mir.lock().unwrap();
            let mir = &mut (*mir);
            if let Some(_data) = mir.get_mut(&ticker_name_clone) {
                return; //value was inserted
            } else {
                panic!("not inserted");
            }
        }
    }

    #[tokio::test]
    async fn test_geneate_whitelist() {
        use crate::tickers::*;
        use kucoin_rs::kucoin::client::{Kucoin, KucoinEnv};
        use log::info;
        crate::logger::log_init();
        let api = Kucoin::new(KucoinEnv::Live, None).unwrap();
        let q1 = "BTC";
        let q2 = "USDT";
        let bases = bases_with_quotes(api.clone(), q1, q2).await;
        let bases = bases.expect("err parsing bases_with_quotes");
        info!("bases.len(): {:#?}", bases.len());

        let res = symbols_selected(api, bases)
            .await
            .expect("symbols_selected");
        info!("symbols_selected: {res:#?}");
        info!("size: {:?}", res.len());

        for (key, val) in &res {
            let quote = &val.quote_currency;
            if quote.ne(q1) || quote.ne(q2) {
                panic!("wrong value found with {key:?}");
            }
        }
        return;
    }
}
