use crate::event::orderbook::OrderbookEvent;
use crate::global::counter_helper;
use crate::model::counter::Counter;
use crate::model::orderbook::FullOrderbook;
use crate::translator::traits::OrderBookChangeTranslator;
use kucoin_api::futures::TryStreamExt;
use kucoin_api::{model::websocket::KucoinWebsocketMsg, websocket::KucoinWebsocket};
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Mutex;

/// Task to publish orderbook events.
/// Subscribes Websocket API.
/// Publishes OrderbookEvent directly after conversion.
pub async fn task_pub_orderbook_event(
    mut ws: KucoinWebsocket,
    sender: Sender<OrderbookEvent>,
) -> Result<(), kucoin_api::failure::Error> {
    let serial = 0;
    loop {
        let msg = ws.try_next().await;
        if let Err(e) = msg {
            log::error!("task_pub_orderbook_event error: {e}");
            panic!()
        }
        let msg = msg?.unwrap();
        // add matches for multi-subscribed sockets handling
        if let KucoinWebsocketMsg::OrderBookMsg(msg) = msg {
            // log::info!("WS: {msg:#?}");
            let (str, data) = msg.data.to_internal(serial);
            let event = OrderbookEvent::OrderbookChangeReceived((str, data));
            if sender.send(event).is_err() {
                log::error!("Orderbook event publish error, check receiver");
            }
        } else if let KucoinWebsocketMsg::TickerMsg(msg) = msg {
            log::info!("TickerMsg: {msg:#?}")
        } else if let KucoinWebsocketMsg::OrderBookChangeMsg(msg) = msg {
            log::info!("OrderbookChange: {msg:#?}")
        } else if let KucoinWebsocketMsg::WelcomeMsg(_) = msg {
        } else if let KucoinWebsocketMsg::PongMsg(_) = msg {
        } else {
            log::info!("Irrelevant Messages");
            log::info!("{msg:#?}")
        }
    }
}

/// Task to sync local orderbook from API.
/// Subscribes OrderbookEvent.
/// Publishes OrderbookEvent after sync.
pub async fn task_sync_orderbook(
    mut receiver: Receiver<OrderbookEvent>,
    sender: Sender<OrderbookEvent>,
    local_full_orderbook: Arc<Mutex<FullOrderbook>>,
    counter: Arc<Mutex<Counter>>,
) -> Result<(), kucoin_api::failure::Error> {
    loop {
        counter_helper::increment(counter.clone()).await;
        let event = receiver.recv().await?;
        let mut full_orderbook = local_full_orderbook.lock().await;
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
