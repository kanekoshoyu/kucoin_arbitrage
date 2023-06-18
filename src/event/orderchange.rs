/// Order change received from exchange
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderChangeEvent {
    OrderReceived((u128, String)),
    OrderOpen((u128, String)),
    OrderMatch((u128, String)),
    OrderFilled((u128, String)),
    OrderCanceled((u128, String)),
}
