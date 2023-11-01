use crate::event::orderchange::OrderChangeEvent;
use crate::monitor;
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
        let msg = ws.try_next().await?;
        let msg = msg.unwrap();
        log::info!("message: {msg:?}");

        if let KucoinWebsocketMsg::TradeReceivedMsg(msg) = msg {
            // TradeReceived is only available to V2.
            // TODO sometimes V2 misses publishing TradeReceivedMsg, arguably due to initialization process issue.
            // Currently using a more stable TradeOpenMsg, although TradeReceived is always ahead of TradeOpen
            log::info!("TradeReceivedMsg: {:?}\n{:#?}", msg.topic, msg.data);
        } else if let KucoinWebsocketMsg::TradeOpenMsg(msg) = msg {
            let time = monitor::timer::stop("order_placement_network".to_string())
                .await
                .unwrap();
            log::info!("order_placement_network: {time:?}");

            log::info!("TradeOpenMsg: {:?}\n{:#?}", msg.topic, msg.data);
            // TODO optimize below to something more insightful
            let event: OrderChangeEvent =
                OrderChangeEvent::OrderOpen((0, format!("{:?}", msg.data.clone())));
            sender.send(event).expect("Publishing OrderOpen failed");
        } else if let KucoinWebsocketMsg::TradeMatchMsg(msg) = msg {
            log::info!("TradeMatchMsg: {:?}\n{:#?}", msg.topic, msg.data);
        } else if let KucoinWebsocketMsg::TradeFilledMsg(msg) = msg {
            log::info!("TradeFilledMsg: {:?}\n{:#?}", msg.topic, msg.data);
            let event: OrderChangeEvent =
                OrderChangeEvent::OrderFilled((0, format!("{:?}", msg.data.clone())));
            sender.send(event).expect("Publishing OrderFilled failed");
        } else if let KucoinWebsocketMsg::TradeCanceledMsg(msg) = msg {
            log::info!("TradeCanceledMsg: {:?}\n{:#?}", msg.topic, msg.data);
            let event: OrderChangeEvent =
                OrderChangeEvent::OrderCanceled((0, format!("{:?}", msg.data.clone())));
            sender.send(event).expect("Publishing OrderCanceled failed");
        } else if let KucoinWebsocketMsg::BalancesMsg(msg) = msg {
            let delta = msg.data.available_change;
            let currency = msg.data.currency;
            log::info!("BalancesMsg: {currency:?}: {delta:?}");
        } else if let KucoinWebsocketMsg::WelcomeMsg(_) = msg {
        } else if let KucoinWebsocketMsg::PongMsg(_) = msg {
        } else {
            log::info!("Irrelevant message in private channel: {:#?}", msg);
        }
    }
}