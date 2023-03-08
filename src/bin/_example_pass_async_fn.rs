use kucoin_rs::tokio;
use std::future::Future;
use futures::future::join_all;
use std::pin::Pin;
use tokio::runtime::Runtime;

async fn foo(x: i32) -> i32 {
    x + 1
}

async fn bar(x: i32) -> i32 {
    x * 2
}

fn main() {
    let mut rt = Runtime::new().unwrap();

    let mut futures: Vec<Pin<Box<dyn Future<Output = i32>>>> = Vec::new();

    for i in 0..5 {
        let fut = Box::pin(foo(i));
        futures.push(fut);
    }

    for i in 0..5 {
        let fut = Box::pin(bar(i));
        futures.push(fut);
    }

    let results = rt.block_on(async {
        join_all(futures).await
    });

    for result in results {
        println!("Result is {}", result);
    }
}