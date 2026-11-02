use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{mpsc, oneshot, watch};
use tokio::time::{Instant, MissedTickBehavior, interval};
use tracing::{debug, info};

use crate::error::{AgentError, Result};
use crate::message::{Message, RobotStatus};
use crate::state::{UpdateEvent, UpdateState};
use crate::store::Store;
use crate::transport::Transport;
use crate::types::{ContentHash, Package, PackageState, RobotId};

/// Agent configuration.
///
/// Derives the [`Debug`] and [`Clone`] traits. Implements the [`Default`] trait.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub gossip_interval: Duration,
    pub reconcile_interval: Duration,
    pub health_check_timeout: Duration,
    pub peer_gossip_timeout: Duration,
    pub peer_request_timeout: Duration,
    pub chunk_size: usize,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            gossip_interval: Duration::from_millis(500),
            reconcile_interval: Duration::from_millis(250),
            health_check_timeout: Duration::from_secs(5),
            peer_gossip_timeout: Duration::from_secs(5),
            peer_request_timeout: Duration::from_secs(2),
            chunk_size: 16 * 1024,
        }
    }
}

/// Lightweight agent handle for user interface. Used to request commands and track robot
/// state.
///
/// Derives the [`Clone`] trait.
#[derive(Clone)]
pub struct AgentHandle {
    id: RobotId,
    cmd_tx: mpsc::Sender<Command>,
    status_rx: watch::Receiver<RobotStatus>,
}

enum Command {
    RequestUpdate(PackageState),
    RequestNeighbours(oneshot::Sender<Vec<RobotId>>),
}

impl AgentHandle {
    pub fn id(&self) -> RobotId {
        self.id
    }

    /// Request an update to a `desired` [`PackageState`] through the mpsc channel.
    pub async fn request_update(&self, desired: PackageState) -> Result<()> {
        self.cmd_tx
            .send(Command::RequestUpdate(desired))
            .await
            .map_err(|_| AgentError::AgentStopped)
    }

    /// Get latest published [`RobotStatus`] snapshot.
    pub fn status(&self) -> RobotStatus {
        self.status_rx.borrow().clone()
    }

    /// Get a subscriber to the latest published [`RobotStatus`].
    pub fn subscribe(&self) -> watch::Receiver<RobotStatus> {
        self.status_rx.clone()
    }

    /// Get robot neighbors through a request/reply via a oneshot channel.
    pub async fn neighbours(&self) -> Result<Vec<RobotId>> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.cmd_tx
            .send(Command::RequestNeighbours(reply_tx))
            .await
            .map_err(|_| AgentError::AgentStopped)?;
        reply_rx.await.map_err(|_| AgentError::AgentStopped)
    }
}

/// On-board agent for managing the robot's state. Handles commands and peer-to-peer communication.
///
/// Started through the [`Agent::spawn`] method, which returns an [`AgentHandle`] for control.
pub struct Agent<S, T>
where
    S: Store,
    T: Transport,
{
    id: RobotId,
    cfg: AgentConfig,
    status: RobotStatus,
    store: Arc<S>,
    transport: T,
    cmd_rx: mpsc::Receiver<Command>,
    status_tx: watch::Sender<RobotStatus>,
    sent_at: Option<Instant>,
    peers: HashMap<RobotId, PeerInfo>,
}

struct PeerInfo {
    last_seen: Instant,
    have: HashSet<ContentHash>,
}

impl<S, T> Agent<S, T>
where
    S: Store,
    T: Transport,
{
    pub fn spawn(id: RobotId, cfg: AgentConfig, store: S, transport: T) -> AgentHandle {
        let (cmd_tx, cmd_rx) = mpsc::channel(32);
        let status = RobotStatus {
            id,
            installed: None,
            previous: None,
            desired: None,
            state: UpdateState::Inactive,
        };
        let (status_tx, status_rx) = watch::channel(status.clone());
        let agent = Agent {
            id,
            cfg,
            status,
            store: Arc::new(store),
            transport,
            cmd_rx,
            status_tx,
            sent_at: None,
            peers: HashMap::new(),
        };
        tokio::spawn(agent.run());
        AgentHandle {
            id,
            cmd_tx,
            status_rx,
        }
    }

    async fn run(mut self) {
        let mut gossip = interval(self.cfg.gossip_interval);
        let mut reconcile = interval(self.cfg.reconcile_interval);
        gossip.set_missed_tick_behavior(MissedTickBehavior::Delay);
        reconcile.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                cmd = self.cmd_rx.recv() => match cmd {
                    Some(cmd) => self.on_command(cmd),
                    None => break,
                },
                msg = self.transport.recv() => match msg {
                    Some((from, msg)) => self.on_message(from, msg).await,
                    None => break,
                },
                _ = gossip.tick() => self.send_gossip().await,
                _ = reconcile.tick() => self.reconcile().await,
            }
            self.publish_status();
        }
        info!(robot = %self.id, "agent stopped");
    }

    /* inputs */

    fn on_command(&mut self, cmd: Command) {
        match cmd {
            Command::RequestUpdate(desired) => self.adopt_desired(desired),
            Command::RequestNeighbours(neighbors_tx) => {
                self.peers
                    .retain(|_, p| p.last_seen.elapsed() < self.cfg.peer_gossip_timeout);
                let _ = neighbors_tx.send(self.peers.keys().copied().collect());
            }
        }
    }

    async fn on_message(&mut self, from: RobotId, msg: Message) {
        match msg {
            Message::Gossip { status, have } => {
                if let Some(desired) = status.desired.clone() {
                    self.adopt_desired(desired);
                }
                self.peers.insert(
                    from,
                    PeerInfo {
                        last_seen: Instant::now(),
                        have: have.into_iter().collect(),
                    },
                );
            }
            Message::ChunkRequest {
                hash,
                offset,
                max_len,
            } => {
                self.serve_chunk(from, hash, offset, max_len).await;
            }
            Message::ChunkResponse { hash, offset, data } => {
                self.sent_at = None;
                match self.store.write_chunk(&hash, offset, &data).await {
                    Ok(new_len) => {
                        self.transition(UpdateEvent::Progress(new_len)).unwrap();
                    }
                    Err(_) => {}
                }
            }
            Message::NotAvailable { hash } => {
                debug!(robot = %self.id, %from, %hash, "peer doesn't have it");
            }
        }
    }

    async fn serve_chunk(&self, to: RobotId, hash: ContentHash, offset: u64, max_len: u32) {
        let reply = match self.store.read_chunk(&hash, offset, max_len as usize).await {
            Ok(data) => Message::ChunkResponse { hash, offset, data },
            Err(e) => {
                debug!(robot = %self.id, error = %e, "cannot serve chunk");
                Message::NotAvailable { hash }
            }
        };
        if let Err(e) = self.transport.send(to, reply).await {
            debug!(robot = %self.id, error = %e, "reply dropped");
        }
    }

    /* periodic */

    fn adopt_desired(&mut self, desired: PackageState) {
        if desired.supersedes(self.status.desired.as_ref()) {
            info!(robot = %self.id, rev = desired.revision, pkg = %desired.package, "new desired state");
            self.status.desired = Some(desired);
        }
    }

    async fn send_gossip(&self) {
        let have = self.store.have().await;
        let reached = self
            .transport
            .broadcast(Message::Gossip {
                status: self.status.clone(),
                have,
            })
            .await;
        debug!(robot = %self.id, reached, "gossiped");
    }

    async fn reconcile(&mut self) {
        let Some(desired) = self.status.desired.clone() else {
            return;
        };
        if self.status.installed.as_ref() == Some(&desired.package) {
            return;
        }
        match self.status.state.clone() {
            UpdateState::Inactive => self.start(desired.package).await,
            UpdateState::RolledBack { target: failed, .. }
            | UpdateState::Failed { target: failed, .. } => {
                if failed != desired.package {
                    self.start(desired.package).await
                }
            }
            UpdateState::Downloading { target, received } => {
                self.drive_download(target, received).await
            }
            UpdateState::Verifying { target } => {
                if self.store.finalize(&target.hash).await.is_ok() {
                    let _ = self.transition(UpdateEvent::Verified);
                } else {
                    self.status.state = UpdateState::Failed {
                        target,
                        reason: String::from("checksum failed"),
                    };
                }
            }
            UpdateState::Staging { .. } => {
                self.stage().await;
                self.transition(UpdateEvent::Staged).unwrap();
            }
            UpdateState::Activating { target } => {
                self.status.previous = self.status.installed.clone();
                self.activate().await;
                self.status.installed = Some(target);
                self.transition(UpdateEvent::Activated).unwrap();
            }
            UpdateState::HealthChecking { .. } => {
                match tokio::time::timeout(self.cfg.health_check_timeout, self.health_check()).await
                {
                    Ok(success) => {
                        if success {
                            self.transition(UpdateEvent::HealthOk).unwrap();
                        } else {
                            self.transition(UpdateEvent::HealthFailed(String::from(
                                "incompatible package",
                            )))
                            .unwrap();
                            self.status.installed = self.status.previous.clone();
                        }
                    }
                    Err(_) => {} // timed-out; wait for next tick}
                }
            }
        }
    }

    async fn start(&mut self, pkg: Package) {
        let have = self.store.partial_len(&pkg.hash).await;
        if self.transition(UpdateEvent::Begin(pkg)).is_ok() && have > 0 {
            let _ = self.transition(UpdateEvent::Progress(have));
        }
    }

    async fn drive_download(&mut self, target: Package, received: u64) {
        if received >= target.size {
            let _ = self.transition(UpdateEvent::DownloadComplete);
            return;
        }
        if self
            .sent_at
            .as_ref()
            .is_some_and(|p| p.elapsed() < self.cfg.peer_request_timeout)
        {
            return; // waiting for a response from peer
        }
        let Some(peer) = self
            .peers
            .iter()
            .filter(|(_, p)| p.have.contains(&target.hash))
            .min_by_key(|(_, p)| p.last_seen.elapsed().as_millis())
            .map(|(&id, _)| id)
        else {
            return; // nobody has it yet; try next tick
        };
        let req = Message::ChunkRequest {
            hash: target.hash,
            offset: received,
            max_len: self.cfg.chunk_size as u32,
        };
        if self.transport.send(peer, req).await.is_ok() {
            self.sent_at = Some(Instant::now());
        }
    }

    async fn stage(&mut self) {
        // Not yet implemented
    }

    async fn activate(&mut self) {
        // Not yet implemented
    }

    async fn health_check(&mut self) -> bool {
        // Not yet implemented
        true
    }

    fn transition(&mut self, event: UpdateEvent) -> Result<()> {
        self.status.state = std::mem::take(&mut self.status.state).apply(event)?;
        Ok(())
    }

    /* status */

    fn publish_status(&self) {
        self.status_tx.send_if_modified(|cur| {
            if *cur == self.status {
                false
            } else {
                *cur = self.status.clone();
                true
            }
        });
    }
}
