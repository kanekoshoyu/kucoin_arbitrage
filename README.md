# KuCoin Triangular Arbitrage Bot in Rust
Rust project to implement zero-risk crypto trinagular arbitrage, explore technical feasiblity of generating passsive income (i.e. sleep to earn!).

## Previous mistakes in Python
2 years ago I have made a python script that runs the triangular arbitrage in KuCoin, but it had several technical issues and endeed up nopt following up.  
https://github.com/kanekoshoyu/Kucoin-Triangular-Arbitrage  
- Implementation with Python and REST polling was way too slow to obtain and execute the arbitratge.
- Generally the Python REST API caused quite high rates of communication error, which took extra time for resorting.
- It didn't count the actual size of the arbitrage order, which meant that the script kept buying some shitcoin and could not sell properly.

## How to run
cargo run --bin XXX

## Major Improvements
- Use precompiled Rust and websocket which subscribes to the latest market data in real time.
- Only take a taker position based on the latest best-bid/ask price/size.
- Implement both network performance as well as arbitrage performance as a task.
- Modularize the subscription sync as well as selection of the coin triangles/algorithm to make it more genertic. 

## To-dos
- Wrap the runtime as a local service which can talk with other components in the nerwork
- Add a Qt/WebAsssembly GUI which visualises the arbitrage situation.