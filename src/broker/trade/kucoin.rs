use crate::event::trade::TradeEvent;
use crate::translator::traits::ToTradeInfo;
use kucoin_api::client::Kucoin;
use kucoin_api::futures::TryStreamExt;
use kucoin_api::model::websocket::{KucoinWebsocketMsg, WSTopic, WSType};
use tokio::sync::broadcast::Sender;

/// Task to publish order change events.
/// Subscribe Kucoi Websocket API, then publish tradeEvent directly after conversion.
pub async fn task_pub_trade_event(
    api: Kucoin,
    sender: Sender<TradeEvent>,
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
            KucoinWebsocketMsg::TradeReceivedMsg(_) => {
                unimplemented!("not sure how it works yet")
            }
            KucoinWebsocketMsg::TradeOpenMsg(msg) => {
                let tradeinfo = msg.data.to_internal()?;
                log::info!("TradeMatch [{}]", tradeinfo.order_id);
                sender.send(TradeEvent::TradeMatch(tradeinfo))?;
            }
            KucoinWebsocketMsg::TradeMatchMsg(msg) => {
                let tradeinfo = msg.data.to_internal()?;
                log::info!("TradeMatch [{}]", tradeinfo.order_id);
                sender.send(TradeEvent::TradeMatch(tradeinfo))?;
            }
            KucoinWebsocketMsg::TradeFilledMsg(msg) => {
                let tradeinfo = msg.data.to_internal()?;
                log::info!("TradeFilledMsg [{}]", tradeinfo.order_id);
                sender.send(TradeEvent::TradeFilled(tradeinfo))?;
            }
            KucoinWebsocketMsg::BalancesMsg(msg) => {
                let delta = msg.data.available_change;
                let currency = msg.data.currency;
                log::info!("BalancesMsg: {currency:?}: {delta:?}");
            }
            KucoinWebsocketMsg::WelcomeMsg(_) => {
                log::info!("Welcome to KuCoin private WS");
            }
            KucoinWebsocketMsg::PingMsg(_) => {}
            KucoinWebsocketMsg::PongMsg(_) => {}
            msg => {
                log::info!("Unregistered message in private channel [{msg:#?}]");
            }
        }
    }
}