use serde::{Deserialize, Serialize};

use crate::errors::{AgentError, Result};
use crate::types::Package;

/// Agent update state. Defines the state transitions for package updates.
///
/// Derives the [`Debug`], [`Clone`], [`Default`], [`PartialEq`], [`Eq`], [`serde::Serialize`],
/// and [`serde::Deserialize`] traits.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateState {
    #[default]
    Inactive,
    Downloading {
        target: Package,
        received: u32,
    },
    Verifying {
        target: Package,
    },
    Staging {
        target: Package,
    },
    Activating {
        target: Package,
    },
    HealthChecking {
        target: Package,
    },
    RolledBack {
        target: Package,
        reason: String,
    },
    Failed {
        target: Package,
        reason: String,
    },
}

/// Agent update events. Defines the triggers for state transitions for package updates.
///
/// Derives the [`Debug`] and [`Clone`] traits.
#[derive(Debug, Clone)]
pub enum UpdateEvent {
    Begin(Package),
    Progress(u32),
    DownloadComplete,
    Verified,
    Staged,
    Activated,
    HealthOk,
    HealthFailed(String),
}

impl UpdateState {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Inactive => "inactive",
            Self::Downloading { .. } => "downloading",
            Self::Verifying { .. } => "verifying",
            Self::Staging { .. } => "staging",
            Self::Activating { .. } => "activating",
            Self::HealthChecking { .. } => "health-checking",
            Self::RolledBack { .. } => "rolled-back",
            Self::Failed { .. } => "failed",
        }
    }

    pub fn is_busy(&self) -> bool {
        match self {
            Self::Inactive | Self::RolledBack { .. } | Self::Failed { .. } => false,
            _ => true,
        }
    }

    pub fn apply(self, event: UpdateEvent) -> Result<Self> {
        use UpdateEvent as E;
        use UpdateState as S;

        match (self, event) {
            (S::Inactive | S::RolledBack { .. } | S::Failed { .. }, E::Begin(target)) => {
                Ok(S::Downloading {
                    target,
                    received: 0,
                })
            }
            (S::Downloading { target, .. }, E::Progress(received)) => Ok(S::Downloading {
                target: target,
                received,
            }),
            (S::Downloading { target, .. }, E::DownloadComplete) => {
                Ok(S::Verifying { target: target })
            }
            (S::Verifying { target }, E::Verified) => Ok(S::Staging { target: target }),
            (S::Staging { target }, E::Staged) => Ok(S::Activating { target: target }),
            (S::HealthChecking { .. }, E::HealthOk) => Ok(S::Inactive),
            (S::HealthChecking { target }, E::HealthFailed(reason)) => Ok(S::RolledBack {
                target: target,
                reason: reason,
            }),
            (status, event) => Err(AgentError::InvalidTransition {
                from: status.name(),
                event: format!("{event:?}"),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Version;

    fn pkg() -> Package {
        Package::from_data("nav", Version::from_string("1.0.0"), b"hello")
    }

    #[test]
    fn happy_path() {
        let p = UpdateState::Inactive
            .apply(UpdateEvent::Begin(pkg()))
            .unwrap()
            .apply(UpdateEvent::Progress(3))
            .unwrap()
            .apply(UpdateEvent::DownloadComplete)
            .unwrap();
        assert!(matches!(p, UpdateState::Verifying { .. }));
    }

    #[test]
    fn invalid_combination() {
        assert!(UpdateState::Inactive.apply(UpdateEvent::HealthOk).is_err());
    }
}
