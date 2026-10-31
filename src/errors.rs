use crate::types::{ContentHash, RobotId};

/// Errors that can be triggered by the on-board agent.
///
/// Derives the [`Debug`] and [`thiserror::Error`] traits.
#[derive(Debug, thiserror::Error)]
pub enum AgentError {}
