use crate::event::orderchange::OrderChangeEvent;
use kucoin_api::futures::TryStreamExt;
use kucoin_api::{model::websocket::KucoinWebsocketMsg, websocket::KucoinWebsocket};
use tokio::sync::broadcast::Sender;

/// Task to publish order change events.
/// Subscribes Websocket API.
/// Publishes OrderChangeEvent directly after conversion.
pub async fn task_pub_orderchange_event(
    mut ws: KucoinWebsocket,
    sender: Sender<OrderChangeEvent>,
) -> Result<(), kucoin_api::failure::Error> {
    loop {
        // awaits subscription message
        let msg = ws.try_next().await?.unwrap();

        // matches message type
        if let KucoinWebsocketMsg::TradeReceivedMsg(msg) = msg {
            let trade_open = msg.data;
            // TODO optimize below to something more insightful
            let event = OrderChangeEvent::OrderReceived((0, format!("{:?}", trade_open.clone())));
            if sender.send(event).is_err() {
                log::error!("Order change event publish error, check receiver");
            }
        } else if let KucoinWebsocketMsg::WelcomeMsg(msg) = msg {
            log::info!("Welcome {:?}", msg);
        } else if let KucoinWebsocketMsg::Error(msg) = msg {
            log::error!("Error: {msg:?}");
            panic!("Error received from KuCoin private channel");
        } else {
            log::info!("Irrelevant Trade messages");
            log::info!("{msg:#?}")
        }
    }
}
