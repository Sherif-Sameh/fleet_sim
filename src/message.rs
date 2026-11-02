use serde::{Deserialize, Serialize};

use crate::state::UpdateState;
use crate::types::{ContentHash, Package, PackageState, RobotId};

/// Message types for robot communication protocol
///
/// Derives the [`Debug`], [`Clone`], [`serde::Serialize`], and [`serde::Deserialize`] traits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    Gossip {
        status: RobotStatus,
        have: Vec<ContentHash>,
    },
    ChunkRequest {
        hash: ContentHash,
        offset: u64,
        max_len: u32,
    },
    ChunkResponse {
        hash: ContentHash,
        offset: u64,
        data: Vec<u8>,
    },
    NotAvailable {
        hash: ContentHash,
    },
}

/// Robot status shared with nearby robots.
///
/// Derives the [`Debug`], [`Clone`], [`PartialEq`], [`Eq`], [`serde::Serialize`], and
/// [`serde::Deserialize`] traits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RobotStatus {
    pub id: RobotId,
    pub installed: Option<Package>,
    pub previous: Option<Package>,
    pub desired: Option<PackageState>,
    pub state: UpdateState,
}
