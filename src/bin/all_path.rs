use eyre::Result;
use kucoin_arbitrage::system_event::task_signal_handle;

#[tokio::main]
async fn main() -> Result<()> {
    println!("waiting for terminating signal");
    tokio::select! {
        _ = task_signal_handle() => println!("end"),
        _ = program() => println!("error"),
    };
    Ok(())
}
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

#[derive(Clone, Copy, Hash, PartialEq, PartialOrd, Eq, Ord)]
struct Pair {
    base: u64,
    quote: u64,
}
impl Debug for Pair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.base, self.quote)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
enum Action {
    Buy,
    Sell,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
struct Trade {
    pair: Pair,
    action: Action,
}
fn dfs(
    current: u64,
    start: u64,
    graph: &HashMap<u64, Vec<Pair>>,
    path: &mut Vec<Trade>,
    visited: &mut HashSet<u64>,
    all_paths: &mut Vec<Vec<Trade>>,
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
                if next_node == start && path.iter().any(|trade| trade.action == Action::Buy) {
                    let mut cycle = path.clone();
                    cycle.push(Trade {
                        pair: pair.clone(),
                        action,
                    });
                    all_paths.push(cycle);
                } else if !visited.contains(&next_node) {
                    path.push(Trade {
                        pair: pair.clone(),
                        action,
                    });
                    dfs(next_node, start, graph, path, visited, all_paths, false); // After the first trade, no need to enforce Buy as start.
                    path.pop();
                }
            }
        }
    }

    visited.remove(&current);
}

fn find_trading_paths(graph: &HashMap<u64, Vec<Pair>>, start: u64) -> Vec<Vec<Trade>> {
    let mut all_paths = Vec::new();
    let mut visited = HashSet::new();
    let mut path = Vec::new();

    // Start DFS with the flag to ensure the first trade is a Buy.
    dfs(
        start,
        start,
        graph,
        &mut path,
        &mut visited,
        &mut all_paths,
        true,
    );

    all_paths
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
    let trading_paths = find_trading_paths(&graph, start_node);
    for (index, path) in trading_paths.iter().enumerate() {
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
            vec![
                Trade {
                    pair: Pair { base: 3, quote: 1 },
                    action: Action::Buy,
                },
                Trade {
                    pair: Pair { base: 2, quote: 3 },
                    action: Action::Buy,
                },
                Trade {
                    pair: Pair { base: 1, quote: 2 },
                    action: Action::Buy,
                },
            ],
            vec![
                Trade {
                    pair: Pair { base: 3, quote: 1 },
                    action: Action::Buy,
                },
                Trade {
                    pair: Pair { base: 3, quote: 1 },
                    action: Action::Sell,
                },
            ],
        ];

        // Check if the trading paths found match the expected paths.
        assert_eq!(trading_paths, expected_paths);
    }
}
