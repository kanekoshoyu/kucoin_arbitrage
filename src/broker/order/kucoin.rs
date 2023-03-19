use crate::event::order::OrderEvent;
use crate::model::order::{LimitOrder, Order};
use kucoin_rs::tokio::sync::broadcast;
// do a broker that sends sends the received message to the API

pub async fn api_order(
    receiver: &mut broadcast::Receiver<OrderEvent>,
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
            }
        };
    }
    Ok(())
}
