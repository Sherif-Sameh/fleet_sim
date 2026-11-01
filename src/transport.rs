use std::future::Future;

use crate::error::Result;
use crate::message::Message;
use crate::types::RobotId;

/// Transport layer trait for robot-robot communication.
pub trait Transport: Send + Sync + 'static {
    /// Unicast to a specific neighbor. Errors if `to` is not currently reachable.
    fn send(&self, to: RobotId, msg: Message) -> impl Future<Output = Result<()>> + Send;

    /// Broadcast to all reachable neighbors. Returns how many were reached.
    fn broadcast(&self, msg: Message) -> impl Future<Output = usize> + Send;

    /// Receive next incoming message (and sender ID) from neighbors. `None` if transport
    /// has shut down.  
    fn recv(&mut self) -> impl Future<Output = Option<(RobotId, Message)>> + Send;
}
