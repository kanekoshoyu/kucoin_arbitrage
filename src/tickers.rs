use crate::strings::*;
use kucoin_rs::failure;
use kucoin_rs::kucoin::client::Kucoin;
use kucoin_rs::kucoin::model::market::SymbolList;
use std::collections::HashMap;

// helper function to check if the symbol contains both quote1 and quote2
fn symbol_is_match(
    dict: &mut HashMap<String, bool>,
    ticker: &str,
    quote1: &str,
    quote2: &str,
) -> bool {
    // no match
    let (temp_base, temp_quote) = symbol_to_tuple(ticker).expect("wrong format");
    if temp_quote.ne(quote1) && temp_quote.ne(quote2) {
        return false;
    }
    if let Some(n) = dict.get_mut(temp_base) {
        // second match, both quote1 or quote2 (assumes quote1 and quote2 happens only uniquely)
        (*n) = true;
        true
    } else {
        // first match, either quote1 or quote2
        dict.insert(temp_base.to_string(), false);
        false
    }
}

// get all the base currencies that has trading pair with quote1 and quote2
// e.g. ETH, DOT
pub async fn bases_with_quotes(
    api: Kucoin,
    quote1: &str,
    quote2: &str,
) -> Result<Vec<String>, failure::Error> {
    let res = api.get_all_tickers().await?;
    let all_ticker = res.data.expect("connection failure");
    let tickers = all_ticker.ticker;
    // let total = tickers.len();
    // info!("Total symbols: {:#?}", total); // approx 1300
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

// generate a list of trading pair String names that has trading pair with quote1 and quote2 (including "quote1-quote2")
// e.g. quote: BTC, USDT returns ["BTC-USDT", "X-BTC", "X-USDT", "Y-BTC", "Y-USDT", "Z-BTC", "Z-USDT"]
pub async fn symbol_whitelisted(
    api: Kucoin,
    quote1: &str,
    quote2: &str,
) -> Result<Vec<String>, failure::Error> {
    let bases = bases_with_quotes(api, quote1, quote2).await?;
    let mut res: Vec<String> = Vec::new();
    // append "quote1-quote2"
    res.push(symbol_string(quote1, quote2));
    // append "base-quote"
    for base in bases.into_iter() {
        let base = base.as_str();
        res.push(symbol_string(base, quote1));
        res.push(symbol_string(base, quote2));
    }
    Ok(res)
}

// get all the symbol list directly from api
pub async fn all_symbol_list(api: Kucoin) -> Vec<SymbolList> {
    let res = api.get_symbol_list(None).await;
    let api_data = res.expect("message  error");
    api_data.data.expect("empty!")
}

pub type SymbolMap = HashMap<String, SymbolList>;

pub async fn symbol_list_filtered(
    api: Kucoin,
    mut symbol_names: Vec<String>,
) -> Result<SymbolMap, failure::Error> {
    let mut res: SymbolMap = HashMap::new();
    let mut symbol_lists = all_symbol_list(api.clone()).await;

    while symbol_names.len() > 0 {
        let symbol_name = symbol_names.pop().expect("error popping name");
        // log::info!("symbol_name: {symbol_name:#?}");

        // let mut n = symbol_lists.len();
        let n = symbol_lists.len(); //approx 1277
        let mut i = 0;
        while i < n {
            let symbol_list = symbol_lists.get(i).expect("error popping symbol_list");
            // log::info!("symboldata{symboldata:#?}");
            let is_same = symbol_list.symbol.eq(&symbol_name);
            let is_tradeable = symbol_list.enable_trading;
            if is_same && is_tradeable {
                // log::info!("match ");
                let symbol_list = symbol_lists.remove(i);
                res.insert(symbol_name, symbol_list);
                break;
            }
            i = i + 1;
        }
        // log::info!("missing: {symbol_name:?}");
    }

    return Ok(res);
}

#[cfg(test)]
mod tests {
    use crate::mirror;
    use core::panic;
    use kucoin_rs::tokio;
    #[test]
    fn test_insert_and_read() {
        let ticker_name = "BTC-USDT".to_string();
        let def_ticker = mirror::TickerInfo::default();
        mirror::insert(ticker_name.clone(), def_ticker.clone());
        if mirror::has(ticker_name.clone()) {
            return;
        } else {
            panic!("not inserted");
        }
    }

    #[tokio::test]
    async fn test_geneate_whitelist() {
        use crate::tickers::*;
        use kucoin_rs::kucoin::client::{Kucoin, KucoinEnv};
        // crate::logger::log_init();
        let api = Kucoin::new(KucoinEnv::Live, None).unwrap();
        let q1 = "BTC";
        let q2 = "USDT";
        let symbols = symbol_whitelisted(api.clone(), q1, q2).await;
        let symbols = symbols.expect("err parsing symbol_whitelisted");
        // info!("symbol_whitelisted.len(): {:#?}", symbols.len());
        let res = symbol_list_filtered(api, symbols.clone())
            .await
            .expect("symbols_selected");
        // info!("symbol_list_filtered.len(): {:?}", res.len());
        let mut vec: Vec<String> = Vec::new();
        for symbol in symbols.clone() {
            if !res.contains_key(&symbol) {
                vec.push(symbol);
            }
        }
        if (vec.len() + res.len()).ne(&symbols.len()) {
            panic!("Doesnt add up, {vec:#?}");
        }
    }

    #[tokio::test]
    async fn test_read_all_symbol_list() {
        use crate::tickers::*;
        use kucoin_rs::kucoin::client::{Kucoin, KucoinEnv};
        // crate::logger::log_init();
        let api = Kucoin::new(KucoinEnv::Live, None).unwrap();
        let x = all_symbol_list(api.clone()).await;
        // info!("{x:#?}");
        if x.len().eq(&0) {
            panic!("no symbol found");
        }
        return;
    }

    #[tokio::test]
    async fn test_read_symbol_list() {
        use crate::tickers::*;
        use kucoin_rs::kucoin::client::{Kucoin, KucoinEnv};
        // crate::logger::log_init();
        let api = Kucoin::new(KucoinEnv::Live, None).unwrap();
        let x = all_symbol_list(api.clone()).await;
        let mut found = false;
        for res in x.into_iter() {
            if res.base_currency == "EPS"
                || res.base_currency == "CBC"
                || res.base_currency == "TKY"
            {
                found = true;
            }
        }
        if found {
            panic!("these tickers are not meant to bn found!");
        }
    }
}
