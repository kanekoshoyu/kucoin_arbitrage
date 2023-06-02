/// Counter for system monitor
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Counter {
    pub name: &'static str,
    pub data_count: u64,
}
impl Counter {
    // Constructs a new instance of [`Counter`].
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            data_count: 0,
        }
    }
}
