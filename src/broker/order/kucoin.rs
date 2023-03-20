use crate::event::order::OrderEvent;
use crate::model::order::Order;
use kucoin_rs::kucoin::client::Kucoin;
use kucoin_rs::tokio::sync::broadcast;

pub async fn api_order(
    receiver: &mut broadcast::Receiver<OrderEvent>,
    kucoin: Kucoin,
) -> Result<(), kucoin_rs::failure::Error> {
    // Converts reveived Message into API call

    while let Ok(event) = receiver.recv().await {
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
    Ok(())
}
