/// Order change received from exchange
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderChangeEvent {
    OrderOpen((u128, String)),
    OrderFilled((u128, String)),
    OrderCanceled((u128, String)),
}
