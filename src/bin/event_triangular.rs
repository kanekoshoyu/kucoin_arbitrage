use kucoin_arbitrage::event::orderbook::OrderbookEvent;
use kucoin_arbitrage::globals::performance;
use kucoin_arbitrage::model::orderbook::FullOrderbook as InhouseFullOrderBook;
use kucoin_arbitrage::strategy::all_taker::task_triangular_arbitrage;
use kucoin_arbitrage::translator::translator::OrderBookChangeTranslator;
use kucoin_rs::futures::TryStreamExt;
use kucoin_rs::kucoin::{
    client::{Kucoin, KucoinEnv},
    model::websocket::{KucoinWebsocketMsg, WSTopic, WSType},
    websocket::KucoinWebsocket,
};
use kucoin_rs::tokio;
use log::{error, info};
use std::sync::Arc;
use tokio::sync::broadcast::{channel, Sender};

async fn broadcast_websocket_l2(
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

#[tokio::main]
async fn main() -> Result<(), kucoin_rs::failure::Error> {
    // provide logging format
    kucoin_arbitrage::logger::log_init();
    info!("Log setup");

    // credentials
    let credentials = kucoin_arbitrage::globals::config::credentials();
    let api = Kucoin::new(KucoinEnv::Live, Some(credentials))?;
    let url = api.get_socket_endpoint(WSType::Public).await?;
    info!("Credentials setup");

    // Initialize the websocket
    let mut ws = api.websocket();
    let subs = vec![
        WSTopic::OrderBook(vec![
            "ETH-BTC".to_string(),
            "BTC-USDT".to_string(),
            "ETH-USDT".to_string(),
        ]),
        // WSTopic::OrderBookChange(vec!["ETH-BTC".to_string(), "BTC-USDT".to_string()]),
    ];
    ws.subscribe(url, subs).await?;
    info!("Websocket subscription setup");

    // Create a broadcast channel.
    let (sender, receiver) = channel(256);
    let sender = Arc::new(sender);
    info!("Channel setup");

    tokio::spawn(async move { broadcast_websocket_l2(ws, sender).await });
    // broadcast_websocket_l2(ws, sender).await;

    // Spawn multiple tasks to receive messages.
    let mut receivers = Vec::new();
    let mut receiver = receiver.resubscribe();

    let receiver_handle = tokio::spawn(async move {
        let mut local_full_orderbook = InhouseFullOrderBook::new();
        loop {
            let event_status = receiver.recv().await;
            performance::increment();
            if event_status.is_err() {
                let e = event_status.unwrap_err();
                info!("Detected {e}, try again");
                continue;
            }
            let event = event_status.unwrap();
            // println!("Received event: {event:?}");
            if let OrderbookEvent::OrderbookReceived((symbol, orderbook_change)) = event {
                // merge the local orderbook with this one
                let status = local_full_orderbook.get_mut(&symbol);
                if status.is_none() {
                    local_full_orderbook.insert(symbol, orderbook_change);
                } else {
                    if let Err(()) = status.unwrap().merge(orderbook_change) {
                        error!("Merge conflict")
                    }
                }
            }
        }
    });
    receivers.push(receiver_handle);
    kucoin_arbitrage::tasks::background_routine().await
}
