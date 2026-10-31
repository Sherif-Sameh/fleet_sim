use tokio;
use tokio::time::{Duration, sleep};

use crate::types::*;
pub mod types;

async fn hello_world() {
    sleep(Duration::from_secs(1)).await;
    println!("Hello, world!");
    let robot = RobotId(0);
    let content_hash = ContentHash([0xaa; 32]);
    println!("Robot: {robot}");
    println!("SHA-256: {content_hash}");
}

#[tokio::main]
async fn main() {
    let handle = tokio::spawn(hello_world());
    handle.await.unwrap();
}
