use crate::event::orderbook::OrderbookEvent;
use crate::model::orderbook::FullOrderbook;
use crate::model::symbol::SymbolInfo;
use crate::translator::traits::{OrderBookChangeTranslator, OrderBookTranslator};
use kucoin_api::client::Kucoin;
use kucoin_api::futures::TryStreamExt;
use kucoin_api::model::market::{OrderBook, OrderBookType};
use kucoin_api::model::websocket::{KucoinWebsocketMsg, WSTopic, WSType};
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use tokio::time::Duration;

/// Subscribe Websocket API, then publish internal OrderbookEvent
pub async fn task_pub_orderbook_event(
    api: Kucoin,
    topics: Vec<WSTopic>,
    sender: Sender<OrderbookEvent>,
) -> Result<(), failure::Error> {
    let serial = 0;
    let url_public = api.get_socket_endpoint(WSType::Public).await?;
    let mut ws = api.websocket();
    ws.subscribe(url_public.clone(), topics).await?;

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

/// Obtain current orderbook of a list of symbol from Kucoin REST API
pub async fn task_get_orderbook(api: Kucoin, symbol: &str) -> Result<OrderBook, failure::Error> {
    let mut try_counter = 0;
    loop {
        try_counter += 1;
        // OrderBookType::Full requires valid API Key
        let res = api.get_orderbook(symbol, OrderBookType::L20).await;
        if res.is_err() {
            log::warn!("orderbook[{symbol}] did not respond ({try_counter:?} tries)");
            continue;
        }
        let res = res.unwrap();
        match res.code.as_str() {
            "429000" => {
                log::warn!("[{symbol:?}] request overloaded ({try_counter:?} tries)")
            }
            "200000" => {
                if res.data.is_none() {
                    log::warn!("orderbook[{symbol}] received none ({try_counter:?} tries)");
                    continue;
                }
                log::info!("obtained [{symbol}]");
                return Ok(res.data.unwrap());
            }
            "400003" => return Err(failure::err_msg("API key needed not but provided")),
            code => return Err(failure::err_msg(format!("unrecognised code [{code:?}]"))),
        }
    }
}

/// Obtain all the inital orderbook using Kucoin REST API
pub async fn task_get_initial_orderbooks(
    api: Kucoin,
    symbol_infos: Vec<SymbolInfo>,
    full_orderbook: Arc<Mutex<FullOrderbook>>,
) -> Result<(), failure::Error> {
    // replace spawn with or a taskpool
    let mut taskpool_aggregate = JoinSet::new();
    // collect all initial orderbook states with REST
    let symbols: Vec<String> = symbol_infos.into_iter().map(|info| info.symbol).collect();
    log::info!("Total symbols: {:?}", symbols.len());
    for symbol in symbols {
        let api = api.clone();
        let full_orderbook_arc = full_orderbook.clone();
        // further improve performance and the server overload issue with mixed use of select and jjoin
        taskpool_aggregate.spawn(async move {
            let data = task_get_orderbook(api, &symbol).await.unwrap();
            let mut x = full_orderbook_arc.lock().await;
            x.insert(symbol.to_string(), data.to_internal());
            symbol
        });
        // prevent server overloading
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    while let Some(res) = taskpool_aggregate.join_next().await {
        if let Err(e) = res {
            return Err(failure::Error::from(e));
        }
        log::info!("Initialized orderbook for [{:?}]", res.unwrap());
    }
    Ok(())
}
