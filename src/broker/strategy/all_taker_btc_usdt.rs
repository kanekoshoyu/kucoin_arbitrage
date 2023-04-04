use crate::event::{chance::ChanceEvent, orderbook::OrderbookEvent};
use crate::model::chance::{ActionInfo, ThreeActions};
use crate::model::order::OrderSide;
use crate::model::orderbook::FullOrderbook;
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
            OrderbookEvent::OrderbookChangeReceived((symbol, delta)) => {
                if symbol == base_symbol {
                    continue;
                }
                let (coin, _) = split_symbol(symbol).unwrap();
                coin_opt = Some(coin);
            }
            _ => {
                log::error!("Unrecognised event");
                continue;
            }
        }
        let coin = coin_opt.unwrap();
        let ab = merge_symbol(base1.clone(), base2.clone());
        let ta = merge_symbol(coin.clone(), base1.clone());
        let tb = merge_symbol(coin.clone(), base2.clone());

        // TODO test below
        print!("Analysing {ab},{ta},{tb}");

        let mut full_orderbook = local_full_orderbook.lock().unwrap();
        let abo = (*full_orderbook).get(&ab);
        let tao = (*full_orderbook).get(&ta);
        let tbo = (*full_orderbook).get(&tb);
        if abo.is_none() || tao.is_none() || tbo.is_none() {
            continue;
        }

        // "symbol" is obtained, get the arbitrage using the local_full_orderbook
        let bbs: ThreeActions = [
            ActionInfo {
                action: OrderSide::Buy,
                ticker: String::from(""),
                volume: String::from(""),
            },
            ActionInfo {
                action: OrderSide::Buy,
                ticker: String::from(""),
                volume: String::from(""),
            },
            ActionInfo {
                action: OrderSide::Sell,
                ticker: String::from(""),
                volume: String::from(""),
            },
        ];
        sender.send(ChanceEvent::AllTaker(bbs))?;
    }
}
