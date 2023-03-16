use kucoin_arbitrage::event::api::ApiEvent;
use kucoin_arbitrage::translator::translator::OrderBookChangeTranslator;
use kucoin_rs::futures::TryStreamExt;
use kucoin_rs::kucoin::{
    client::{Kucoin, KucoinEnv},
    model::websocket::{KucoinWebsocketMsg, WSTopic, WSType},
    websocket::KucoinWebsocket,
};
use kucoin_rs::tokio;
use log::info;
use std::sync::Arc;
use tokio::sync::broadcast::{channel, Sender};

async fn broadcast_websocket_l2(
    mut ws: KucoinWebsocket,
    sender: Arc<Sender<ApiEvent>>,
) -> Result<(), kucoin_rs::failure::Error> {
    let serial = 0;
    loop{
        if let Some(msg) = ws.try_next().await.unwrap() {
            // add matches for multi-subscribed sockets handling
            if let KucoinWebsocketMsg::OrderBookMsg(msg) = msg {
                let l2 = msg.data;
                let (str, data) = l2.to_internal(serial);
                info!("L2 recceived {str:#?}");
                let event = ApiEvent::OrderbookReceived(data);
                sender.send(event).unwrap();
                info!("{l2:#?}")
            } else {
                info!("Irrelevant Messages");
                info!("{msg:#?}")
            }
        }
        info!("Exiting sync_tickers");
    }
    Ok(())

}

#[tokio::main]
async fn main() -> Result<(), kucoin_rs::failure::Error> {
    // provide logging format
    kucoin_arbitrage::logger::log_init();
    info!("Hello world");
    let credentials = kucoin_arbitrage::globals::config::credentials();
    // TODO fix API error here
    // Initialize the Kucoin API struct
    let api = Kucoin::new(KucoinEnv::Live, Some(credentials))?;
    // Generate the dynamic Public or Private websocket url and endpoint
    let url = api.get_socket_endpoint(WSType::Public).await?;
    // Initialize the websocket
    let mut ws = api.websocket();
    let subs = vec![WSTopic::Ticker(vec![
        "ETH-BTC".to_string(),
        "BTC-USDT".to_string(),
        "ETH-USDT".to_string(),
    ])];
    ws.subscribe(url, subs).await?;

    // Create a broadcast channel.
    let (sender, receiver) = channel(10);
    // Convert the sender into an Arc pointer to share across tasks.
    let sender = Arc::new(sender);
    tokio::task::spawn(async move { broadcast_websocket_l2(ws, sender).await });
    // Spawn multiple tasks to receive messages.
    let mut receivers = Vec::new();
    let mut receiver = receiver.resubscribe();

    let receiver_handle = tokio::task::spawn(async move {
        loop {
            let event = receiver.recv().await.unwrap();
            println!("Received event: {event:?}");
        }
    });
    receivers.push(receiver_handle);
    // }
    Ok(())
}
