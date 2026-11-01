/// Errors that can be triggered by the on-board agent.
///
/// Derives the [`Debug`] and [`thiserror::Error`] traits.
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("invalid transition from {from} on event {event}")]
    InvalidTransition { from: &'static str, event: String },
}

pub type Result<T, E = AgentError> = std::result::Result<T, E>;
