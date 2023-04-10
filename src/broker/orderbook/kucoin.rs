use crate::event::orderbook::OrderbookEvent;
use crate::globals::performance;
use crate::model::orderbook::FullOrderbook;
use crate::translator::translator::OrderBookChangeTranslator;
use kucoin_rs::futures::TryStreamExt;
use kucoin_rs::kucoin::{model::websocket::KucoinWebsocketMsg, websocket::KucoinWebsocket};
use kucoin_rs::tokio::sync::broadcast::{Receiver, Sender};
use std::sync::{Arc, Mutex};
//TODO implement the internal trade order task in kucoin

/// Task to puiblish orderbook events from websocket api output
pub async fn task_pub_orderevent(
    mut ws: KucoinWebsocket,
    sender: Sender<OrderbookEvent>,
) -> Result<(), kucoin_rs::failure::Error> {
    let serial = 0;
    loop {
        let msg = ws.try_next().await?.unwrap();
        // add matches for multi-subscribed sockets handling
        if let KucoinWebsocketMsg::OrderBookMsg(msg) = msg {
            let (str, data) = msg.data.to_internal(serial);
            // info!("L2 recceived {str:#?}\n{data:#?}");
            let event = OrderbookEvent::OrderbookChangeReceived((str, data));
            sender.send(event).unwrap();
        } else if let KucoinWebsocketMsg::TickerMsg(msg) = msg {
            log::info!("TickerMsg: {msg:#?}")
        } else if let KucoinWebsocketMsg::OrderBookChangeMsg(msg) = msg {
            log::info!("OrderbookChange: {msg:#?}")
        } else if let KucoinWebsocketMsg::WelcomeMsg(_msg) = msg {
            log::info!("Connection setup")
        } else if let KucoinWebsocketMsg::PongMsg(_msg) = msg {
            log::info!("Connection maintained")
        } else {
            log::info!("Irrelevant Messages");
            log::info!("{msg:#?}")
        }
    }
}

/// Task to puiblish orderbook events from websocket api output
pub async fn task_sync_orderbook(
    mut receiver: Receiver<OrderbookEvent>,
    local_full_orderbook: Arc<Mutex<FullOrderbook>>,
) -> Result<(), kucoin_rs::failure::Error> {
    loop {
        let event = receiver.recv().await?;
        performance::increment();
        let mut full_orderbook = local_full_orderbook.lock().unwrap();
        match event {
            OrderbookEvent::OrderbookReceived((symbol, orderbook)) => {
                (*full_orderbook).insert(symbol.clone(), orderbook);
                log::info!("Initialised Orderbook for {symbol}")
            }
            OrderbookEvent::OrderbookChangeReceived((symbol, orderbook_change)) => {
                let orderbook = (*full_orderbook).get_mut(&symbol);
                if orderbook.is_none() {
                    log::warn!("received {symbol} but orderbook not initialised yet.");
                    // REST Orderbook should be loaded before syncing with WebSocket OrderbookChange
                    continue;
                }
                log::info!("insertion");
                if let Err(()) = orderbook.unwrap().merge(orderbook_change) {
                    log::error!("Merge conflict")
                }
            }
        }
    }
}
