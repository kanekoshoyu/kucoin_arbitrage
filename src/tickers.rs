use crate::strings::ticker_to_tuple;
use kucoin_rs::failure;
use kucoin_rs::kucoin::client::Kucoin;
use std::collections::HashMap;

// pub async fn tickers(api: &str, :&str) -> Result<Vec<String>, failure::Error> {

pub async fn ticker_whitelist(
    api: Kucoin,
    base1: &str,
    base2: &str,
) -> Result<Vec<String>, failure::Error> {
    let res = api.get_all_tickers().await?;
    let all_ticker = res.data.expect("connection failure");
    // info!("Time: {:#?}", all_ticker.time);
    let tickers = all_ticker.ticker;
    // let total = tickers.len();
    // info!("Total Tickers: {:#?}", total); //1300

    let mut dict: HashMap<String, bool> = HashMap::new();
    let mut vec: Vec<String> = Vec::new();
    for ticker in tickers.into_iter() {
        log::info!("{ticker:#?}");
        let symbol = ticker.symbol;
        if ticker_is_match(&mut dict, symbol.as_str(), base1, base2) {
            let (a, _b) = ticker_to_tuple(symbol.as_str()).expect("wrong format");
            vec.push(a.to_string())
        }
    }
    Ok(vec)
}

pub async fn symbol_whitelist(
    api: Kucoin,
    base1: &str,
    base2: &str,
) -> Result<Vec<String>, failure::Error> {
    let titles = ticker_whitelist(api, base1, base2).await?;
    let mut res: Vec<String> = Vec::new();
    let mut n = String::from(base1);
    n.push('-');
    n.push_str(base2);
    res.push(n);

    for title in titles.into_iter() {
        let mut push_base = |base: &str| {
            res.push({
                let mut clone = title.clone();
                clone.push('-');
                clone.push_str(base);
                clone
            });
        };
        push_base(base1);
        push_base(base2);
        // TODO: there is a subscription limit
        if res.len() > 100 {
            return Ok(res);
        }
    }
    Ok(res)
}

// rough way of reading.
fn ticker_is_match(
    dict: &mut HashMap<String, bool>,
    ticker: &str,
    base1: &str,
    base2: &str,
) -> bool {
    // no match
    // let bases = ("BTC", "USDT");
    let (a, b) = ticker_to_tuple(ticker).expect("wrong format");
    if b.ne(base1) && b.ne(base2) {
        return false;
    }
    // first match
    if let Some(n) = dict.get_mut(a) {
        (*n) = true;
        true
    } else {
        dict.insert(a.to_string(), false);
        false
    }
}

#[cfg(test)]
mod tests {
    use core::panic;

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
}
