use crate::event::orderbook::OrderbookEvent;
use crate::model::orderbook::FullOrderbook;
use crate::monitor::counter;
use crate::monitor::counter::Counter;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Mutex;

/// Subscribe OrderbookEvent, then publish OrderbookEvent after syncing local orderbook
pub async fn task_sync_orderbook(
    mut receiver: Receiver<OrderbookEvent>,
    sender: Sender<OrderbookEvent>,
    local_full_orderbook: Arc<Mutex<FullOrderbook>>,
    counter: Arc<Mutex<Counter>>,
) -> Result<(), failure::Error> {
    loop {
        counter::increment(counter.clone()).await;
        let event = receiver.recv().await?;
        let mut full_orderbook = local_full_orderbook.lock().await;
        match event {
            OrderbookEvent::OrderbookReceived((symbol, orderbook)) => {
                (*full_orderbook).insert(symbol.clone(), orderbook);
                log::info!("Initialised Orderbook for {symbol}")
            }
            OrderbookEvent::OrderbookChangeReceived((symbol, orderbook_change)) => {
                let orderbook = (*full_orderbook).get_mut(&symbol);
                if orderbook.is_none() {
                    return Err(failure::err_msg(format!(
                        "received {symbol} but orderbook not initialised yet."
                    )));
                }
                // log::info!("insertion: {orderbook_change:#?}");
                match orderbook.unwrap().merge(orderbook_change) {
                    Ok(res) => {
                        if let Some(ob) = res {
                            // log::info!("update: {ob:#?}");
                            sender
                                .send(OrderbookEvent::OrderbookChangeReceived((symbol, ob)))
                                .unwrap();
                        }
                    }
                    Err(e) => {
                        log::error!("Merge conflict: {e}")
                    }
                }
            }
        }
    }
}
