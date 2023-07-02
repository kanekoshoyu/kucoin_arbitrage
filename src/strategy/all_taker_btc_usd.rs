use crate::event::{chance::ChanceEvent, orderbook::OrderbookEvent};
use crate::global::counter_helper;
use crate::model::chance::{ActionInfo, TriangularArbitrageChance};
use crate::model::counter::Counter;
use crate::model::orderbook::{FullOrderbook, Orderbook};
use crate::model::symbol::SymbolInfo;
use crate::strings::split_symbol;
use ordered_float::OrderedFloat;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Mutex;

/// Async task to subscribe to hte websocket events, calculate chances,  
pub async fn task_pub_chance_all_taker_btc_usd(
    mut receiver: Receiver<OrderbookEvent>,
    sender: Sender<ChanceEvent>,
    local_full_orderbook: Arc<Mutex<FullOrderbook>>,
    symbol_map: Arc<Mutex<BTreeMap<String, SymbolInfo>>>,
    usd_budget: f64,
    counter: Arc<Mutex<Counter>>,
) -> Result<(), kucoin_api::failure::Error> {
    let btc = String::from("BTC");
    let usd = String::from("USDT");
    let btc_usd = std::format!("{btc}-{usd}");
    loop {
        counter_helper::increment(counter.clone()).await;
        let event = receiver.recv().await?;
        // log::info!("received orderbook_update");
        let alt: Option<String> = match event {
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
        let alt = alt.unwrap();
        let alt_btc = std::format!("{alt}-{btc}");
        let alt_usd = std::format!("{alt}-{usd}");

        // get orderbook
        let full_orderbook = local_full_orderbook.lock().await;
        let orderbook_btc_usd = (*full_orderbook).get(&btc_usd);
        if orderbook_btc_usd.is_none() {
            log::warn!("trying to get from unregistered orderbook [{}]", btc_usd);
            continue;
        }
        let orderbook_alt_btc = (*full_orderbook).get(&alt_btc);
        if orderbook_alt_btc.is_none() {
            log::warn!("trying to get from unregistered orderbook {}]", alt_btc);
            continue;
        }
        let orderbook_alt_usd = (*full_orderbook).get(&alt_usd);
        if orderbook_alt_usd.is_none() {
            log::warn!("trying to get from unregistered orderbook [{}]", alt_usd);
            continue;
        }

        // clone symbol info from Arc Mutex
        let (info_btc_usd, info_alt_btc, info_alt_usd) = {
            let symbol_map = symbol_map.lock().await;
            (
                symbol_map.get(&btc_usd).unwrap().clone(),
                symbol_map.get(&alt_btc).unwrap().clone(),
                symbol_map.get(&alt_usd).unwrap().clone(),
            )
        };

        let chance = triangular_chance_sequence(
            info_btc_usd,
            info_alt_btc,
            info_alt_usd,
            orderbook_btc_usd.unwrap(),
            orderbook_alt_btc.unwrap(),
            orderbook_alt_usd.unwrap(),
            usd_budget,
        );

        if chance.is_none() {
            continue;
        }
        let chance = chance.unwrap();

        // found profitable chance
        if chance.profit > OrderedFloat(0.0) {
            sender.send(ChanceEvent::AllTaker(chance)).unwrap();
        }
    }
}

fn triangular_chance_sequence(
    info_btc_usd: SymbolInfo,
    info_alt_btc: SymbolInfo,
    info_alt_usd: SymbolInfo,
    orderbook_btc_usd: &Orderbook,
    orderbook_alt_btc: &Orderbook,
    orderbook_alt_usd: &Orderbook,
    usd_amount: f64,
) -> Option<TriangularArbitrageChance> {
    // log::info!("TSC: {}", info_alt_btc.base);
    // get the least ask
    let (btc_usd_ask, btc_usd_ask_volume) = orderbook_btc_usd.ask.first_key_value().unwrap();
    let (alt_btc_ask, alt_btc_ask_volume) = orderbook_alt_btc.ask.first_key_value().unwrap();
    let (alt_usd_ask, alt_usd_ask_volume) = orderbook_alt_usd.ask.first_key_value().unwrap();
    // get the largest bid
    let (btc_usd_bid, btc_usd_bid_volume) = orderbook_btc_usd.bid.last_key_value().unwrap();
    let (alt_btc_bid, alt_btc_bid_volume) = orderbook_alt_btc.bid.last_key_value().unwrap();
    let (alt_usd_bid, alt_usd_bid_volume) = orderbook_alt_usd.bid.last_key_value().unwrap();

    // This should be obtained from the API
    let trading_fee = 0.001;

    triangular_chance_sequence_f64(
        PairProfile {
            symbol: info_btc_usd.symbol,
            ask: btc_usd_ask.into_inner(),
            ask_volume: btc_usd_ask_volume.into_inner(),
            bid: btc_usd_bid.into_inner(),
            bid_volume: btc_usd_bid_volume.into_inner(),
            trading_min: *info_btc_usd.base_min,
            trading_increment: *info_btc_usd.base_increment,
            trading_fee,
        },
        PairProfile {
            symbol: info_alt_btc.symbol,
            ask: alt_btc_ask.into_inner(),
            ask_volume: alt_btc_ask_volume.into_inner(),
            bid: alt_btc_bid.into_inner(),
            bid_volume: alt_btc_bid_volume.into_inner(),
            trading_min: *info_alt_btc.base_min,
            trading_increment: *info_alt_btc.base_increment,
            trading_fee,
        },
        PairProfile {
            symbol: info_alt_usd.symbol,
            ask: alt_usd_ask.into_inner(),
            ask_volume: alt_usd_ask_volume.into_inner(),
            bid: alt_usd_bid.into_inner(),
            bid_volume: alt_usd_bid_volume.into_inner(),
            trading_min: *info_alt_usd.base_min,
            trading_increment: *info_alt_usd.base_increment,
            trading_fee,
        },
        usd_amount,
    )
}

// Struct for easier parsing of the pair
#[derive(Debug)]
pub struct PairProfile {
    pub symbol: String,
    pub ask: f64,
    pub ask_volume: f64,
    pub bid: f64,
    pub bid_volume: f64,
    pub trading_min: f64,
    pub trading_increment: f64,
    pub trading_fee: f64,
}

// TODO add more test cases to verify the below functions

/// amount to order (for API use), and amount to obtain (for next use) at best ask
/// ```
/// use kucoin_arbitrage::strategy::all_taker_btc_usd::{buy, PairProfile};
/// let btc_usd = PairProfile { symbol: "BTC-USDT".to_string(), ask: 26875.0, ask_volume: 0.89357531, bid: 26874.9, bid_volume: 2.44038719, trading_min: 1e-5, trading_increment: 1e-8, trading_fee: 0.001 };
/// let (btc_order, btc_bought) = buy(&btc_usd, 10.0);
/// assert_eq!(btc_order, 0.00037209);
/// assert_eq!(btc_bought, 0.00037171791);
/// ```
pub fn buy(profile: &PairProfile, quote_amount: f64) -> (f64, f64) {
    // adjust base for order (after trade at ask, order in base currency)
    let base_amount = adjust_amount(
        quote_amount / profile.ask,
        profile.trading_min,
        profile.trading_increment,
        profile.ask_volume,
    );
    // return both the trade amount as well as the amount to obtain
    (base_amount, base_amount * (1.0 - profile.trading_fee))
}

/// amount to order (for API use), and amount to obtain (for next use) as best bid
/// ```
/// use kucoin_arbitrage::strategy::all_taker_btc_usd::{sell, PairProfile};
/// let btc_usd = PairProfile { symbol: "BTC-USDT".to_string(), ask: 26875.0, ask_volume: 0.89357531, bid: 26874.9, bid_volume: 2.44038719, trading_min: 1e-5, trading_increment: 1e-8, trading_fee: 0.001 };
/// let (btc_order, btc_bought) = sell(&btc_usd, 0.0004);
/// assert_eq!(btc_order, 0.0004);
/// assert_eq!(btc_bought, 10.739210040000001);
/// ```
pub fn sell(profile: &PairProfile, base_amount: f64) -> (f64, f64) {
    // adjust base for order (before trade at bid, order in base currency)
    let base_amount = adjust_amount(
        base_amount,
        profile.trading_min,
        profile.trading_increment,
        profile.bid_volume,
    );
    // return both the trade amount as well as the amount to obtain
    let quote_amount = base_amount * profile.bid;
    (base_amount, quote_amount * (1.0 - profile.trading_fee))
}

/// internal chance function, stripped down for doctest
fn triangular_chance_sequence_f64(
    btc_usd: PairProfile,
    alt_btc: PairProfile,
    alt_usd: PairProfile,
    usd_amount: f64,
) -> Option<TriangularArbitrageChance> {
    // Buy/Buy/Sell path: USD -> BTC -> ALT -> USD
    let (bbs_b_btc_amount, bbs_btc_bought) = buy(&btc_usd, usd_amount);
    let (bbs_b_alt_amount, bbs_alt_bought) = buy(&alt_btc, bbs_btc_bought);
    let (bbs_s_alt_amount, bbs_usd_bought) = sell(&alt_usd, bbs_alt_bought);
    // BBS proft (USD)
    let bbs_profit: f64 = bbs_usd_bought - usd_amount;

    // Buy/Sell/Sell path: USD -> ALT -> BTC -> USD
    let (bss_b_alt_amount, bss_alt_bought) = buy(&alt_usd, usd_amount);
    let (bss_s_alt_amount, bss_btc_bought) = sell(&alt_btc, bss_alt_bought);
    let (bss_s_btc_amount, bss_usd_bought) = sell(&btc_usd, bss_btc_bought);
    // BSS proft (USD)
    let bss_profit: f64 = bss_usd_bought - usd_amount;

    // return BBS
    if bbs_profit > 0.0 && bbs_profit > bss_profit {
        return Some(TriangularArbitrageChance {
            profit: OrderedFloat(bbs_profit),
            actions: [
                ActionInfo::buy(
                    btc_usd.symbol,
                    OrderedFloat(btc_usd.ask),
                    OrderedFloat(bbs_b_btc_amount),
                ),
                ActionInfo::buy(
                    alt_btc.symbol,
                    OrderedFloat(alt_btc.ask),
                    OrderedFloat(bbs_b_alt_amount),
                ),
                ActionInfo::sell(
                    alt_usd.symbol,
                    OrderedFloat(alt_usd.bid),
                    OrderedFloat(bbs_s_alt_amount),
                ),
            ],
        });
    }

    // return BSS
    if bss_profit > 0.0 && bss_profit > bbs_profit {
        return Some(TriangularArbitrageChance {
            profit: OrderedFloat(bss_profit),
            actions: [
                ActionInfo::buy(
                    alt_usd.symbol,
                    OrderedFloat(alt_usd.ask),
                    OrderedFloat(bss_b_alt_amount),
                ),
                ActionInfo::sell(
                    alt_btc.symbol,
                    OrderedFloat(alt_btc.bid),
                    OrderedFloat(bss_s_alt_amount),
                ),
                ActionInfo::sell(
                    btc_usd.symbol,
                    OrderedFloat(btc_usd.bid),
                    OrderedFloat(bss_s_btc_amount),
                ),
            ],
        });
    }

    // No profit
    None
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
