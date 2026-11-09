use std::error::Error;

use glam::Vec2;
use tokio::time::Duration;
use tracing::info;
use tracing_subscriber::EnvFilter;

use fleet_sim::agent::{Agent, AgentConfig, AgentHandle};
use fleet_sim::store::memory::MemoryStore;
use fleet_sim::transport::radio::RadioNetwork;
use fleet_sim::types::{PackageState, RobotId, Version};

/// Custom Result type alias using a boxed dynamic error.
pub type ResultBox<T> = std::result::Result<T, Box<dyn Error + Send + Sync + 'static>>;

#[tokio::main]
async fn main() -> ResultBox<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    // Setup robot network such that each robot has only a single neighbor in range
    let (max_range, drop_prob) = (10.0, 0.03);
    let net = RadioNetwork::new(max_range, drop_prob);
    let mut agents: Vec<AgentHandle> = Vec::new();
    let mut pkg_des = None;
    for i in 0..5u32 {
        let id = RobotId(i);
        let store = MemoryStore::default();
        if i == 0 {
            // Only the first robot starts out holding the desired package
            pkg_des =
                Some(store.publish("cv2", Version::from_string("2.3.9"), vec![0xAB; 100_000]));
        }
        let transport = net.register(id, Vec2::new(i as f32 * 8.0, 0.0));
        agents.push(Agent::spawn(id, AgentConfig::default(), store, transport));
    }
    // Publish the desired package state across the network
    agents[0]
        .request_update(PackageState {
            package: pkg_des.unwrap(),
            revision: 1,
        })
        .await?;
    // Await for the desired package to propagate across the network
    // and report the fleet's state at each timestep
    for t in 0..12 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let line = agents
            .iter()
            .map(|a| {
                let s = a.status();
                let rev = s.desired.as_ref().map_or(0, |p| p.revision);
                format!("{}[rev {rev}, {}]", s.id, s.state.name())
            })
            .collect::<Vec<_>>()
            .join("  ");
        info!("t={t}s  {line}\n");
    }
    Ok(())
}
