use crate::event::orderbook::OrderbookEvent;
use crate::global::performance;
use crate::model::orderbook::FullOrderbook;
use crate::translator::translator::OrderBookChangeTranslator;
use kucoin_api::futures::TryStreamExt;
use kucoin_api::{model::websocket::KucoinWebsocketMsg, websocket::KucoinWebsocket};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast::{Receiver, Sender};
//TODO implement the internal trade order task in kucoin

/// Task to puiblish orderbook events from websocket api output
pub async fn task_pub_orderbook_event(
    mut ws: KucoinWebsocket,
    sender: Sender<OrderbookEvent>,
) -> Result<(), kucoin_api::failure::Error> {
    let serial = 0;
    loop {
        let msg = ws.try_next().await?.unwrap();
        // add matches for multi-subscribed sockets handling
        if let KucoinWebsocketMsg::OrderBookMsg(msg) = msg {
            // log::info!("WS: {msg:#?}");
            let (str, data) = msg.data.to_internal(serial);
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
    sender: Sender<OrderbookEvent>,
    local_full_orderbook: Arc<Mutex<FullOrderbook>>,
) -> Result<(), kucoin_api::failure::Error> {
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
                // log::info!("insertion: {orderbook_change:#?}");
                match orderbook.unwrap().merge(orderbook_change) {
                    Ok(res) => {
                        if let Some(ob) = res {
                            // log::info!("update: {ob:#?}");
                            sender
                                .send(OrderbookEvent::OrderbookChangeReceived((symbol, ob)))
                                .unwrap();
                        }
                    }
                    Err(e) => {
                        log::error!("Merge conflict: {e}")
                    }
                }
            }
        }
    }
}
