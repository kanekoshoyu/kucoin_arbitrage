use eyre::Result;
use kucoin_arbitrage::system_event::task_signal_handle;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
// TODO suggest a data structure which we can use to store all the paths and search those paths efficiently
// TODO use interned string to store the asset symbol as u64
// TODO directly write a function from the actual data to verify them.

#[tokio::main]
async fn main() -> Result<()> {
    println!("exit upon terminating signal");
    tokio::select! {
        _ = task_signal_handle() => eyre::bail!("end"),
        _ = program() => Ok(()),
    }
}

////////////////////////////// struct

#[derive(Clone, Copy, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct Pair {
    base: u64,
    quote: u64,
}
impl Debug for Pair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.base, self.quote)
    }
}

#[derive(Clone, Copy, Hash, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum Action {
    Buy,
    Sell,
}

#[derive(Clone, Copy, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct TradeAction {
    pair: Pair,
    action: Action,
}
impl Debug for TradeAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}[{:?}]", self.action, self.pair)
    }
}

#[derive(Clone, Hash, PartialEq, PartialOrd, Eq, Ord, Default)]
pub struct TradeCycle {
    actions: Vec<TradeAction>,
}
impl From<Vec<TradeAction>> for TradeCycle {
    fn from(actions: Vec<TradeAction>) -> Self {
        TradeCycle { actions }
    }
}
impl TradeCycle {
    pub fn new() -> Self {
        TradeCycle::default()
    }
    pub fn push(&mut self, trade_action: TradeAction) {
        self.actions.push(trade_action)
    }
    pub fn pop(&mut self) -> Option<TradeAction> {
        self.actions.pop()
    }
}
impl Debug for TradeCycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cycle{:?}", self.actions)
    }
}

// all cycles can be used to look up for trade cycles with the asset ID
pub type AllCycles = HashMap<u64, Vec<TradeCycle>>;

////////////////////////////// fn

/// update paths and
fn dfs(
    current: u64,
    start: u64,
    graph: &HashMap<u64, Vec<Pair>>,
    cycle: &mut TradeCycle,
    visited: &mut HashSet<u64>,
    found_cycles: &mut Vec<TradeCycle>,
    must_start_with_buy: bool,
) {
    visited.insert(current);
    if let Some(pairs) = graph.get(&current) {
        for pair in pairs {
            let (next_node, action) = if current == pair.base {
                (pair.quote, Action::Sell)
            } else {
                (pair.base, Action::Buy)
            };

            // Enforce "Buy before Sell" rule: If path is empty, start only with Buy. Otherwise, proceed as per the action.
            if !must_start_with_buy || action == Action::Buy {
                if next_node == start
                    && cycle
                        .actions
                        .iter()
                        .any(|trade| trade.action == Action::Buy)
                {
                    let mut cycle = cycle.clone();
                    cycle.push(TradeAction {
                        pair: pair.clone(),
                        action,
                    });
                    found_cycles.push(cycle);
                } else if !visited.contains(&next_node) {
                    cycle.push(TradeAction {
                        pair: pair.clone(),
                        action,
                    });
                    dfs(next_node, start, graph, cycle, visited, found_cycles, false); // After the first trade, no need to enforce Buy as start.
                    cycle.pop();
                }
            }
        }
    }
    visited.remove(&current);
}

/// generate all the cyclic paths from the Graph
fn find_trading_paths(graph: &HashMap<u64, Vec<Pair>>, start: u64) -> Vec<TradeCycle> {
    let mut found_cycles = Vec::new();
    let mut visited = HashSet::new();
    let mut path = TradeCycle::new();
    // Start DFS with the flag to ensure the first trade is a Buy.
    dfs(
        start,
        start,
        graph,
        &mut path,
        &mut visited,
        &mut found_cycles,
        true,
    );
    found_cycles
}

async fn program() {
    let pairs = vec![
        Pair { base: 1, quote: 2 },
        Pair { base: 2, quote: 3 },
        Pair { base: 3, quote: 1 },
        Pair { base: 2, quote: 4 },
        Pair { base: 4, quote: 1 },
    ];

    // Constructing the graph from Pair structs
    let mut graph: HashMap<u64, Vec<Pair>> = HashMap::new();
    for pair in pairs {
        graph
            .entry(pair.base)
            .or_insert_with(Vec::new)
            .push(pair.clone());
        graph.entry(pair.quote).or_insert_with(Vec::new).push(pair);
    }

    let start_node = 1u64;
    let found_cycles = find_trading_paths(&graph, start_node);
    for (index, path) in found_cycles.iter().enumerate() {
        println!("Path {}: {:?}", index + 1, path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dfs() {
        // Setup a simple graph that represents the trading pairs.
        let pairs = vec![
            Pair { base: 1, quote: 2 },
            Pair { base: 2, quote: 3 },
            Pair { base: 3, quote: 1 },
        ];

        let mut graph: HashMap<u64, Vec<Pair>> = HashMap::new();
        for pair in pairs {
            graph
                .entry(pair.base)
                .or_insert_with(Vec::new)
                .push(pair.clone());
            graph.entry(pair.quote).or_insert_with(Vec::new).push(pair);
        }

        let start_node = 1u64;
        let trading_paths = find_trading_paths(&graph, start_node);

        // Define the expected paths using the Trade struct.
        // Note: The expected paths should match the actual trading paths you expect based on your graph setup.
        let expected_paths = vec![
            TradeCycle::from(vec![
                TradeAction {
                    pair: Pair { base: 3, quote: 1 },
                    action: Action::Buy,
                },
                TradeAction {
                    pair: Pair { base: 2, quote: 3 },
                    action: Action::Buy,
                },
                TradeAction {
                    pair: Pair { base: 1, quote: 2 },
                    action: Action::Buy,
                },
            ]),
            TradeCycle::from(vec![
                TradeAction {
                    pair: Pair { base: 3, quote: 1 },
                    action: Action::Buy,
                },
                TradeAction {
                    pair: Pair { base: 3, quote: 1 },
                    action: Action::Sell,
                },
            ]),
        ];

        // Check if the trading paths found match the expected paths.
        assert_eq!(trading_paths, expected_paths);
    }
}
