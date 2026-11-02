use tokio;
use tokio::time::{Duration, sleep};

use crate::types::*;
pub mod agent;
pub mod error;
pub mod message;
pub mod state;
pub mod store;
pub mod transport;
pub mod types;

async fn hello_world() {
    sleep(Duration::from_secs(1)).await;
    println!("Hello, world!");
    let robot = RobotId(0);
    let content_hash = ContentHash([0xaa; 32]);
    let version = Version::from_string("1.32.1");
    let package = Package::from_data("torch", version, &[0x2a; 200]);
    let state = PackageState {
        package: package.clone(),
        revision: 60,
    };
    println!("Robot: {robot}");
    println!("SHA-256: {content_hash}");
    println!("Version: {version}");
    println!("Package: {package}");
    println!("State: {state}");
}

#[tokio::main]
async fn main() {
    let handle = tokio::spawn(hello_world());
    handle.await.unwrap();
}
