use crate::event::{chance::ChanceEvent, orderbook::OrderbookEvent};
use crate::model::chance::{ActionInfo, ThreeActions};
use crate::model::order::OrderSide;
use crate::model::orderbook::FullOrderbook;
use kucoin_rs::tokio::sync::broadcast::{Receiver, Sender};
use std::sync::{Arc, Mutex};

/// Async Task to subscribe to hte websocket events, calculate chances,  
pub async fn task_pub_chance_all_taker_btc_usdt(
    receiver: &mut Receiver<OrderbookEvent>,
    sender: &mut Sender<ChanceEvent>,
    local_full_orderbook: Arc<Mutex<FullOrderbook>>,
) -> Result<(), kucoin_rs::failure::Error> {
    loop {
        let event = receiver.recv().await?;
        let symbol: String;
        if let OrderbookEvent::OrderbookChangeReceived((symbol, _)) = event {
        } else {
            log::info!("Please retry");
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
