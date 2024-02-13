use crate::event::orderbook::OrderbookEvent;
use crate::model::orderbook::FullOrderbook;
use eyre::Result;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Mutex;

/// Subscribe OrderbookEvent, then publish OrderbookEvent after syncing local orderbook
pub async fn task_sync_orderbook(
    mut receiver: Receiver<OrderbookEvent>,
    sender: Sender<OrderbookEvent>,
    local_full_orderbook: Arc<Mutex<FullOrderbook>>,
) -> Result<()> {
    loop {
        let event = receiver.recv().await?;
        let mut full_orderbook = local_full_orderbook.lock().await;
        match event {
            OrderbookEvent::OrderbookReceived((symbol, orderbook)) => {
                (*full_orderbook).insert(symbol.clone(), orderbook);
                tracing::info!("Initialised Orderbook for {symbol}")
            }
            OrderbookEvent::OrderbookChangeReceived((symbol, orderbook_change)) => {
                let orderbook = (*full_orderbook).get_mut(&symbol).ok_or(eyre::eyre!(
                    "received {symbol} but orderbook not initialised yet."
                ))?;
                // tracing::info!("insertion: {orderbook_change:#?}");
                match orderbook.merge(orderbook_change) {
                    Ok(Some(ob)) => {
                        sender.send(OrderbookEvent::OrderbookChangeReceived((symbol, ob)))?;
                    }
                    Err(e) => {
                        tracing::error!("Merge conflict: {e}")
                    }
                    _ => {} // no update in best price
                }
            }
        }
    }
}
