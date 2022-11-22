extern crate kucoin_rs;

use kucoin_rs::failure;
use kucoin_rs::kucoin::model::market::Tick;
use kucoin_rs::tokio::{self};

use env_logger::Builder;
use kucoin_rs::kucoin::client::{Kucoin, KucoinEnv};

// provide eazy data
extern crate lazy_static;
use chrono::Local;
use log::*;
use regex::Regex;
use std::collections::HashMap;
use std::io::Write;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}]: {}",
                Local::now().format("%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();

    let api = Kucoin::new(KucoinEnv::Live, None)?;
    let res = api.get_all_tickers().await?;
    info!("Hello world");

    let all_ticker = res.data.expect("connection failure");
    info!("Time: {:#?}", all_ticker.time);
    let tickers = all_ticker.ticker;
    let total = tickers.len();
    info!("Total: {:#?}", total); //1299

    let mut dict: HashMap<String, bool> = HashMap::new();

    let mut n = 0;
    for ticker in tickers.into_iter() {
        let t: Tick = ticker;
        let symbol = t.symbol;
        let (a, b) = symbol_to_tuple(symbol);
        // info!("{a}:{b}");
        if quote_is_match(&mut dict, &a, &b) {
            // ticker a is the one
            info!("{:?}", a);
            n += 1;
        }
    }
    info!("Matched: {n}");
    Ok(())
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
    let base1 = "BTC";
    let base2 = "USDT";
    if base.ne(base1) && base.ne(base2) {
        return false;
    }
    // first match
    if dict.get(quote).is_none() {
        let quote_str = quote.as_str();
        let quote = String::from_str(quote_str).unwrap();
        dict.insert(quote, false);
        return false;
    }
    // second match
    *(dict.get_mut(quote).unwrap()) = true;
    return true;
}

fn symbol_to_tuple(text: String) -> (String, String) {
    // regex to divide the tickers
    let splitter = Regex::new(r"[-]").unwrap();
    // info!("{text}");
    let txt: &str = &text[..];
    let splits: Vec<_> = splitter.split(txt).into_iter().collect();
    if 2 != splits.len() {
        panic!("invalid format {splits:?}")
    }
    (splits[0].to_string(), splits[1].to_string())
}
