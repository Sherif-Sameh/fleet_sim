use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use glam::Vec2;
use tokio::sync::mpsc;

use crate::error::{AgentError, Result};
use crate::message::Message;
use crate::transport::Transport;
use crate::types::RobotId;

type Sender = mpsc::Sender<(RobotId, Message)>;

/// Radio transport layer. Simulates lossy range-based radio communication.
///
/// Implements the [`Transport`] trait.
pub struct RadioTransport {
    id: RobotId,
    net: RadioNetwork,
    receiver: mpsc::Receiver<(RobotId, Message)>,
}

/// Radio communications network. Neighbors within `max_range` can communicate with each other.
///
/// Derives the [`Clone`] trait.
#[derive(Clone)]
pub struct RadioNetwork {
    world: Arc<Mutex<World>>,
    max_range: f32,
    drop_prob: f32,
}

/// Simulated world with virtual robots and their tokio [`mpsc::Sender`] transmitters.
///
/// Derives the [`Default`] trait.
#[derive(Default)]
struct World {
    positions: HashMap<RobotId, Vec2>,
    senders: HashMap<RobotId, Sender>,
}

impl Transport for RadioTransport {
    async fn send(&self, to: RobotId, msg: Message) -> Result<()> {
        let (_, tx) = self
            .net
            .neighbours(self.id)
            .into_iter()
            .find(|(id, _)| *id == to)
            .ok_or(AgentError::OutOfRange(to))?;
        if rand::random::<f32>() < self.net.drop_prob {
            return Err(AgentError::ConnectionLost(to));
        }
        tx.try_send((self.id, msg))
            .map_err(|_| AgentError::TransmissionFailed(to))?;
        Ok(())
    }

    async fn broadcast(&self, msg: Message) -> usize {
        self.net
            .neighbours(self.id)
            .into_iter()
            .filter(|(_, tx)| tx.try_send((self.id, msg.clone())).is_ok())
            .count()
    }

    async fn recv(&mut self) -> Option<(RobotId, Message)> {
        self.receiver.recv().await
    }
}

impl RadioNetwork {
    pub fn new(max_range: f32, drop_prob: f32) -> Self {
        Self {
            world: Arc::default(),
            max_range,
            drop_prob,
        }
    }

    pub fn register(&self, id: RobotId, pos: Vec2) -> RadioTransport {
        let (tx, rx) = mpsc::channel(64);
        let mut w = self.world.lock().unwrap();
        w.positions.insert(id, pos);
        w.senders.insert(id, tx);
        RadioTransport {
            id,
            net: self.clone(),
            receiver: rx,
        }
    }

    pub fn move_robot(&self, id: RobotId, pos: Vec2) {
        self.world.lock().unwrap().positions.insert(id, pos);
    }

    fn neighbours(&self, from: RobotId) -> Vec<(RobotId, Sender)> {
        let w = self.world.lock().unwrap();
        let &cur = w.positions.get(&from).unwrap_or(&Vec2::ZERO);
        w.positions
            .iter()
            .filter(|&(&id, &pos)| id != from && cur.distance(pos) <= self.max_range)
            .filter_map(|(&id, _)| w.senders.get(&id).map(|tx| (id, tx.clone())))
            .collect()
    }
}
