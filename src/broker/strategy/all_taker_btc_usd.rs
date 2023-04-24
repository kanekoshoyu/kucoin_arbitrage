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
pub async fn task_pub_chance_all_taker_btc_usd(
    mut receiver: Receiver<OrderbookEvent>,
    _sender: Sender<ChanceEvent>,
    local_full_orderbook: Arc<Mutex<FullOrderbook>>,
) -> Result<(), kucoin_rs::failure::Error> {
    let btc = String::from("BTC");
    let usd = String::from("USDT");
    let btc_usd = std::format!("{btc}-{usd}");
    loop {
        let event = receiver.recv().await?;
        // log::info!("received orderbook_update");
        let eth: Option<String>;
        match event {
            OrderbookEvent::OrderbookChangeReceived((symbol, _delta)) => {
                if symbol == btc_usd {
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
        let eth_usd = std::format!("{eth}-{usd}");

        // TODO test below
        // log::info!("Analysing {btc_usd},{eth_btc},{eth_usd}");

        let full_orderbook = local_full_orderbook.lock().unwrap();
        let orderbook_btc_usd = (*full_orderbook).get(&btc_usd);
        let orderbook_eth_btc = (*full_orderbook).get(&eth_btc);
        let orderbook_eth_usd = (*full_orderbook).get(&eth_usd);
        if orderbook_btc_usd.is_none() || orderbook_eth_btc.is_none() || orderbook_eth_usd.is_none()
        {
            log::warn!("empty orderbook");
            continue;
        }
        // log::info!("Full orderbook: \n{full_orderbook:#?}");
        // let orderbook_eth_usd = orderbook_eth_usd.unwrap();

        let mut chance = triangular_chance_sequence(
            orderbook_btc_usd.unwrap(),
            orderbook_eth_btc.unwrap(),
            orderbook_eth_usd.unwrap(),
            10.0,
        );
        if chance.actions[1].action.eq(&OrderSide::Buy) {
            // bbs
            chance.actions[0].ticker = btc_usd.clone();
            chance.actions[1].ticker = eth_btc.clone();
            chance.actions[2].ticker = eth_usd.clone();
        } else {
            // bss
            chance.actions[0].ticker = eth_usd.clone();
            chance.actions[1].ticker = eth_btc.clone();
            chance.actions[2].ticker = btc_usd.clone();
        }

        // found profitable chance
        if chance.profit > OrderedFloat(0.0) {
            log::info!("profit: {}", chance.profit.into_inner());
            log::info!("full_orderbook: \n{:#?}", (*full_orderbook));
            log::info!("chance \n{chance:#?}");
            panic!()
        }
    }
}

fn triangular_chance_sequence(
    orderbook_btc_usd: &Orderbook,
    orderbook_eth_btc: &Orderbook,
    orderbook_eth_usd: &Orderbook,
    usd_amount: f64,
) -> TriangularArbitrageChance {
    let (btc_usd_bid, btc_usd_bid_volume) = orderbook_btc_usd.bid.last_key_value().unwrap();
    let (eth_btc_bid, eth_btc_bid_volume) = orderbook_eth_btc.bid.last_key_value().unwrap();
    let (eth_usd_bid, eth_usd_bid_volume) = orderbook_eth_usd.bid.last_key_value().unwrap();
    let (btc_usd_ask, btc_usd_ask_volume) = orderbook_btc_usd.ask.first_key_value().unwrap();
    let (eth_btc_ask, eth_btc_ask_volume) = orderbook_eth_btc.ask.first_key_value().unwrap();
    let (eth_usd_ask, eth_usd_ask_volume) = orderbook_eth_usd.ask.first_key_value().unwrap();

    triangular_chance_sequence_f64(
        PairProfile {
            ask: btc_usd_ask.into_inner(),
            ask_volume: btc_usd_ask_volume.into_inner(),
            bid: btc_usd_bid.into_inner(),
            bid_volume: btc_usd_bid_volume.into_inner(),
            quote_available: usd_amount,
            trading_min: 0.01,
            trading_increment: 0.01,
            trading_fee: 0.001
        },
        PairProfile {
            ask: eth_btc_ask.into_inner(),
            ask_volume: eth_btc_ask_volume.into_inner(),
            bid: eth_btc_bid.into_inner(),
            bid_volume: eth_btc_bid_volume.into_inner(),
            quote_available: 0.0,
            trading_min: 0.01,
            trading_increment: 0.01,
            trading_fee: 0.001
        },
        PairProfile {
            ask: eth_usd_ask.into_inner(),
            ask_volume: eth_usd_ask_volume.into_inner(),
            bid: eth_usd_bid.into_inner(),
            bid_volume: eth_usd_bid_volume.into_inner(),
            quote_available: 0.0,
            trading_min: 0.01,
            trading_increment: 0.01,
            trading_fee: 0.001
        },
    )
}

fn round_amount(amount: f64, increment: f64) -> f64 {
    (amount / increment).round() * increment
}

// Struct for easier parsing of the pair
struct PairProfile {
    ask: f64,
    ask_volume: f64,
    bid: f64,
    bid_volume: f64,
    // amount of USD in BTC/USD
    quote_available: f64,
    _trading_min: f64,
    trading_increment: f64,
    _trading_fee: f64,
}

/// internal chance function, stripped down for doctest
/// >>> triangular_chance_sequence_f64()
///
fn triangular_chance_sequence_f64(
    btc_usd: PairProfile,
    eth_btc: PairProfile,
    eth_usd: PairProfile,
) -> TriangularArbitrageChance {
    // TODO for higher accuracy at high volume trading, use the PairProfile trading fee when its query is done 
    let fee: f64 = 0.001;
    let weight: f64 = 1.0 - fee;

    // Calculate the maximum amount of base currency that can be traded for each sequence
    let max_usd_bbs: f64 = btc_usd
        .ask_volume
        .min(eth_btc.ask_volume * btc_usd.bid)
        .min(eth_usd.bid_volume * eth_btc.ask);
    let max_usd_bss: f64 = eth_usd
        .ask_volume
        .min(eth_btc.bid_volume * eth_usd.bid)
        .min(btc_usd.bid_volume * eth_btc.bid);

    // Use the minimum between available base currency and the maximum amount allowed by the volumes
    let usd_before_bbs = btc_usd.quote_available.min(max_usd_bbs);
    let usd_before_bss = btc_usd.quote_available.min(max_usd_bss);

    let trade_amounts_bbs_0 = round_amount(usd_before_bbs, btc_usd.trading_increment);
    let trade_amounts_bbs_1 = round_amount(
        trade_amounts_bbs_0 * weight / btc_usd.ask,
        eth_btc.trading_increment,
    );
    let trade_amounts_bbs_2 = round_amount(
        trade_amounts_bbs_1 * weight / eth_btc.bid,
        eth_usd.trading_increment,
    );

    let trade_amounts_bss_0 = round_amount(usd_before_bss, eth_usd.trading_increment);
    let trade_amounts_bss_1 = round_amount(
        trade_amounts_bss_0 * weight / eth_usd.ask,
        eth_btc.trading_increment,
    );
    let trade_amounts_bss_2 = round_amount(
        trade_amounts_bss_1 * weight / eth_btc.bid,
        btc_usd.trading_increment,
    );

    // Calculate the net profit of each sequence after accounting for fees and volume constraints
    // let usd_after_bbs = usd_before_bbs / btc_usd.ask / eth_btc_ask * eth_usd_bid * pow(weight, 3);
    // let usd_after_bss = usd_before_bss / eth_usd.ask / eth_btc.bid * btc_usd_bid * pow(weight, 3);
    let usd_after_bbs = trade_amounts_bbs_0 / btc_usd.ask * trade_amounts_bbs_1 * (1.0 - fee)
        / eth_btc.ask
        * eth_usd.bid
        * (1.0 - fee);
    let usd_after_bss = trade_amounts_bss_0 / eth_usd.ask * trade_amounts_bss_1 / eth_btc.bid
        * btc_usd.bid
        * pow(weight, 3);

    // Calculate profit in base currency
    let profit_bbs = usd_after_bbs - usd_before_bbs;
    let profit_bss = usd_after_bss - usd_before_bss;
    log::info!("profit_bbs: {profit_bbs}");
    log::info!("profit_bss: {profit_bss}");

    // Calculate the amounts to trade for each step of the sequence
    if profit_bbs >= profit_bss {
        return TriangularArbitrageChance {
            profit: OrderedFloat(profit_bbs),
            actions: [
                ActionInfo::buy(OrderedFloat(btc_usd.ask), OrderedFloat(trade_amounts_bbs_0)),
                ActionInfo::buy(OrderedFloat(eth_btc.ask), OrderedFloat(trade_amounts_bbs_1)),
                ActionInfo::sell(OrderedFloat(eth_usd.bid), OrderedFloat(trade_amounts_bbs_2)),
            ],
        };
    }
    TriangularArbitrageChance {
        profit: OrderedFloat(profit_bss),
        actions: [
            ActionInfo::buy(OrderedFloat(eth_usd.ask), OrderedFloat(trade_amounts_bss_0)),
            ActionInfo::sell(OrderedFloat(eth_btc.bid), OrderedFloat(trade_amounts_bss_1)),
            ActionInfo::sell(OrderedFloat(btc_usd.bid), OrderedFloat(trade_amounts_bss_2)),
        ],
    }
}
