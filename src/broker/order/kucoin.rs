use crate::event::order::OrderEvent;
use crate::model::order::{LimitOrder, Order};
use kucoin_rs::kucoin::client::Kucoin;
use kucoin_rs::tokio::sync::broadcast;

// do a broker that sends sends the received message to the API

pub async fn api_order(
    receiver: &mut broadcast::Receiver<OrderEvent>,
    kucoin: Kucoin,
) -> Result<(), kucoin_rs::failure::Error> {
    // Converts reveived Message into API call

    while let Ok(event) = receiver.recv().await {
        // println!("Received event: {event:?}");
        match event {
            OrderEvent::GetAllOrders => {
                unimplemented!()
            }
            OrderEvent::CancelOrder(order) => {
                unimplemented!()
            }
            OrderEvent::CancelAllOrders => {
                unimplemented!()
            }
            OrderEvent::PostOrder(order) => {
                let id = order.id();
                // TODO use api here

                kucoin.post_limit_order(
                    order.id(),
                    order.symbol(),
                    order.side(),
                    order.price(),
                    size,
                    optionals = None,
                ).await
                // convert all the variables into &str
                // pub async fn post_limit_order(
                //     &self,
                //     client_oid: &str,
                //     symbol: &str,
                //     side: &str,
                //     price: &str,
                //     size: &str,
                //     optionals: Option<OrderOptionals<'_>>,
                // ) -> Result<APIDatum<OrderResp>, APIError> {
            }
        };
    }
    Ok(())
}
