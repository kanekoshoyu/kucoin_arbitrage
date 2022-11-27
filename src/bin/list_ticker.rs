extern crate kucoin_rs;

use kucoin_rs::failure;
use kucoin_rs::kucoin::client::{Kucoin, KucoinEnv};
use kucoin_rs::tokio::{self};
extern crate lazy_static;
use kucoin_arbitrage::shared::*;
use log::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    kucoin_arbitrage::shared::log_init();
    let api = Kucoin::new(KucoinEnv::Live, None)?;
    let res = ticker_list_with_btc_usdt(api).await?;
    let n = res.len();
    info!("Matched: {n}");
    // info!("res: {res:#?}");
    Ok(())
}

pub async fn ticker_list_with_btc_usdt(api: Kucoin) -> Result<Vec<String>, failure::Error> {
    let res = api.get_all_tickers().await?;
    let all_ticker = res.data.expect("connection failure");
    // info!("Time: {:#?}", all_ticker.time);
    let tickers = all_ticker.ticker;
    let total = tickers.len();
    info!("Total Tickers: {:#?}", total); //1300

    let mut dict: HashMap<String, bool> = HashMap::new();
    let mut vec: Vec<String> = Vec::new();
    for ticker in tickers.into_iter() {
        let symbol = ticker.symbol;
        let (a, b) = ticker_to_tuple(symbol).expect("wrong format");
        // info!("{a}:{b}");
        if quote_is_match(&mut dict, &a, &b) {
            vec.push(a);
        }
    }
    Ok(vec)
}

/*
    Symbol looks like this
    "GMB-BTC"
    "TRIBE-USDT"
    "MLK-USDT"
*/

// rough way of reading.
fn quote_is_match(dict: &mut HashMap<String, bool>, quote: &String, base: &String) -> bool {
    // no match
    let bases = ("BTC", "USDT");
    if base.ne(bases.0) && base.ne(bases.1) {
        return false;
    }
    // first match
    if dict.get(quote).is_none() {
        dict.insert(quote.to_owned(), false);
        false
    } else {
        // second match
        *(dict.get_mut(quote).unwrap()) = true;
        true
    }
}
