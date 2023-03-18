use crate::event::orderbook::OrderbookEvent;
use crate::translator::translator::OrderBookChangeTranslator;
use kucoin_rs::futures::TryStreamExt;
use kucoin_rs::kucoin::{model::websocket::KucoinWebsocketMsg, websocket::KucoinWebsocket};
use kucoin_rs::tokio::sync::broadcast::Sender;
use log::info;
use std::sync::Arc;
//TODO implement the internal trade order task in kucoin

pub async fn broadcast_websocket_l2(
    mut ws: KucoinWebsocket,
    sender: Arc<Sender<OrderbookEvent>>,
) -> Result<(), kucoin_rs::failure::Error> {
    let serial = 0;
    while let Some(msg) = ws.try_next().await? {
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
            info!("Connection setup")
        } else if let KucoinWebsocketMsg::PongMsg(_msg) = msg {
            info!("Connection maintained")
        } else {
            info!("Irrelevant Messages");
            info!("{msg:#?}")
        }
    }
    Ok(())
}
