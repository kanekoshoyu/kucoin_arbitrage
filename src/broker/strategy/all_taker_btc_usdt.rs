#![feature(map_first_last)]
use crate::event::{chance::ChanceEvent, orderbook::OrderbookEvent};
use crate::model::chance::{ActionInfo, TriangularArbitrageChance};
use crate::model::order::OrderSide;
use crate::model::orderbook::{FullOrderbook, PVMap};
use crate::strings::{merge_symbol, split_symbol};
use kucoin_rs::tokio::sync::broadcast::{Receiver, Sender};
use std::sync::{Arc, Mutex};

/// Async Task to subscribe to hte websocket events, calculate chances,  
pub async fn task_pub_chance_all_taker_btc_usdt(
    receiver: &mut Receiver<OrderbookEvent>,
    sender: &mut Sender<ChanceEvent>,
    local_full_orderbook: Arc<Mutex<FullOrderbook>>,
) -> Result<(), kucoin_rs::failure::Error> {
    let base1 = String::from("BTC");
    let base2 = String::from("USDT");
    let base_symbol = String::from("BTC-USDT");
    loop {
        let event = receiver.recv().await?;
        let mut coin_opt: Option<String> = None;
        match event {
            OrderbookEvent::OrderbookChangeReceived((symbol, _delta)) => {
                if symbol == base_symbol {
                    continue;
                }
                let (coin, _) = split_symbol(symbol).unwrap();
                coin_opt = Some(coin);
            }
            _ => {
                log::error!("Unrecognised event {event:?}");
                continue;
            }
        }
        let coin = coin_opt.unwrap();
        let ab = merge_symbol(base1.clone(), base2.clone());
        let ta = merge_symbol(coin.clone(), base1.clone());
        let tb = merge_symbol(coin.clone(), base2.clone());

        // TODO test below
        log::info!("Analysing {ab},{ta},{tb}");

        let mut full_orderbook = local_full_orderbook.lock().unwrap();
        let abo = (*full_orderbook).get(&ab);
        let tao = (*full_orderbook).get(&ta);
        let tbo = (*full_orderbook).get(&tb);
        if abo.is_none() || tao.is_none() || tbo.is_none() {
            log::warn!("empty orderbook");
            continue;
        }

        let abo = abo.unwrap();
        let tao = tao.unwrap();
        let tbo = tbo.unwrap();

        let bss = bss_chance(abo.bid.clone(), tao.ask.clone(), tbo.ask.clone());
        ()
    }
}

///
fn bss_chance(mut bid: PVMap, mut ask1: PVMap, mut ask2: PVMap) -> TriangularArbitrageChance {
    log::info!("{bid:?}");
    // log::info!("{ask1:?}");
    // log::info!("{ask2:?}");

    // best buy
    let (best_bid_price, best_bid_volume) = bid.last_key_value().unwrap();
    let (best_ask1_price, best_ask1_volume) = ask1.first_key_value().unwrap();
    let (best_ask2_price, best_ask2_volume) = ask2.first_key_value().unwrap();

    // TODO setup below
    // get transaction fees
    // get minimum orders
    // get order resolution

    return TriangularArbitrageChance::default();
}

// modify from the code below
/*
fn bss_action_sequence(sum: f64, ticker_info_bss: [TickerInfo; 3]) -> ActionSequence {
    let err_msg = "ticker_info_bss error";
    let [b, s1, s2] = ticker_info_bss;

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

 */
