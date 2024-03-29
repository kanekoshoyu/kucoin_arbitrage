use eyre::Result;
/// Test WebSocket Message Rate
/// Subscribe to messages, run for 10 seconds, get the rate respectively
use kucoin_api::futures::TryStreamExt;
use kucoin_api::{
    client::{Kucoin, KucoinEnv},
    model::websocket::{KucoinWebsocketMsg, WSTopic, WSType},
    websocket::KucoinWebsocket,
};
use kucoin_arbitrage::monitor::counter;
use std::sync::Arc;
use tokio::sync::Mutex;

/// main function
#[tokio::main]
async fn main() -> Result<()> {
    // provide logging format
    // kucoin_arbitrage::logger::log_init()?;
    let counter = Arc::new(Mutex::new(counter::Counter::new("api_input")));
    tracing::info!("Testing Kucoin WS Message Rate");

    // config
    let config = kucoin_arbitrage::config::from_file("config.toml")?;
    let monitor_interval: u32 = config.behaviour.monitor_interval_sec;

    // Initialize the Kucoin API struct
    let api = Kucoin::new(KucoinEnv::Live, Some(config.kucoin_credentials()))
        .map_err(|e| eyre::eyre!(e))?;
    let url = api
        .get_socket_endpoint(WSType::Public)
        .await
        .map_err(|e| eyre::eyre!(e))?;
    let mut ws = api.websocket();
    let symbols = [
        "ETH-BTC".to_string(),
        "BTC-USDT".to_string(),
        "ETH-USDT".to_string(),
    ];
    let subs = vec![WSTopic::OrderBook(symbols.to_vec())];
    ws.subscribe(url, subs).await.map_err(|e| eyre::eyre!(e))?;

    tracing::info!("Async polling");
    tokio::spawn(sync_tickers(ws, counter.clone()));
    let _res = tokio::join!(kucoin_arbitrage::monitor::task::task_log_mps(
        vec![counter.clone()],
        monitor_interval as u64
    ));
    panic!("Program should not arrive here")
}

async fn sync_tickers(
    mut ws: KucoinWebsocket,
    counter: Arc<Mutex<counter::Counter>>,
) -> Result<()> {
    while let Some(msg) = ws.try_next().await.map_err(|e| eyre::eyre!(e))? {
        match msg {
            KucoinWebsocketMsg::OrderBookMsg(_msg) => {
                // TODO make counter more generic
                counter::reset(counter.clone()).await;
            }
            KucoinWebsocketMsg::PongMsg(_) => continue,
            KucoinWebsocketMsg::WelcomeMsg(_) => continue,
            _ => {
                panic!("unexpected msgs received: {msg:?}")
            }
        }
    }
    Ok(())
}
