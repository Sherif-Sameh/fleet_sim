use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
// use sha2::{Digest, Sha256};

/// Tuple struct wrapper around robot ID.
///
/// Derives the [`Debug`], [`Clone`], [`Copy`], [`PartialEq`], [`Eq`], [`PartialOrd`], [`Ord`]
/// [`Hash`], [`serde::Serialize`], and [`serde::Deserialize`] traits. Implements the
/// [`fmt::Display`] trait.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RobotId(pub u32);

impl fmt::Display for RobotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Robot {}", self.0)
    }
}

/// Tuple struct wrapper around the SHA-256 32-byte array.
///
/// Derives the [`Clone`], [`Copy`], [`PartialEq`], [`Eq`], [`Hash`], [`serde::Serialize`], and
/// [`serde::Deserialize`] traits. Implements the [`fmt::Display`] and [`fmt::Debug`] traits.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash(pub [u8; 32]);

impl ContentHash {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(Sha256::digest(bytes).into())
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0[..8] {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl fmt::Debug for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ContentHash({self})")
    }
}
