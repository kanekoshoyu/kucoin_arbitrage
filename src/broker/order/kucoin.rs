use crate::event::order::OrderEvent;
use crate::model::order::Order;
use kucoin_api::client::Kucoin;
use tokio::sync::broadcast;
use uuid::Uuid;
/// Converts received OrderEvent into REST API call
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
                // TODO study the API below
                // kucoin.cancel_all_orders(symbol, trade_type)
                unimplemented!();
            }
            OrderEvent::PlaceLimitOrder(order) => {
                // get the broadcast duration
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
                    "200000" => {
                        let uuid = Uuid::parse_str(&order.id)?;
                        log::info!("Limit order placement successful [{}]", uuid.as_u128());
                    }
                    "200004" => {
                        log::error!("Insufficient fund, check order placement status {order:?}");
                    }
                    "400100" => {
                        log::error!("Invalid order size increment {order:?}");
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
