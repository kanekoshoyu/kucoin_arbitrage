use crate::event::orderchange::OrderChangeEvent;
use kucoin_api::client::Kucoin;
use kucoin_api::futures::TryStreamExt;
use kucoin_api::model::websocket::{KucoinWebsocketMsg, WSTopic, WSType};
use tokio::sync::broadcast::Sender;

/// Task to publish order change events.
/// Subscribe Kucoi Websocket API, then publish OrderChangeEvent directly after conversion.
pub async fn task_pub_orderchange_event(
    api: Kucoin,
    sender: Sender<OrderChangeEvent>,
) -> Result<(), failure::Error> {
    let url_private = api.get_socket_endpoint(WSType::Private).await?;
    let mut ws = api.websocket();
    let topics = vec![WSTopic::TradeOrders];
    ws.subscribe(url_private.clone(), topics).await?;
    loop {
        // Awaits subscription message
        let ws_msg = ws.try_next().await?;
        let ws_msg = ws_msg.unwrap();
        match ws_msg {
            KucoinWebsocketMsg::TradeReceivedMsg(msg) => {
                log::info!("TradeReceivedMsg[{:#?}]", msg);
            }
            KucoinWebsocketMsg::TradeOpenMsg(msg) => {
                log::info!("TradeOpenMsg[{:#?}]", msg);
            }
            KucoinWebsocketMsg::TradeMatchMsg(msg) => {
                log::info!("TradeMatchMsg[{:#?}]", msg);
            }
            KucoinWebsocketMsg::TradeFilledMsg(msg) => {
                log::info!("TradeCanceledMsg[{:?}][{:#?}]", msg.topic, msg.data);
                let event = OrderChangeEvent::OrderCanceled((0, format!("{:?}", &msg.data)));
                sender.send(event).expect("Publishing OrderCanceled failed");
            }
            KucoinWebsocketMsg::BalancesMsg(msg) => {
                let delta = msg.data.available_change;
                let currency = msg.data.currency;
                log::info!("BalancesMsg: {currency:?}: {delta:?}");
            }
            KucoinWebsocketMsg::WelcomeMsg(_) => {}
            KucoinWebsocketMsg::PingMsg(_) => {}
            KucoinWebsocketMsg::PongMsg(_) => {}
            msg => {
                log::info!("Unregistered message in private channel [{msg:#?}]");
            }
        }
    }
}
