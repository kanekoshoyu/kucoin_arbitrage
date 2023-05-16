use crate::event::{chance::ChanceEvent, orderbook::OrderbookEvent};
use crate::model::chance::{ActionInfo, TriangularArbitrageChance};
use crate::model::orderbook::{FullOrderbook, Orderbook};
use crate::model::symbol::SymbolInfo;
use crate::strings::split_symbol;
use ordered_float::OrderedFloat;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Mutex;

/// Async Task to subscribe to hte websocket events, calculate chances,  
pub async fn task_pub_chance_all_taker_btc_usd(
    mut receiver: Receiver<OrderbookEvent>,
    _sender: Sender<ChanceEvent>,
    local_full_orderbook: Arc<Mutex<FullOrderbook>>,
    symbol_map: Arc<Mutex<BTreeMap<String, SymbolInfo>>>,
) -> Result<(), kucoin_api::failure::Error> {
    let btc = String::from("BTC");
    let usd = String::from("USDT");
    let btc_usd = std::format!("{btc}-{usd}");
    let usd_budget = 10.0;
    loop {
        let event = receiver.recv().await?;
        // log::info!("received orderbook_update");
        let eth: Option<String> = match event {
            OrderbookEvent::OrderbookChangeReceived((symbol, _delta)) => {
                if symbol == btc_usd {
                    continue;
                }
                let (coin, _) = split_symbol(symbol).unwrap();
                Some(coin)
            }
            _ => {
                log::error!("Unrecognised event {event:?}");
                continue;
            }
        };
        let eth = eth.unwrap();
        let eth_btc = std::format!("{eth}-{btc}");
        let eth_usd = std::format!("{eth}-{usd}");

        // get orderbook
        let full_orderbook = local_full_orderbook.lock().await;
        let orderbook_btc_usd = (*full_orderbook).get(&btc_usd);
        if orderbook_btc_usd.is_none() {
            log::warn!("empty orderbook [{}]", btc_usd);
            continue;
        }
        let orderbook_eth_btc = (*full_orderbook).get(&eth_btc);
        if orderbook_eth_btc.is_none() {
            log::warn!("empty orderbook [{}]", eth_btc);
            continue;
        }
        let orderbook_eth_usd = (*full_orderbook).get(&eth_usd);
        if orderbook_eth_usd.is_none() {
            log::warn!("empty orderbook [{}]", eth_usd);
            continue;
        }

        // clone symbol info from Arc Mutex
        let (info_btc_usd, info_eth_btc, info_eth_usd) = {
            let symbol_map = symbol_map.lock().await;
            (
                symbol_map.get(&btc_usd).unwrap().clone(),
                symbol_map.get(&eth_btc).unwrap().clone(),
                symbol_map.get(&eth_usd).unwrap().clone(),
            )
        };

        let chance = triangular_chance_sequence(
            info_btc_usd,
            info_eth_btc,
            info_eth_usd,
            orderbook_btc_usd.unwrap(),
            orderbook_eth_btc.unwrap(),
            orderbook_eth_usd.unwrap(),
            usd_budget,
        );

        if chance.is_none() {
            continue;
        }
        let chance = chance.unwrap();

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
    info_btc_usd: SymbolInfo,
    info_eth_btc: SymbolInfo,
    info_eth_usd: SymbolInfo,
    orderbook_btc_usd: &Orderbook,
    orderbook_eth_btc: &Orderbook,
    orderbook_eth_usd: &Orderbook,
    usd_amount: f64,
) -> Option<TriangularArbitrageChance> {
    let (btc_usd_bid, btc_usd_bid_volume) = orderbook_btc_usd.bid.last_key_value().unwrap();
    let (eth_btc_bid, eth_btc_bid_volume) = orderbook_eth_btc.bid.last_key_value().unwrap();
    let (eth_usd_bid, eth_usd_bid_volume) = orderbook_eth_usd.bid.last_key_value().unwrap();
    let (btc_usd_ask, btc_usd_ask_volume) = orderbook_btc_usd.ask.first_key_value().unwrap();
    let (eth_btc_ask, eth_btc_ask_volume) = orderbook_eth_btc.ask.first_key_value().unwrap();
    let (eth_usd_ask, eth_usd_ask_volume) = orderbook_eth_usd.ask.first_key_value().unwrap();

    // This should be obtained from the API
    let trading_fee = 0.001;

    triangular_chance_sequence_f64(
        PairProfile {
            symbol: info_btc_usd.symbol,
            ask: btc_usd_ask.into_inner(),
            ask_volume: btc_usd_ask_volume.into_inner(),
            bid: btc_usd_bid.into_inner(),
            bid_volume: btc_usd_bid_volume.into_inner(),
            quote_available: usd_amount,
            trading_min: *info_btc_usd.base_min,
            trading_increment: *info_btc_usd.base_increment,
            trading_fee,
        },
        PairProfile {
            symbol: info_eth_btc.symbol,
            ask: eth_btc_ask.into_inner(),
            ask_volume: eth_btc_ask_volume.into_inner(),
            bid: eth_btc_bid.into_inner(),
            bid_volume: eth_btc_bid_volume.into_inner(),
            quote_available: 0.0,
            trading_min: *info_eth_btc.base_min,
            trading_increment: *info_eth_btc.base_increment,
            trading_fee,
        },
        PairProfile {
            symbol: info_eth_usd.symbol,
            ask: eth_usd_ask.into_inner(),
            ask_volume: eth_usd_ask_volume.into_inner(),
            bid: eth_usd_bid.into_inner(),
            bid_volume: eth_usd_bid_volume.into_inner(),
            quote_available: 0.0,
            trading_min: *info_eth_usd.base_min,
            trading_increment: *info_eth_usd.base_increment,
            trading_fee,
        },
    )
}

// Struct for easier parsing of the pair
#[derive(Debug)]
struct PairProfile {
    symbol: String,
    ask: f64,
    ask_volume: f64,
    bid: f64,
    bid_volume: f64,
    // amount of USD in BTC/USD
    quote_available: f64,
    trading_min: f64,
    trading_increment: f64,
    trading_fee: f64,
}

/// internal chance function, stripped down for doctest
/// >>> triangular_chance_sequence_f64()
///
fn triangular_chance_sequence_f64(
    btc_usd: PairProfile,
    eth_btc: PairProfile,
    eth_usd: PairProfile,
) -> Option<TriangularArbitrageChance> {
    // verify the PairProfile data inputs
    // log::info!("btc_usd:\n{btc_usd:#?}");
    // log::info!("eth_btc:\n{eth_btc:#?}");
    // log::info!("eth_usd:\n{eth_usd:#?}");

    let usd_amount = btc_usd.quote_available;

    // Buy/Buy/Sell path: USD -> BTC -> ETH -> USD
    let mut bbs_1_btc = usd_amount / btc_usd.ask;
    bbs_1_btc = adjust_amount(
        bbs_1_btc,
        btc_usd.trading_min,
        btc_usd.trading_increment,
        btc_usd.ask_volume,
    );
    if 0.0 == bbs_1_btc {
        return None;
    }

    let mut bbs_2_eth = after_fee(bbs_1_btc, btc_usd.trading_fee) / eth_btc.ask;
    bbs_2_eth = adjust_amount(
        bbs_2_eth,
        eth_btc.trading_min,
        eth_btc.trading_increment,
        eth_btc.ask_volume,
    );
    if 0.0 == bbs_2_eth {
        return None;
    }

    let mut bbs_3_eth = after_fee(bbs_2_eth, eth_btc.trading_fee);
    bbs_3_eth = adjust_amount(
        bbs_3_eth,
        eth_usd.trading_min,
        eth_usd.trading_increment,
        eth_usd.bid_volume,
    );
    if 0.0 == bbs_3_eth {
        return None;
    }

    let profit_bbs =
        bbs_3_eth * eth_usd.bid - after_fee(bbs_3_eth, eth_usd.trading_fee) - usd_amount;

    // Buy/Sell/Sell path: USD -> ETH -> BTC -> USD
    let mut bss_1_eth = usd_amount / eth_usd.ask;
    bss_1_eth = adjust_amount(
        bss_1_eth,
        eth_usd.trading_min,
        eth_usd.trading_increment,
        eth_usd.ask_volume,
    );
    if 0.0 == bss_1_eth {
        return None;
    }

    let mut bss_2_eth = after_fee(bss_1_eth, eth_usd.trading_fee) * eth_btc.bid;
    bss_2_eth = adjust_amount(
        bss_2_eth,
        eth_btc.trading_min,
        eth_btc.trading_increment,
        eth_btc.bid_volume,
    );
    if 0.0 == bss_2_eth {
        return None;
    }

    let mut bss_3_btc = after_fee(bss_2_eth, eth_btc.trading_fee);
    bss_3_btc = adjust_amount(
        bss_3_btc,
        btc_usd.trading_min,
        btc_usd.trading_increment,
        btc_usd.bid_volume,
    );
    if 0.0 == bss_3_btc {
        return None;
    }

    let profit_bss =
        bss_3_btc * btc_usd.bid - after_fee(bss_3_btc, btc_usd.trading_fee) - usd_amount;

    // print profits
    log::info!(
        "{},{},{} [BBS]: {}",
        btc_usd.symbol,
        eth_btc.symbol,
        eth_usd.symbol,
        profit_bbs
    );
    log::info!(
        "{},{},{} [BSS]: {}",
        eth_usd.symbol,
        eth_btc.symbol,
        btc_usd.symbol,
        profit_bss
    );

    // return the max profit chance
    if profit_bbs >= profit_bss {
        // USD -> BTC -> ETH -> USD
        Some(TriangularArbitrageChance {
            profit: OrderedFloat(profit_bbs),
            actions: [
                ActionInfo::buy(
                    btc_usd.symbol,
                    OrderedFloat(btc_usd.ask),
                    OrderedFloat(bbs_1_btc),
                ),
                ActionInfo::buy(
                    eth_btc.symbol,
                    OrderedFloat(eth_btc.ask),
                    OrderedFloat(bbs_2_eth),
                ),
                ActionInfo::sell(
                    eth_usd.symbol,
                    OrderedFloat(eth_usd.bid),
                    OrderedFloat(bbs_3_eth),
                ),
            ],
        })
    } else {
        // USD -> ETH -> BTC -> USD
        Some(TriangularArbitrageChance {
            profit: OrderedFloat(profit_bss),
            actions: [
                ActionInfo::buy(
                    eth_usd.symbol,
                    OrderedFloat(eth_usd.ask),
                    OrderedFloat(bss_1_eth),
                ),
                ActionInfo::sell(
                    eth_btc.symbol,
                    OrderedFloat(eth_btc.bid),
                    OrderedFloat(bss_2_eth),
                ),
                ActionInfo::sell(
                    btc_usd.symbol,
                    OrderedFloat(btc_usd.bid),
                    OrderedFloat(bss_3_btc),
                ),
            ],
        })
    }
}

/// rounds the trade volume based on mimimum, increment and the avaiable volume
/// use 'if else' rather than 'min' as f64 does not impelment Ord
/// ```
/// use kucoin_arbitrage::strategy::all_taker_btc_usd::adjust_amount;
/// assert_eq!(adjust_amount(10.0, 1.0, 0.1, 10.0), 10.0);
/// assert_eq!(adjust_amount(10.0, 5.0, 0.1, 10.0), 10.0);
/// assert_eq!(adjust_amount(10.0, 20.0, 0.1, 10.0), 0.0);
/// assert_eq!(adjust_amount(10.0, 1.0, 0.1, 5.0), 5.0);
/// assert_eq!(adjust_amount(3.14, 1.0, 0.5, 5.0), 3.0);
/// ```
pub fn adjust_amount(amount: f64, minimum: f64, increment: f64, available: f64) -> f64 {
    // round amount to the multiple of increment
    let amount = (amount / increment).floor() * increment;
    if amount < minimum {
        // less than minimum tradeable, return 0
        0.0
    } else if amount < available {
        // less than available volume, return avaiable volume
        amount
    } else {
        available
    }
}

fn after_fee(amount: f64, fee: f64) -> f64 {
    amount - amount * fee
}
