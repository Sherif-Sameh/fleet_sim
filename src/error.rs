use crate::types::{ContentHash, RobotId};

/// Errors that can be triggered by the on-board agent.
///
/// Derives the [`Debug`] and [`thiserror::Error`] traits.
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("invalid transition from {from} on event {event}")]
    InvalidTransition { from: &'static str, event: String },

    #[error("blob {0} not in store")]
    NotInStore(ContentHash),

    #[error("write at offset {got}, but store expected {expected}")]
    BadOffset { expected: u64, got: u64 },

    #[error("hash mismatch: expected {expected}, got {actual}")]
    HashMismatch {
        expected: ContentHash,
        actual: ContentHash,
    },

    #[error("{0} is out of radio range")]
    OutOfRange(RobotId),

    #[error("lost connection with {0}")]
    ConnectionLost(RobotId),

    #[error("failed to send to {0}")]
    TransmissionFailed(RobotId),

    #[error("agent has stopped")]
    AgentStopped,
}

pub type Result<T, E = AgentError> = std::result::Result<T, E>;
