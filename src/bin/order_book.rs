extern crate kucoin_rs;

use kucoin_rs::failure;
use kucoin_rs::futures::TryStreamExt;
use kucoin_rs::kucoin::{
    client::{Credentials, Kucoin, KucoinEnv},
    model::websocket::{KucoinWebsocketMsg, WSTopic, WSType},
    websocket::KucoinWebsocket,
};
use kucoin_rs::tokio::{
    self,
    time::{sleep, Duration},
};

use kucoin_arbitrage::mirror::{Map, MIRROR};
use kucoin_arbitrage::shared::*;
use lazy_static::lazy_static;
use log::*;
use std::sync::{Arc, Mutex};

// gets the jobs done
// Arc has implicit 'static bound, so it cannot contain reference to local variable.
lazy_static! {
    static ref CONFIG: Arc<Mutex<Config>> = Arc::new(Mutex::new(load_ini()));
    static ref PERFORMANCE: Arc<Mutex<Performance>> =
        Arc::new(Mutex::new(Performance { data_count: 0 }));
}

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // provide logging format
    kucoin_arbitrage::shared::log_init();
    info!("Hello world");

    let c = CONFIG.clone();
    let mg = c.lock().unwrap();
    let credentials = Credentials::new((*mg).api_key, (*mg).secret_key, (*mg).passphrase);
    drop(mg);

    info!("{credentials:#?}");
    let api = Kucoin::new(KucoinEnv::Live, Some(credentials))?;
    let url = api.get_socket_endpoint(WSType::Public).await?;
    let mut ws = api.websocket();

    // TODO: link the list_ticker to here and subscribe for all the tickers with BTC/USDT (Triangle)
    let subs = vec![WSTopic::OrderBook(vec!["ETH-BTC".to_string()])];
    ws.subscribe(url, subs).await?;

    info!("Async polling");
    // TODO: arbitrage performance analysis, such as arbitrage chance per minute

    let perf = PERFORMANCE.clone();
    let mirr = MIRROR.clone();
    tokio::spawn(async move { sync_tickers_rt(ws, perf, mirr).await });

    let monitor_delay = {
        let c = CONFIG.clone();
        let mg = c.lock().unwrap();
        let interval_sec: u64 = (*mg).monitor_interval_sec;
        drop(mg);
        Duration::from_secs(interval_sec)
    };
    // main loop is for monitoring the system performance
    loop {
        sleep(monitor_delay).await;
        report_status(PERFORMANCE.clone(), CONFIG.clone()).expect("report status error");
    }
}

// Though PERFORMANCE and CONFIG are globally accessible at the moment, we need to clone it annyways. We can just clone in the main function
fn report_status(
    perf: Arc<Mutex<Performance>>,
    conf: Arc<Mutex<Config>>,
) -> Result<(), failure::Error> {
    info!("reporting");
    let p = perf.lock().unwrap();
    let c = conf.lock().unwrap();
    let data_rate = (*p).data_count / (*c).monitor_interval_sec;
    drop(p);
    drop(c);

    info!("Data rate: {data_rate:?} points/sec");
    // clear the data
    {
        let mut p = perf.lock().unwrap();
        (*p).data_count = 0;
    }

    Ok(())
}

use kucoin_arbitrage::strings::topic_to_symbol;

async fn sync_tickers_rt(
    mut ws: KucoinWebsocket,
    perf: Arc<Mutex<Performance>>,
    mirror: Arc<Mutex<Map>>,
) -> Result<(), failure::Error> {
    while let Some(msg) = ws.try_next().await? {
        // add matches for multi-subscribed sockets handling
        match msg {
            KucoinWebsocketMsg::OrderBookMsg(msg) => {
                increment_data_counter(perf.to_owned());
                order_message_received(msg, mirror.to_owned());
                // info!("{:#?}", msg);
            }
            KucoinWebsocketMsg::PongMsg(_msg) => {}
            KucoinWebsocketMsg::WelcomeMsg(_msg) => {}
            _ => {
                panic!("unexpected msgs received: {msg:?}")
            }
        }
    }
    Ok(())
}

use kucoin_rs::kucoin::model::websocket::{Level2, WSResp};

fn increment_data_counter(perf: Arc<Mutex<Performance>>) {
    {
        let mut p = perf.lock().unwrap();
        (*p).data_count += 1;
    }
}
fn order_message_received(msg: WSResp<Level2>, mirror: Arc<Mutex<Map>>) {
    if msg.subject.ne("trade.l2update") {
        error!("unrecognised subject: {:?}", msg.subject);
        return;
    }
    // get the ticker name
    let ticker_name = topic_to_symbol(msg.topic).expect("wrong ticker format");
    // info!("Ticker received: {ticker_name}");
    let data = msg.data;
    // info!("{:#?}", data);
    let asks = data.changes.asks;
    let bids = data.changes.bids;
    for ask in asks.into_iter() {
        if ask.len().ne(&3) {
            panic!("wrong format");
        }
    }
    for bids in bids.into_iter() {
        if bids.len().ne(&3) {
            panic!("wrong format");
        }
    }
    // check if the ticker already exists in the map
    // let x = ticker_name.clone();
    // {
    //     let mut m = mirror.lock().unwrap();
    //     let tickers: &mut Map = &mut (*m);
    //     if let Some(data) = tickers.get_mut(&x) {
    //         // unimplemented!("found");
    //         data.symbol = msg.data;
    //     } else {
    //         tickers.insert(x, TickerInfo::new(msg.data));
    //     }
    // }
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_ticker_read() {
        let topic = "/market/ticker:ETH-BTC";
        let wanted = "ETH-BTC";
        let n = topic.find(":");
        if n.is_none() {
            panic!(": not found");
        }
        let n = n.unwrap() + 1; //add 1 after ":"
        let slice = &topic[n..];
        assert_eq!(wanted, slice);
    }

    #[test]
    fn test_get_ticker_string() {
        let topic = String::from("/market/ticker:ETH-BTC");
        let wanted = "ETH-BTC";
        let slice = crate::topic_to_symbol(topic).unwrap();
        assert_eq!(wanted, slice);
    }
}
