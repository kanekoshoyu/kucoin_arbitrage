use crate::event::trade::TradeEvent;
use crate::translator::traits::ToTradeInfo;
use eyre::Result;
use kucoin_api::client::Kucoin;
use kucoin_api::futures::TryStreamExt;
use kucoin_api::model::websocket::{KucoinWebsocketMsg, WSTopic, WSType};
use tokio::sync::broadcast::Sender;

/// Task to publish order change events.
/// Subscribe Kucoim Websocket API, then publish tradeEvent directly after conversion.
pub async fn task_pub_trade_event(api: Kucoin, sender: Sender<TradeEvent>) -> Result<()> {
    let url_private = api
        .get_socket_endpoint(WSType::Private)
        .await
        .map_err(|e| eyre::eyre!("failed connecting private endpoint, check API key ({e})"))?;
    let mut ws = api.websocket();
    // TODO test TradeOrdersV2
    let topics = vec![WSTopic::TradeOrders];
    ws.subscribe(url_private.clone(), topics)
        .await
        .expect("failed subscribing trade event");
    loop {
        // Awaits subscription message
        let ws_msg = ws.try_next().await.map_err(|e| eyre::eyre!(e))?;
        let ws_msg = ws_msg.unwrap();
        match ws_msg {
            KucoinWebsocketMsg::TradeReceivedMsg(msg) => {
                let tradeinfo = msg.data.to_internal()?;
                tracing::info!(
                    "TradeReceived[{}] (not so sure when it gets received)",
                    tradeinfo.order_id
                );
                // sender.send(TradeEvent::TradeReceived(tradeinfo))?;
            }
            KucoinWebsocketMsg::TradeOpenMsg(msg) => {
                let tradeinfo = msg.data.to_internal()?;
                tracing::info!("TradeOpen[{}]", tradeinfo.order_id);
                sender.send(TradeEvent::TradeOpen(tradeinfo))?;
            }
            KucoinWebsocketMsg::TradeMatchMsg(msg) => {
                let tradeinfo = msg.data.to_internal()?;
                tracing::info!("TradeMatch[{}]", tradeinfo.order_id);
                sender.send(TradeEvent::TradeMatch(tradeinfo))?;
            }
            KucoinWebsocketMsg::TradeFilledMsg(msg) => {
                let tradeinfo = msg.data.to_internal()?;
                tracing::info!("TradeFilledMsg[{}]", tradeinfo.order_id);
                sender.send(TradeEvent::TradeFilled(tradeinfo))?;
            }
            KucoinWebsocketMsg::WelcomeMsg(_) => {
                tracing::info!("Welcome to KuCoin private WS");
            }
            KucoinWebsocketMsg::PingMsg(_) => {}
            KucoinWebsocketMsg::PongMsg(_) => {}
            msg => {
                tracing::info!("Unregistered message in private channel [{msg:#?}]");
            }
        }
    }
}
