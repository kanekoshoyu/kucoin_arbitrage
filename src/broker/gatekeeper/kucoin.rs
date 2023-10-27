use crate::event::chance::ChanceEvent;
use crate::event::order::OrderEvent;
use crate::event::trade::TradeEvent;
use crate::model::order::{LimitOrder, OrderType};
use tokio::sync::broadcast::{Receiver, Sender};
use uuid::Uuid;
// TODO implement when all_taker_btc_usdt is done
// TODO implement profit maximization

/// Broker that accepts chances, then outputs actual orders based on other limiting factors
/// Gate Keeper
/// - Amount of money left in the account
/// - transaction formatted to tradeable digits
/// - 45 orders per 3 seconds
/// - 200 active order at once
pub async fn task_gatekeep_chances(
    mut rx_chance: Receiver<ChanceEvent>,
    mut rx_trade: Receiver<TradeEvent>,
    tx_order: Sender<OrderEvent>,
) -> Result<(), failure::Error> {
    loop {
        let status = rx_chance.recv().await;
        if let Err(e) = status {
            log::error!("gatekeep chance parsing error {e:?}");
            return Ok(());
        }
        let event: ChanceEvent = status.unwrap();
        match event {
            ChanceEvent::AllTaker(chance) => {
                log::info!("All Taker Chance found!");
                log::info!("{chance:?}");
                // i is [0, 1, 2]
                for i in 0..3 {
                    let uuid = Uuid::new_v4();
                    let order: LimitOrder = LimitOrder {
                        id: uuid.to_string(),
                        order_type: OrderType::Limit,
                        side: chance.actions[i].action,
                        symbol: chance.actions[i].ticker.clone(),
                        amount: chance.actions[i].volume.to_string(),
                        price: chance.actions[i].price.to_string(),
                    };
                    tx_order.send(OrderEvent::PlaceLimitOrder(order))?;
                    let fill_target = chance.actions[i].price.0;
                    let mut fill_cumulative = 0.0;
                    while fill_cumulative < fill_target {
                        let trade_event = rx_trade.recv().await?;
                        match trade_event {
                            TradeEvent::TradeFilled(info) => {
                                if info.order_id.eq(&uuid.as_u128()) {
                                    // TODO use actual data to deduct the amount_untraded
                                    log::warn!("while we are currently assuming it is all filled at once, please implement accumulation");
                                    log::info!("{info:?}");
                                    fill_cumulative = fill_target;
                                }
                            }
                            other => {
                                // print for debugging purpose
                                if let TradeEvent::TradeMatch(info) = other {
                                    log::info!("Ignoring TradeMatch[{}]", info.order_id);
                                }
                            }
                        }
                    }
                }
                log::info!("cycle completed!")
            }
            ChanceEvent::MakerTakerTaker(_actions) => {}
        }
    }
}
