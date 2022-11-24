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

    let subs = vec![WSTopic::Ticker(vec![
        "ETH-BTC".to_string(),
        "BTC-USDT".to_string(),
        "ETH-USDT".to_string(),
    ])];
    ws.subscribe(url, subs).await?;

    info!("Async polling");
    let perf = PERFORMANCE.clone();
    tokio::spawn(async move { sync_tickers(ws, perf).await });

    let monitor_delay = {
        let c = CONFIG.clone();
        let mg = c.lock().unwrap();
        let interval_sec: u64 = (*mg).monitor_interval_sec;
        drop(mg);
        Duration::from_secs(interval_sec)
    };
    loop {
        sleep(monitor_delay).await;
        report_status(PERFORMANCE.clone(), CONFIG.clone()).expect("report status error");
    }
}

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

async fn sync_tickers(
    mut ws: KucoinWebsocket,
    perf: Arc<Mutex<Performance>>,
) -> Result<(), failure::Error> {
    while let Some(msg) = ws.try_next().await? {
        match msg {
            KucoinWebsocketMsg::TickerMsg(_msg) => {
                let mut p = perf.lock().unwrap();
                (*p).data_count += 1;
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
