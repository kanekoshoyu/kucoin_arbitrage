use crate::event::orderbook::OrderbookEvent;
use crate::globals::performance;
use crate::model::orderbook::{FullOrderbook, Orderbook};
use crate::translator::translator::OrderBookChangeTranslator;
// external
use kucoin_rs::futures::TryStreamExt;
use kucoin_rs::kucoin::{model::websocket::KucoinWebsocketMsg, websocket::KucoinWebsocket};
use kucoin_rs::tokio::sync::broadcast::{Receiver, Sender};
use std::sync::{Arc, Mutex};
//TODO implement the internal trade order task in kucoin

/// Task to puiblish orderbook events from websocket api output
pub async fn task_pub_orderevent(
    mut ws: KucoinWebsocket,
    sender: Arc<Sender<OrderbookEvent>>,
) -> Result<(), kucoin_rs::failure::Error> {
    let serial = 0;
    loop {
        let msg = ws.try_next().await?.unwrap();
        // add matches for multi-subscribed sockets handling
        if let KucoinWebsocketMsg::OrderBookMsg(msg) = msg {
            let (str, data) = msg.data.to_internal(serial);
            // info!("L2 recceived {str:#?}\n{data:#?}");
            let event = OrderbookEvent::OrderbookReceived((str, data));
            sender.send(event).unwrap();
        } else if let KucoinWebsocketMsg::TickerMsg(_msg) = msg {
            // info!("{msg:#?}")
        } else if let KucoinWebsocketMsg::OrderBookChangeMsg(_msg) = msg {
            // info!("{msg:#?}")
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
    receiver: &mut Receiver<OrderbookEvent>,
    local_full_orderbook: Arc<Mutex<FullOrderbook>>,
) -> Result<(), kucoin_rs::failure::Error> {
    loop {
        let event = receiver.recv().await?;
        performance::increment();
        let mut full_orderbook = local_full_orderbook.lock().unwrap();

        if let OrderbookEvent::OrderbookReceived((symbol, orderbook_change)) = event {
            // merge the local orderbook with this one
            let orderbook = (*full_orderbook).get_mut(&symbol);
            if orderbook.is_none() {
                (*full_orderbook).insert(symbol, orderbook_change);
                // log::info!("Created")
            } else {
                if let Err(()) = orderbook.unwrap().merge(orderbook_change) {
                    log::error!("Merge conflict")
                }
                // log::info!("Inserted")
            }
        }
    }
}
