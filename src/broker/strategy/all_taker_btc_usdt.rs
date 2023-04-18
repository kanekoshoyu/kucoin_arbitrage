use crate::event::{chance::ChanceEvent, orderbook::OrderbookEvent};
use crate::model::chance::{ActionInfo, ThreeActions, TriangularArbitrageChance};
use crate::model::order::OrderSide;
use crate::model::orderbook::{FullOrderbook, Orderbook, PVMap};
use crate::strings::split_symbol;
use num_traits::pow;
use ordered_float::OrderedFloat;
use std::cmp::{max, min};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast::{Receiver, Sender};

/// Async Task to subscribe to hte websocket events, calculate chances,  
pub async fn task_pub_chance_all_taker_btc_usdt(
    mut receiver: Receiver<OrderbookEvent>,
    sender: Sender<ChanceEvent>,
    local_full_orderbook: Arc<Mutex<FullOrderbook>>,
) -> Result<(), kucoin_rs::failure::Error> {
    let btc = String::from("BTC");
    let usdt = String::from("USDT");
    let btc_usdt = std::format!("{btc}-{usdt}");
    loop {
        let event = receiver.recv().await?;
        // log::info!("received orderbook_update");
        let mut eth: Option<String> = None;
        match event {
            OrderbookEvent::OrderbookChangeReceived((symbol, _delta)) => {
                if symbol == btc_usdt {
                    continue;
                }
                let (coin, _) = split_symbol(symbol).unwrap();
                eth = Some(coin);
            }
            _ => {
                log::error!("Unrecognised event {event:?}");
                continue;
            }
        }
        let eth = eth.unwrap();
        let eth_btc = std::format!("{eth}-{btc}");
        let eth_usdt = std::format!("{eth}-{usdt}");

        // TODO test below
        // log::info!("Analysing {btc_usdt},{eth_btc},{eth_usdt}");

        let full_orderbook = local_full_orderbook.lock().unwrap();
        let orderbook_btc_usdt = (*full_orderbook).get(&btc_usdt);
        let orderbook_eth_btc = (*full_orderbook).get(&eth_btc);
        let orderbook_eth_usdt = (*full_orderbook).get(&eth_usdt);
        if orderbook_btc_usdt.is_none()
            || orderbook_eth_btc.is_none()
            || orderbook_eth_usdt.is_none()
        {
            log::warn!("empty orderbook");
            continue;
        }
        // log::info!("Full orderbook: \n{full_orderbook:#?}");
        // let orderbook_eth_usdt = orderbook_eth_usdt.unwrap();

        let chance = triangular_chance_sequence(
            orderbook_btc_usdt.unwrap(),
            orderbook_eth_btc.unwrap(),
            orderbook_eth_usdt.unwrap(),
            10.0,
        );
        if let Some(mut chance) = chance {
            if chance.actions[1].action.eq(&OrderSide::Buy) {
                // bbs
                chance.actions[0].ticker = btc_usdt.clone();
                chance.actions[1].ticker = eth_btc.clone();
                chance.actions[2].ticker = eth_usdt.clone();
            } else {
                chance.actions[0].ticker = eth_usdt.clone();
                chance.actions[1].ticker = eth_btc.clone();
                chance.actions[2].ticker = btc_usdt.clone();
            }
            log::info!("chance: {chance:?}");
        }

        // TODO check profit for bss, bbs,

        // if best.profit > OrderedFloat(0.0) {
        //     // TODO publish
        //     let chance = ChanceEvent::AllTaker(best);
        //     log::info!("{chance:?}");
        //     let _res = sender.send(chance);
        // }
    }
}

fn triangular_chance_sequence(
    orderbook_btc_usdt: &Orderbook,
    orderbook_eth_btc: &Orderbook,
    orderbook_eth_usdt: &Orderbook,
    usdt_amount: f64,
) -> Option<TriangularArbitrageChance> {
    let (btc_usdt_bid, btc_usdt_bid_volume) = orderbook_btc_usdt.bid.last_key_value().unwrap();
    let (eth_btc_bid, eth_btc_bid_volume) = orderbook_eth_btc.bid.last_key_value().unwrap();
    let (eth_usdt_bid, eth_usdt_bid_volume) = orderbook_eth_usdt.bid.last_key_value().unwrap();
    let (btc_usdt_ask, btc_usdt_ask_volume) = orderbook_btc_usdt.ask.first_key_value().unwrap();
    let (eth_btc_ask, eth_btc_ask_volume) = orderbook_eth_btc.ask.first_key_value().unwrap();
    let (eth_usdt_ask, eth_usdt_ask_volume) = orderbook_eth_usdt.ask.first_key_value().unwrap();

    triangular_chance_sequence_f64(
        btc_usdt_bid.into_inner(),
        btc_usdt_ask.into_inner(),
        eth_usdt_bid.into_inner(),
        eth_usdt_ask.into_inner(),
        eth_btc_bid.into_inner(),
        eth_btc_ask.into_inner(),
        btc_usdt_bid_volume.into_inner(),
        btc_usdt_ask_volume.into_inner(),
        eth_usdt_bid_volume.into_inner(),
        eth_usdt_ask_volume.into_inner(),
        eth_btc_bid_volume.into_inner(),
        eth_btc_ask_volume.into_inner(),
        usdt_amount,
    )
}

fn triangular_chance_sequence_f64(
    btc_usdt_bid: f64,
    btc_usdt_ask: f64,
    eth_usdt_bid: f64,
    eth_usdt_ask: f64,
    eth_btc_bid: f64,
    eth_btc_ask: f64,
    btc_usdt_bid_volume: f64,
    btc_usdt_ask_volume: f64,
    eth_usdt_bid_volume: f64,
    eth_usdt_ask_volume: f64,
    eth_btc_bid_volume: f64,
    eth_btc_ask_volume: f64,
    usdt_amount: f64,
) -> Option<TriangularArbitrageChance> {
    let fee = 0.001;
    let weight = 1.0 - fee;

    // Calculate the maximum amount of base currency that can be traded for each sequence
    let max_base_currency_bbs = btc_usdt_ask_volume
        .min(eth_btc_ask_volume * btc_usdt_bid)
        .min(eth_usdt_bid_volume * eth_btc_ask);
    let max_base_currency_bss = eth_usdt_ask_volume
        .min(eth_btc_bid_volume * eth_usdt_bid)
        .min(btc_usdt_bid_volume * eth_btc_bid);

    // Use the minimum between available base currency and the maximum amount allowed by the volumes
    let usdt_before_bbs = usdt_amount.min(max_base_currency_bbs);
    let usdt_before_bss = usdt_amount.min(max_base_currency_bss);

    // Calculate the net profit of each sequence after accounting for fees and volume constraints
    let usdt_after_bbs =
        usdt_before_bbs / btc_usdt_ask / eth_btc_ask * eth_usdt_bid * pow(weight, 3);
    let usdt_after_bss =
        usdt_before_bss / eth_usdt_ask / eth_btc_bid * btc_usdt_bid * pow(weight, 3);

    // Calculate profit in base currency
    let profit_bbs = usdt_after_bbs - usdt_before_bbs;
    let profit_bss = usdt_after_bss - usdt_after_bss;

    // Calculate the amounts to trade for each step of the sequence
    if profit_bbs > 0.0 && profit_bbs > profit_bss {
        let trade_amounts_bbs = (
            usdt_before_bbs,
            usdt_before_bbs * weight / btc_usdt_ask,
            usdt_before_bbs * weight / btc_usdt_ask * weight / eth_btc_ask,
        );
        return Some(TriangularArbitrageChance {
            profit: OrderedFloat(profit_bbs),
            actions: [
                ActionInfo::buy(OrderedFloat(trade_amounts_bbs.0)),
                ActionInfo::buy(OrderedFloat(trade_amounts_bbs.1)),
                ActionInfo::sell(OrderedFloat(trade_amounts_bbs.2)),
            ],
        });
    }
    if profit_bss > 0.0 && profit_bbs < profit_bss {
        let trade_amounts_bss = (
            usdt_before_bss,
            usdt_before_bss * weight / eth_usdt_ask,
            usdt_before_bss * weight / eth_usdt_ask * weight / eth_btc_bid,
        );
        return Some(TriangularArbitrageChance {
            profit: OrderedFloat(profit_bss),
            actions: [
                ActionInfo::buy(OrderedFloat(trade_amounts_bss.0)),
                ActionInfo::buy(OrderedFloat(trade_amounts_bss.1)),
                ActionInfo::sell(OrderedFloat(trade_amounts_bss.2)),
            ],
        });
    }
    None
}
