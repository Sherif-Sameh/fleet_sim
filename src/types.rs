use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Custom Result type alias using a boxed dynamic error.
pub type ResultDyn<T> = std::result::Result<T, Box<dyn Error + Send + Sync + 'static>>;

/// Robot ID.
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

/// Package state. For sharing desired and current states between robots.
///
/// Derives the [`Debug`], [`Clone`], [`PartialEq`], [`Eq`], [`serde::Serialize`], and
/// [`serde::Deserialize`] traits. Implements the [`fmt::Display`] trait.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageState {
    pub package: Package,
    pub revision: u64,
}

impl PackageState {
    pub fn supersedes(&self, current: Option<&PackageState>) -> bool {
        current.map_or(true, |c| self.revision > c.revision)
    }
}

impl fmt::Display for PackageState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:rev{}", self.package, self.revision)
    }
}

/// Package metadata.
///
/// Derives the [`Debug`], [`Clone`], [`PartialEq`], [`Eq`], [`serde::Serialize`], and
/// [`serde::Deserialize`] traits. Implements the [`fmt::Display`] trait.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: Version,
    pub hash: ContentHash,
    pub size: u64,
}

impl Package {
    pub fn from_data(name: &str, version: Version, data: &[u8]) -> Self {
        Self {
            name: String::from(name),
            version,
            hash: ContentHash::from_data(data),
            size: data.len() as u64,
        }
    }
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.version)
    }
}

/// Package version. Contains major, minor, and patch fields.
///
/// Derives the [`Clone`], [`Copy`], [`PartialEq`], [`Eq`], [`PartialOrd`], [`Ord`], [`Hash`],
/// [`serde::Serialize`], and [`serde::Deserialize`] traits. Implements the [`fmt::Display`] and
/// [`fmt::Debug`] traits.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub fn from_string(version: &str) -> Self {
        let version: Vec<u32> = version
            .split(".")
            .map(|n| n.trim().parse::<u32>().unwrap())
            .collect();
        assert_eq!(version.len(), 3);
        Self {
            major: version[0],
            minor: version[1],
            patch: version[2],
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl fmt::Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Version {}", self)
    }
}

/// Wrapper around SHA-256 32-byte array.
///
/// Derives the [`Clone`], [`Copy`], [`PartialEq`], [`Eq`], [`Hash`], [`serde::Serialize`], and
/// [`serde::Deserialize`] traits. Implements the [`fmt::Display`] and [`fmt::Debug`] traits.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash(pub [u8; 32]);

impl ContentHash {
    pub fn from_data(data: &[u8]) -> Self {
        Self(Sha256::digest(data).into())
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
