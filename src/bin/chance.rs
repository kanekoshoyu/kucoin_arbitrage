extern crate kucoin_rs;

use kucoin_rs::failure;
use kucoin_rs::futures::TryStreamExt;
use kucoin_rs::kucoin::{
    client::{Kucoin, KucoinEnv},
    model::websocket::{KucoinWebsocketMsg, WSTopic, WSType},
    websocket::KucoinWebsocket,
};
use kucoin_rs::tokio::{self};

use kucoin_arbitrage::mirror::*;
use log::*;
use std::sync::{Arc, Mutex};

use kucoin_arbitrage::globals::{config, performance};
use kucoin_arbitrage::logger;
use kucoin_arbitrage::tasks;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // provide logging format
    logger::log_init();
    use kucoin_arbitrage::tickers::symbol_whitelist;
    info!("Hello world");

    let credentials = config::credentials();

    info!("{credentials:#?}");
    let api = Kucoin::new(KucoinEnv::Live, Some(credentials))?;
    let api_clone = api.clone();
    let url = api.get_socket_endpoint(WSType::Public).await?;
    let ticker_list = symbol_whitelist(api_clone, "BTC", "USDT").await?;

    let mut ws = api.websocket();

    // fill in the arbitrage list automatically

    let subs = vec![WSTopic::Ticker(ticker_list)];
    ws.subscribe(url, subs).await?;

    info!("Async polling");
    let mir = MIRROR.clone();
    tokio::spawn(async move { sync_tickers(ws, mir).await });

    // loop {
    //     report_status().expect("report status error");
    // }
    tasks::background_routine().await
}

use kucoin_arbitrage::mirror::Map;
use kucoin_arbitrage::strings::*;

async fn sync_tickers(
    mut ws: KucoinWebsocket,
    mirror: Arc<Mutex<Map>>,
) -> Result<(), failure::Error> {
    let base1 = "BTC";
    let base2 = "USDT";
    while let Some(msg) = ws.try_next().await? {
        match msg {
            KucoinWebsocketMsg::TickerMsg(msg) => {
                performance::increment();
                // if the updated data is greater than the ex
                // TODO: optimize the cloning mess here.
                let ticker = topic_to_symbol(msg.topic).expect("wrong topic format");
                let ticker_clone = ticker.clone();
                let (coin1, _coin2) =
                    symbol_to_tuple(ticker_clone.as_str()).expect("wrong ticker format");
                let ticker_clone = ticker.clone();
                {
                    // update the map
                    let mut m = mirror.lock().unwrap();
                    let tickers: &mut Map = &mut (*m);
                    if let Some(data) = tickers.get_mut(&ticker_clone) {
                        // TODO: some sort of a delta function to reduce the time we keep rewriting.
                        data.symbol = msg.data;
                    } else {
                        tickers.insert(ticker_clone, TickerInfo::new(msg.data));
                    }
                }
                // lambda function
                let append = |a: &str, b: &str| {
                    let mut res = String::from(a);
                    res.push('-');
                    res.push_str(b);
                    res
                };

                let ab = append(base1, base2); //BTC-USDT
                if ticker.eq(ab.as_str()) {
                    // skip when it is a btc-usdt pair (i.e. ab)
                    continue;
                }
                let ta = append(coin1, base1); //ETH-BTC
                let tb = append(coin1, base2); //ETH-USDT

                // info!("studying Triangle: {tb}, {ta}, {ab}");

                let triangle: Option<[TickerInfo; 3]> = {
                    // update the map
                    let mut m = mirror.lock().unwrap();
                    let tickers: &mut Map = &mut (*m);
                    let tb = tickers.get(&tb);
                    let ta = tickers.get(&ta);
                    let ab = tickers.get(&ab);
                    if tb.is_none() {
                        continue;
                    }
                    if ta.is_none() {
                        continue;
                    }
                    if ab.is_none() {
                        continue;
                    }
                    let tb = tb.unwrap().to_owned();
                    let ta = ta.unwrap().to_owned();
                    let ab = ab.unwrap().to_owned();

                    Some([tb, ta, ab])
                };
                let triangle = triangle.unwrap();
                let tb = triangle.get(0).unwrap().to_owned();
                let ta = triangle.get(1).unwrap().to_owned();
                let ab = triangle.get(2).unwrap().to_owned();

                // conduct the analysis
                let res = chance(tb, ta, ab);
                if let Some(sequence) = res {
                    // TODO: calculate the profit ratio accutately
                    let sc = sequence.clone();
                    let profit_percentage = profit_percentage(sequence) * 100f64;
                    // TODO: chance this brute threshold into something more quantitative
                    if profit_percentage < 1.5 {
                        continue;
                    }
                    let profit_percentage = format!("{:.5}", profit_percentage);
                    let i = sc.get(1).unwrap();
                    info!("Found arbitrage at {coin1:?}");
                    if i.action.eq(&Action::Buy) {
                        info!("BBS, profit {}%", profit_percentage);
                    } else {
                        info!("BSS, profit {}%", profit_percentage);
                    }
                    // info!("{sequence:#?}");
                }
            }
            KucoinWebsocketMsg::PongMsg(_msg) => {}
            KucoinWebsocketMsg::WelcomeMsg(_msg) => {}
            _ => {
                panic!("unexpected msgs received: {msg:?}")
            }
        }
    }
    Ok(())
}

use kucoin_arbitrage::mirror::TickerInfo;
#[derive(Debug, Clone, PartialEq, Eq)]

pub enum Action {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
pub struct ActionInfo {
    action: Action,
    ticker: TickerInfo,
    volume: String,
}

// sequence in ascending order
type ActionSequence = [ActionInfo; 3];

// TODO: profit in USDT
fn profit_usdt(_seq: ActionSequence) {
    unimplemented!();
}

fn profit_percentage(seq: ActionSequence) -> f64 {
    let [x, y, z] = seq;
    if y.action.eq(&Action::Sell) {
        // BSS
        let high = x.ticker.get_ask().0;
        let low = y.ticker.get_bid().0 * z.ticker.get_bid().0;
        return (high - low) / high;
    } else {
        // BBS
        let high = x.ticker.get_ask().0 * y.ticker.get_ask().0;
        let low = z.ticker.get_bid().0;
        return (high - low) / high;
    }
}
fn chance(
    ticker_target_base: TickerInfo,
    ticker_target_alt: TickerInfo,
    ticker_alt_base: TickerInfo,
) -> Option<ActionSequence> {
    // get both prices
    let tb = ticker_target_base.clone();
    // info!("{tb:#?}");
    let ta = ticker_target_alt.clone();
    let ab = ticker_alt_base.clone();
    let ((tb_ap, _tb_av), (tb_bp, _tb_bv)) = tb.get_askbid(); //BSS buy, BBS sell
    let ((ta_ap, _ta_av), (ta_bp, _ta_bv)) = ta.get_askbid(); //BBS buy. BSS sell
    let ((ab_ap, _ab_av), (ab_bp, _ab_bv)) = ab.get_askbid(); //BBS buy, BSS sell

    // TODO: copnsider transaction fee as well
    // BSS (forward)
    if tb_ap > (ta_bp * ab_bp) {
        // TODO: get the sum from a shared Mutex place (USDT)
        let sum = 100f64;
        return Some(bss_action_sequence(sum, [tb, ta, ab]));
        // unimplemented!("BSS (buy target first)")
    }
    // BBS (backward)
    if ab_ap * ta_ap > tb_bp {
        let sum = 100f64;
        return Some(bbs_action_sequence(sum, [ab, ta, tb]));
        // unimplemented!("BBS (sell target last)")
    }
    // get the last one

    return None;
}

fn bbs_action_sequence(sum: f64, ticker_info_bbs: [TickerInfo; 3]) -> ActionSequence {
    let err_msg = "ticker_info_bss error";
    let b1 = ticker_info_bbs.get(0).expect(err_msg);
    let b2 = ticker_info_bbs.get(1).expect(err_msg);
    let s = ticker_info_bbs.get(2).expect(err_msg);

    let b1a = b1.get_ask();
    let b2a = b2.get_ask();
    let sb = s.get_bid();

    let ratio = bbs_lcf_ratio(sum, b1a, b2a, sb);

    let b1_size = sum * ratio;
    let b2_size = b1_size * b1a.0;
    let s_size = b2_size * b2a.0 / sb.0;
    let b1_size = format!("{:.2}", b1_size);
    let b2_size = format!("{:.2}", b2_size);
    let s_size = format!("{:.2}", s_size);

    return [
        // calculate the size
        ActionInfo {
            action: Action::Buy,
            ticker: b1.clone(),
            volume: b1_size,
        },
        ActionInfo {
            action: Action::Buy,
            ticker: b2.clone(),
            volume: b2_size,
        },
        ActionInfo {
            action: Action::Sell,
            ticker: s.clone(),
            volume: s_size,
        },
    ];
}

fn bss_action_sequence(sum: f64, ticker_info_bss: [TickerInfo; 3]) -> ActionSequence {
    let err_msg = "ticker_info_bss error";
    let b = ticker_info_bss.get(0).expect(err_msg);
    let s1 = ticker_info_bss.get(1).expect(err_msg);
    let s2 = ticker_info_bss.get(2).expect(err_msg);

    let ba = b.get_ask();
    let s1b = s1.get_bid();
    let s2b = s2.get_bid();
    let ratio = bss_lcf_ratio(sum, ba, s1b, s2b);

    let b_size = sum * ratio;
    let s1_size = b_size * s1b.0 / s1b.0;
    let s2_size = s1_size * s1b.0 / s2b.0;
    // TODO: make floating point precision programmable
    let b_size = format!("{:.2}", b_size);
    let s1_size = format!("{:.2}", s1_size);
    let s2_size = format!("{:.2}", s2_size);

    return [
        ActionInfo {
            action: Action::Buy,
            ticker: b.clone(),
            volume: b_size,
        },
        ActionInfo {
            action: Action::Sell,
            ticker: s1.clone(),
            volume: s1_size,
        },
        ActionInfo {
            action: Action::Sell,
            ticker: s2.clone(),
            volume: s2_size,
        },
    ];
}

fn bbs_lcf_ratio(sum: f64, b1: (f64, f64), b2: (f64, f64), s: (f64, f64)) -> f64 {
    // buy
    let b1_min = sum.min(b1.1);
    let nb1 = b1_min * b1.0;
    let b2_min = nb1.min(b2.1);
    let nb2 = b2_min * b2.0;
    // sell
    let s_min = nb2.min(s.0 * s.1);
    let ratio = (b1_min * b2_min * s_min) / (sum * nb1 * nb2);
    // TODO: add transaction fee into calculation
    return ratio;
}

fn bss_lcf_ratio(sum: f64, b: (f64, f64), s1: (f64, f64), s2: (f64, f64)) -> f64 {
    // buy
    let b_min = sum.min(b.1);
    let nb = b_min * b.0;
    // sell
    let s1_min = nb.min(s1.0 * s1.1);
    let ns1 = s1_min * s1.0;
    let s2_min = ns1.min(s2.0 * s2.1);
    let ratio = (b_min * s1_min * s2_min) / (sum * nb * ns1);
    // TODO: add transaction fee into calculation
    return ratio;
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_bbs_lcf() {
        let sum = 100f64;
        let b1 = (1.6f64, 100f64);
        let b2 = (82f64, 80f64);
        let s = (128f64, 50f64);

        let ratio = crate::bbs_lcf_ratio(sum, b1, b2, s);
        let b1_size = sum * ratio;
        let b2_size = b1_size * b1.0;
        let s_size = b2_size * b2.0 / s.0;
        let b1_size = format!("{:.2}", b1_size);
        let b2_size = format!("{:.2}", b2_size);
        let s_size = format!("{:.2}", s_size);

        let float_err = "float deparse error";
        let b1_size = b1_size.parse::<f64>().expect(float_err);
        let b2_size = b2_size.parse::<f64>().expect(float_err);
        let s_size = s_size.parse::<f64>().expect(float_err);
        if (b1_size <= b1.1) && (b2_size <= b2.1) && (s_size <= s.1) {
            return;
        }
        panic!("failed: [{b1_size},{b2_size},{s_size}]");
    }

    #[test]
    fn test_bss_lcf() {
        let sum = 100f64;
        let b = (1.6f64, 50f64);
        let s1 = (1.3f64, 80f64);
        let s2 = (1.1f64, 100f64);

        let ratio = crate::bss_lcf_ratio(sum, b, s1, s2);
        let b_size = sum * ratio;
        let s1_size = b_size * b.0 / s1.0;
        let s2_size = s1_size * s1.0 / s2.0;
        let b_size = format!("{:.2}", b_size);
        let s1_size = format!("{:.2}", s1_size);
        let s2_size = format!("{:.2}", s2_size);

        let float_err = "float deparse error";
        // panic!("failed: [{ratio}]");
        let b_size = b_size.parse::<f64>().expect(float_err);
        let s1_size = s1_size.parse::<f64>().expect(float_err);
        let s2_size = s2_size.parse::<f64>().expect(float_err);
        if (b_size <= b.1) && (s1_size <= s1.1) && (s2_size <= s2.1) {
            return;
        }
        panic!("failed: [{b_size},{s1_size},{s2_size}]");
    }

    #[test]
    fn test_bbs_chance() {
        use crate::{Action, TickerInfo};

        let _b1 = (1.6f64, 100f64);
        let _b2 = (82f64, 80f64);
        let _s = (128f64, 50f64);

        let b1 = {
            let mut t = TickerInfo::default();
            t.symbol.best_ask = "1.6".to_string();
            t.symbol.best_ask_size = "100".to_string();
            t.symbol.best_bid = "1.6".to_string();
            t.symbol.best_bid_size = "100".to_string();
            t
        };

        let b2 = {
            let mut t = TickerInfo::default();
            t.symbol.best_ask = "82".to_string();
            t.symbol.best_ask_size = "80".to_string();
            t.symbol.best_bid = "82".to_string();
            t.symbol.best_bid_size = "80".to_string();
            t
        };

        let s = {
            let mut t = TickerInfo::default();
            t.symbol.best_ask = "128".to_string();
            t.symbol.best_ask_size = "50".to_string();
            t.symbol.best_bid = "128".to_string();
            t.symbol.best_bid_size = "50".to_string();
            t
        };

        let res = crate::chance(b1, b2, s);
        if res.is_none() {
            panic!("Chance undetected");
        }
        let res = res.unwrap();
        let err = "wrong action";
        if res.get(0).unwrap().action.ne(&Action::Buy) {
            panic!("{err:?}");
        }
        if res.get(1).unwrap().action.ne(&Action::Buy) {
            panic!("{err:?}");
        }
        if res.get(2).unwrap().action.ne(&Action::Sell) {
            panic!("{err:?}");
        }
    }

    #[test]
    fn test_read_ticker() {
        use kucoin_arbitrage::mirror::TickerInfo;

        let t = {
            let mut t = TickerInfo::default();
            t.symbol.best_ask = "0.1".to_string();
            t.symbol.best_ask_size = "1".to_string();
            t.symbol.best_bid = "0.2".to_string();
            t.symbol.best_bid_size = "2".to_string();
            t
        };
        let ((ap, av), (bp, bv)) = t.get_askbid();
        print!("\nreceived value: {ap}, {av}, {bp}, {bv}\n");
        if ap.ne(&0.1) || av.ne(&1f64) || bp.ne(&0.2) || bv.ne(&2f64) {
            panic!("false value");
        }
        if ap.eq(&0.1) && av.eq(&1f64) && bp.eq(&0.2) && bv.eq(&2f64) {
            return;
        }
        panic!("false value");

        // unimplemented!()
    }

    #[test]
    fn test_read_ticker_ugly() {
        use kucoin_arbitrage::mirror::TickerInfo;

        let t = {
            let mut t = TickerInfo::default();
            t.symbol.best_ask = "0.3123213".to_string();
            t.symbol.best_ask_size = "433".to_string();
            t.symbol.best_bid = "0.2127".to_string();
            t.symbol.best_bid_size = "423437".to_string();
            t
        };
        let ((ap, av), (bp, bv)) = t.get_askbid();
        print!("\nreceived value: {ap}, {av}, {bp}, {bv}\n");
        if ap.ne(&0.3123213) || av.ne(&433f64) || bp.ne(&0.2127) || bv.ne(&423437f64) {
            panic!("false value");
        }
        if ap.eq(&0.3123213) && av.eq(&433f64) && bp.eq(&0.2127) && bv.eq(&423437f64) {
            return;
        }
        panic!("false value");

        // unimplemented!()
    }

    #[test]
    fn test_chance_fair() {
        use kucoin_arbitrage::mirror::TickerInfo;

        let eu = {
            let mut t = TickerInfo::default();
            t.symbol.best_ask = "1".to_string();
            t.symbol.best_ask_size = "1".to_string();
            t.symbol.best_bid = "1".to_string();
            t.symbol.best_bid_size = "1".to_string();
            t
        };
        let eb = eu.clone();
        let bu = eu.clone();
        let res = crate::chance(eu, eb, bu);
        if res.is_some() {
            let res = res.unwrap();
            panic!("false alarm, {res:#?}");
        }
    }

    #[test]
    fn test_chance_buy() {
        use kucoin_arbitrage::mirror::TickerInfo;

        let mut eu = {
            let mut t = TickerInfo::default();
            t.symbol.best_ask = "0.1".to_string();
            t.symbol.best_ask_size = "1".to_string();
            t.symbol.best_bid = "0.1".to_string();
            t.symbol.best_bid_size = "1".to_string();
            t
        };

        let eb = eu.clone();
        let bu = eu.clone();
        eu.symbol.best_ask = "0.05".to_string();

        let res = crate::chance(eu, eb, bu);
        if res.is_none() {
            panic!("wrong, it should buy");
        }
    }
}
