use std::sync::Arc;

use crate::model::counter::Counter;
use crate::model::order::Order;
use crate::{event::order::OrderEvent, global::counter_helper};
use kucoin_api::client::Kucoin;
use tokio::sync::{broadcast, Mutex};

/// Converts received OrderEvent into API call
pub async fn task_place_order(
    mut receiver: broadcast::Receiver<OrderEvent>,
    kucoin: Kucoin,
    counter: Arc<Mutex<Counter>>,
) -> Result<(), kucoin_api::failure::Error> {
    loop {
        counter_helper::increment(counter.clone()).await;

        let event = receiver.recv().await?;
        // println!("Received event: {event:?}");
        match event {
            OrderEvent::GetAllOrders => {
                // unimplemented!("mossing source of order_id");
                let status = kucoin.get_recent_orders().await;
                if let Err(e) = status {
                    log::error!("There was an error with kucoin API cancelling all order ({e})");
                } else {
                    let status_api_data = status.unwrap();
                    let data = status_api_data.data.unwrap();
                    for datum in data {
                        log::info!("get_recent_orders obtained {datum:#?}")
                    }
                }
            }
            OrderEvent::CancelOrder(order) => {
                if let Err(e) = kucoin.cancel_order(order.id().to_string().as_str()).await {
                    log::error!("There was an error with kucoin API cancelling single order ({e})");
                }

                unimplemented!()
            }
            OrderEvent::CancelAllOrders => {
                unimplemented!("mossing source of symbol and trade_type");
                // if let Err(e) = kucoin.cancel_all_orders(symbol, trade_type).await {
                //     log::error!("There was an error with kucoin API cancelling all order ({e})");
                // }
            }
            OrderEvent::PostOrder(order) => {
                log::info!("order placement\n{order:?}");
                if let Err(e) = kucoin
                    .post_limit_order(
                        order.id().to_string().as_str(),
                        order.symbol().as_str(),
                        order.side().to_string().as_str(),
                        order.price().as_str(),
                        order.amount().as_str(),
                        None,
                    )
                    .await
                {
                    log::error!("There was an error with kucoin API placing order ({e})");
                }
            }
        };
    }
}
