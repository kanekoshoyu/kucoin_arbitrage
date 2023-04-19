use crate::event::{chance::ChanceEvent, orderbook::OrderbookEvent};
use crate::model::chance::{ActionInfo, TriangularArbitrageChance};
use crate::model::order::OrderSide;
use crate::model::orderbook::{FullOrderbook, Orderbook};
use crate::strings::split_symbol;
use num_traits::pow;
use ordered_float::OrderedFloat;
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

        let mut chance = triangular_chance_sequence(
            orderbook_btc_usdt.unwrap(),
            orderbook_eth_btc.unwrap(),
            orderbook_eth_usdt.unwrap(),
            10.0,
        );
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

        // found profitable chance
        if chance.profit > OrderedFloat(5.0) {
            log::info!("profit: {}", chance.profit.into_inner());
            log::info!("full_orderbook: \n{:#?}", (*full_orderbook));
            log::info!("chance \n{chance:#?}");
            panic!()
        }
    }
}

fn triangular_chance_sequence(
    orderbook_btc_usdt: &Orderbook,
    orderbook_eth_btc: &Orderbook,
    orderbook_eth_usdt: &Orderbook,
    usdt_amount: f64,
) -> TriangularArbitrageChance {
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

fn round_amount(amount: f64, increment: f64) -> f64 {
    (amount / increment).round() * increment
}

/// internal chance function, stripped down for doctest
/// >>> triangular_chance_sequence_f64()
///
fn triangular_chance_sequence_f64(
    btc_usd_bid: f64,
    btc_usd_ask: f64,
    eth_usd_bid: f64,
    eth_usd_ask: f64,
    eth_btc_bid: f64,
    eth_btc_ask: f64,
    btc_usd_bid_volume: f64,
    btc_usd_ask_volume: f64,
    eth_usd_bid_volume: f64,
    eth_usd_ask_volume: f64,
    eth_btc_bid_volume: f64,
    eth_btc_ask_volume: f64,
    usd_amount: f64,
) -> TriangularArbitrageChance {
    let fee: f64 = 0.001;
    let weight: f64 = 1.0 - fee;

    // Calculate the maximum amount of base currency that can be traded for each sequence
    let max_usd_bbs: f64 = btc_usd_ask_volume
        .min(eth_btc_ask_volume * btc_usd_bid)
        .min(eth_usd_bid_volume * eth_btc_ask);
    let max_usd_bss: f64 = eth_usd_ask_volume
        .min(eth_btc_bid_volume * eth_usd_bid)
        .min(btc_usd_bid_volume * eth_btc_bid);

    // Use the minimum between available base currency and the maximum amount allowed by the volumes
    let usd_before_bbs = usd_amount.min(max_usd_bbs);
    let usd_before_bss = usd_amount.min(max_usd_bss);

    let btc_usd_increment = 0.01;
    let eth_btc_increment = 0.001;
    let eth_usd_increment = 0.01;

    let trade_amounts_bbs_0 = round_amount(usd_before_bbs, btc_usd_increment);
    let trade_amounts_bbs_1 = round_amount(
        trade_amounts_bbs_0 * weight / btc_usd_ask,
        eth_btc_increment,
    );
    let trade_amounts_bbs_2 = round_amount(
        trade_amounts_bbs_1 * weight / eth_btc_bid,
        eth_usd_increment,
    );

    let trade_amounts_bss_0 = round_amount(usd_before_bss, eth_usd_increment);
    let trade_amounts_bss_1 = round_amount(
        trade_amounts_bss_0 * weight / eth_usd_ask,
        eth_btc_increment,
    );
    let trade_amounts_bss_2 = round_amount(
        trade_amounts_bss_1 * weight / eth_btc_bid,
        btc_usd_increment,
    );

    // Calculate the net profit of each sequence after accounting for fees and volume constraints
    // let usd_after_bbs = usd_before_bbs / btc_usd_ask / eth_btc_ask * eth_usd_bid * pow(weight, 3);
    // let usd_after_bss = usd_before_bss / eth_usd_ask / eth_btc_bid * btc_usd_bid * pow(weight, 3);
    let usd_after_bbs = trade_amounts_bbs_0 / btc_usd_ask * trade_amounts_bbs_1 * (1.0 - fee) / eth_btc_ask * eth_usd_bid * (1.0 - fee);
    let usd_after_bss = trade_amounts_bss_0 / eth_usd_ask * trade_amounts_bss_1 / eth_btc_bid * btc_usd_bid * pow(weight, 3);

    // Calculate profit in base currency
    let profit_bbs = usd_after_bbs - usd_before_bbs;
    let profit_bss = usd_after_bss - usd_before_bss;
    // log::info!("profit_bbs: {profit_bbs}");
    // log::info!("profit_bss: {profit_bss}");

    // Calculate the amounts to trade for each step of the sequence
    if profit_bbs >= profit_bss {
        return TriangularArbitrageChance {
            profit: OrderedFloat(profit_bbs),
            actions: [
                ActionInfo::buy(OrderedFloat(btc_usd_ask), OrderedFloat(trade_amounts_bbs_0)),
                ActionInfo::buy(OrderedFloat(eth_btc_ask), OrderedFloat(trade_amounts_bbs_1)),
                ActionInfo::sell(OrderedFloat(eth_usd_bid), OrderedFloat(trade_amounts_bbs_2)),
            ],
        };
    }
    TriangularArbitrageChance {
        profit: OrderedFloat(profit_bss),
        actions: [
            ActionInfo::buy(OrderedFloat(eth_usd_ask), OrderedFloat(trade_amounts_bss_0)),
            ActionInfo::sell(OrderedFloat(eth_btc_bid), OrderedFloat(trade_amounts_bss_1)),
            ActionInfo::sell(OrderedFloat(btc_usd_bid), OrderedFloat(trade_amounts_bss_2)),
        ],
    }
}
