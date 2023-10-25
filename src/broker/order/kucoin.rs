use crate::event::order::OrderEvent;
use crate::model::order::Order;
use crate::monitor;
use kucoin_api::client::Kucoin;
use tokio::sync::broadcast;

/// Converts received OrderEvent into API call
pub async fn task_place_order(
    mut receiver: broadcast::Receiver<OrderEvent>,
    kucoin: Kucoin,
) -> Result<(), failure::Error> {
    loop {
        let event = receiver.recv().await?;
        // println!("Received event: {event:?}");
        match event {
            OrderEvent::GetAllOrders => {
                let status = kucoin.get_recent_orders().await?;
                log::info!("{status:?}");
            }
            OrderEvent::CancelOrder(order) => {
                let status = kucoin.cancel_order(order.id().as_ref()).await?;
                log::info!("{status:?}");
            }
            OrderEvent::CancelAllOrders => {
                unimplemented!();
            }
            OrderEvent::PlaceOrder(order) => {
                // get the broadcast duration
                let time = monitor::timer::stop("order_placement_broadcast".to_string()).await?;
                log::info!("order_placement_broadcast: {:?}", time);
                let status = kucoin
                    .post_limit_order(
                        order.id().as_ref(),
                        order.symbol().as_ref(),
                        order.side().as_ref(),
                        order.price().as_ref(),
                        order.amount().as_ref(),
                        None,
                    )
                    .await?;
                match status.code.as_str() {
                    "200004" => {
                        return Err(failure::err_msg("Insufficient fund"));
                    }
                    code => return Err(failure::err_msg(format!("unrecognised code [{code:?}]"))),
                };
            }
            OrderEvent::PlaceBorrowOrder(_order) => {
                // TODO learn more about the function below
                // kucoin.post_borrow_order(currency, trade_type, size, max_rate, term)
                unimplemented!();
            }
        };
    }
}
