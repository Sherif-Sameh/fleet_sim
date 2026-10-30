use tokio;
use tokio::time::{Duration, sleep};

async fn hello_world() {
    sleep(Duration::from_secs(1)).await;
    println!("Hello, world!");
}

#[tokio::main]
async fn main() {
    let handle = tokio::spawn(hello_world());
    handle.await.unwrap();
}
