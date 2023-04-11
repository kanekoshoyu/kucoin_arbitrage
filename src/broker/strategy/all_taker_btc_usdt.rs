use crate::event::{chance::ChanceEvent, orderbook::OrderbookEvent};
use crate::model::chance::{ActionInfo, TriangularArbitrageChance};
use crate::model::orderbook::{FullOrderbook, PVMap};
use crate::strings::{merge_symbol, split_symbol};
use tokio::sync::broadcast::{Receiver, Sender};
use ordered_float::OrderedFloat;
use std::cmp::{max, min};
use std::sync::{Arc, Mutex};

/// Async Task to subscribe to hte websocket events, calculate chances,  
pub async fn task_pub_chance_all_taker_btc_usdt(
    mut receiver: Receiver<OrderbookEvent>,
    sender: Sender<ChanceEvent>,
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

        let full_orderbook = local_full_orderbook.lock().unwrap();
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

        let bss = bss_chance(
            ab.clone(),
            ta.clone(),
            tb.clone(),
            abo.ask.clone(),
            tao.bid.clone(),
            tbo.bid.clone(),
        );

        let bbs = bss_chance(
            tb.clone(),
            ta.clone(),
            ab.clone(),
            tbo.ask.clone(),
            tao.ask.clone(),
            abo.bid.clone(),
        );

        // TODO check profit for bss, bbs,
        log::info!("BSS profit: {}", bss.profit);
        log::info!("BBS profit: {}", bbs.profit);
        let best = max(bss, bbs);

        if best.profit > OrderedFloat(0.0) {
            // TODO publish
            let chance = ChanceEvent::AllTaker(best);
            log::info!("{chance:?}");
            let _res = sender.send(chance);
        }
    }
}

/// get the BSS chance
/// Uses PVMap instead of price-volume pair to give higher flexibility in implementation changes
fn bss_chance(
    ask_symbol: String,
    bid1_symbol: String,
    bid2_symbol: String,
    ask: PVMap,
    bid1: PVMap,
    bid2: PVMap,
) -> TriangularArbitrageChance {
    log::info!("Getting the BBS chance");

    // best buy
    let (best_ask_price, best_ask_volume) = ask.last_key_value().unwrap();
    let (best_bid1_price, best_bid1_volume) = bid1.first_key_value().unwrap();
    let (best_bid2_price, best_bid2_volume) = bid2.first_key_value().unwrap();

    // TODO get actual transaction fee
    let trade_fee = 0.1;
    // TODO get min size and increment

    // ETH-USDT, ETH-BTC, BTC-USDT
    let v_usdt_old = OrderedFloat(10.0); // arbitrary limit

    // Buy, min() requires Ord, keep using OrderedFloat<f32>
    let v_eth = best_ask_price * min(v_usdt_old, best_ask_volume.to_owned()) * (1.0 - trade_fee);
    // Sell
    let v_btc = min(v_eth / best_bid1_price, best_bid1_volume.to_owned()) * (1.0 - trade_fee);
    // Sell
    let v_usdt_new = min(v_btc / best_bid2_price, best_bid2_volume.to_owned()) * (1.0 - trade_fee);

    // TODO reduce the size to fit the min_size and increment
    let profit = v_usdt_new - v_usdt_old;

    // TODO Double check, when selling do we use the quote or base coin
    TriangularArbitrageChance {
        profit,
        actions: [
            ActionInfo::buy(ask_symbol, v_eth),
            ActionInfo::sell(bid1_symbol, v_btc),
            ActionInfo::sell(bid2_symbol, v_usdt_new),
        ],
    }
}

// TODO modify from the code below
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

/// get the BBS chance
/// Uses PVMap instead of price-volume pair to give higher flexibility in implementation changes
fn bbs_chance(
    ask1_symbol: String,
    ask2_symbol: String,
    bid_symbol: String,
    ask1: PVMap,
    ask2: PVMap,
    bid: PVMap,
) -> TriangularArbitrageChance {
    log::info!("Getting the BBS chance");

    // best buy
    let (best_ask1_price, best_ask1_volume) = ask1.last_key_value().unwrap();
    let (best_ask2_price, best_ask2_volume) = ask2.first_key_value().unwrap();
    let (best_bid_price, best_bid_volume) = bid.first_key_value().unwrap();

    // TODO do everything mentioned as of bbs_chance
    let trade_fee = 0.1;
    // TODO get min size and increment

    // BTC-USDT, ETH-BTC, ETH-USDT
    let v_usdt_old = OrderedFloat(10.0); // arbitrary limit

    // Buy, min() requires Ord, keep using OrderedFloat<f32>
    let v_btc = best_ask1_price * min(v_usdt_old, best_ask1_volume.to_owned()) * (1.0 - trade_fee);
    // Buy
    let v_eth = best_ask2_price * min(v_btc, best_ask2_volume.to_owned()) * (1.0 - trade_fee);
    // Sell
    let v_usdt_new = min(v_btc / best_bid_price, best_bid_volume.to_owned()) * (1.0 - trade_fee);

    // TODO reduce the size to fit the min_size and increment
    let profit = v_usdt_new - v_usdt_old;

    // TODO Double check, when selling do we use the quote or base coin
    TriangularArbitrageChance {
        profit,
        actions: [
            ActionInfo::buy(ask1_symbol, v_btc),
            ActionInfo::buy(ask2_symbol, v_eth),
            ActionInfo::sell(bid_symbol, v_usdt_new),
        ],
    }
}
