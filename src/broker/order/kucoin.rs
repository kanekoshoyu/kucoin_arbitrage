use crate::event::order::OrderEvent;
use crate::model::order::Order;
use eyre::Result;
use kucoin_api::client::Kucoin;
use tokio::sync::broadcast;
use uuid::Uuid;

/// Converts received OrderEvent into REST API call
pub async fn task_place_order(
    mut receiver: broadcast::Receiver<OrderEvent>,
    kucoin: Kucoin,
) -> Result<()> {
    loop {
        let event = receiver.recv().await?;
        // println!("Received event: {event:?}");
        match event {
            OrderEvent::GetAllOrders => {
                let status = kucoin
                    .get_recent_orders()
                    .await
                    .map_err(|e| eyre::eyre!(e))?;
                tracing::info!("{status:?}");
            }
            OrderEvent::CancelOrder(order) => {
                let status = kucoin
                    .cancel_order(order.id().as_ref())
                    .await
                    .map_err(|e| eyre::eyre!(e))?;
                tracing::info!("{status:?}");
            }
            OrderEvent::CancelAllOrders => {
                todo!("implement batch order cancellation with kucoin.cancel_all_orders(symbol, trade_type)");
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
                    .await
                    .map_err(|e| eyre::eyre!(e))?;
                match status.code.as_str() {
                    "200000" => {
                        let uuid = Uuid::parse_str(&order.id)?;
                        tracing::info!("Limit order placement successful [{}]", uuid.as_u128());
                    }
                    "200004" => {
                        tracing::error!(
                            "Insufficient fund, check order placement status {order:?}"
                        );
                    }
                    "400100" => {
                        tracing::error!("Invalid order size increment {order:?}");
                    }
                    code => eyre::bail!("unrecognised code [{code:?}]"),
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
