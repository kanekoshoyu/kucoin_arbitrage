extern crate kucoin_rs;

use kucoin_rs::failure;
use kucoin_rs::futures::TryStreamExt;
use kucoin_rs::kucoin::{
    client::{Credentials, Kucoin, KucoinEnv},
    model::websocket::{KucoinWebsocketMsg, WSTopic, WSType},
    websocket::KucoinWebsocket,
};
use kucoin_rs::tokio::{
    self,
    time::{sleep, Duration},
};

use kucoin_arbitrage::mirror::*;
use kucoin_arbitrage::shared::*;
use lazy_static::lazy_static;
use log::*;
use std::sync::{Arc, Mutex};

// gets the jobs done
// Arc has implicit 'static bound, so it cannot contain reference to local variable.
lazy_static! {
    static ref CONFIG: Arc<Mutex<Config>> = Arc::new(Mutex::new(load_ini()));
    static ref PERFORMANCE: Arc<Mutex<Performance>> =
        Arc::new(Mutex::new(Performance { data_count: 0 }));
}

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // provide logging format
    kucoin_arbitrage::shared::log_init();
    info!("Hello world");

    let c = CONFIG.clone();
    let mg = c.lock().unwrap();
    let credentials = Credentials::new((*mg).api_key, (*mg).secret_key, (*mg).passphrase);
    drop(mg);

    info!("{credentials:#?}");
    let api = Kucoin::new(KucoinEnv::Live, Some(credentials))?;
    let url = api.get_socket_endpoint(WSType::Public).await?;
    let mut ws = api.websocket();

    let subs = vec![WSTopic::Ticker(vec![
        "ETH-BTC".to_string(),
        "BTC-USDT".to_string(),
        "ETH-USDT".to_string(),
    ])];
    ws.subscribe(url, subs).await?;

    info!("Async polling");
    let perf = PERFORMANCE.clone();
    let mir = MIRROR.clone();
    tokio::spawn(async move { sync_tickers(ws, perf, mir).await });

    let monitor_delay = {
        let c = CONFIG.clone();
        let mg = c.lock().unwrap();
        let interval_sec: u64 = (*mg).monitor_interval_sec;
        drop(mg);
        Duration::from_secs(interval_sec)
    };
    loop {
        sleep(monitor_delay).await;
        report_status(PERFORMANCE.clone(), CONFIG.clone()).expect("report status error");
    }
}

fn report_status(
    perf: Arc<Mutex<Performance>>,
    conf: Arc<Mutex<Config>>,
) -> Result<(), failure::Error> {
    info!("reporting");
    let p = perf.lock().unwrap();
    let c = conf.lock().unwrap();
    let data_rate = (*p).data_count / (*c).monitor_interval_sec;
    drop(p);
    drop(c);

    info!("Data rate: {data_rate:?} points/sec");
    // clear the data
    {
        let mut p = perf.lock().unwrap();
        (*p).data_count = 0;
    }

    Ok(())
}

use kucoin_arbitrage::mirror::Map;
use kucoin_arbitrage::shared::topic_to_ticker;

async fn sync_tickers(
    mut ws: KucoinWebsocket,
    perf: Arc<Mutex<Performance>>,
    mirror: Arc<Mutex<Map>>,
) -> Result<(), failure::Error> {
    while let Some(msg) = ws.try_next().await? {
        match msg {
            KucoinWebsocketMsg::TickerMsg(msg) => {
                // if the updated data is greater than the ex
                // TODO: optimize the cloning mess here.
                let ticker = topic_to_ticker(msg.topic).expect("wrong topic format");
                let ticker_clone = ticker.clone();
                let (coin1, _coin2) = ticker_to_tuple(ticker_clone).expect("wrong ticker format");
                let ticker_clone = ticker.clone();
                {
                    // update the map
                    let mut m = mirror.lock().unwrap();
                    let tickers: &mut Map = &mut (*m);
                    if let Some(data) = tickers.get_mut(&ticker_clone) {
                        data.symbol = msg.data;
                    } else {
                        tickers.insert(ticker_clone, TickerInfo::new(msg.data));
                    }
                }
                let ab = "BTC-USDT";
                if ticker.eq(ab) {
                    // skip when it is a btc-usdt pair (i.e. ab)
                    continue;
                }
                // either ETC-BTC or ETH-USDT
                let tb = {
                    let mut res = coin1.clone();
                    res.push_str("-USDT");
                    res
                };
                let ta = {
                    let mut res = coin1.clone();
                    res.push_str("-BTC");
                    res
                };
                let ab = ab.to_string();

                info!("studying Triangle: {tb}, {ta}, {ab}");

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

                // TODO: conduct the analysis
                let res = chance(tb, ta, ab);
                if let Some(_sequence) = res {
                    // TODO: calculate the profit ratio
                    info!("found arbitrage chance");
                    // info!("found arbitrage chance: {sequence:#?}");
                }

                {
                    let mut p = perf.lock().unwrap();
                    (*p).data_count += 1;
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

fn chance(
    ticker_target_base: TickerInfo,
    ticker_target_alt: TickerInfo,
    ticker_alt_base: TickerInfo,
) -> Option<ActionSequence> {
    // get both prices
    let tb = ticker_target_base.clone();
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

        let b1 = (1.6f64, 100f64);
        let b2 = (82f64, 80f64);
        let s = (128f64, 50f64);

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
