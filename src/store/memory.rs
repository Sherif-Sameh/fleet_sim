use std::collections::HashMap;
use std::sync::Mutex;

use crate::error::{AgentError, Result};
use crate::store::{Blob, Store};
use crate::types::{ContentHash, Package, Version};

/// Package store for memory storage via a [`HashMap`].
///
/// Each package blob is retrieved via its [`ContentHash`] and stores the raw data and its
/// status via a [`Blob`] struct. Packages can be added (sync) in one go or sent (async) over
/// several chunks.
///
/// Derives the [`Default`] trait.
#[derive(Default)]
pub struct MemoryStore {
    blobs: Mutex<HashMap<ContentHash, Blob>>,
}

impl MemoryStore {
    pub fn publish(&self, name: &str, version: Version, data: Vec<u8>) -> Package {
        let pkg = Package::from_data(name, version, &data);
        self.blobs.lock().unwrap().insert(
            pkg.hash,
            Blob {
                data,
                complete: true,
            },
        );
        pkg
    }
}

impl Store for MemoryStore {
    async fn have(&self) -> Vec<ContentHash> {
        self.blobs
            .lock()
            .unwrap()
            .iter()
            .filter_map(|(&h, b)| (b.complete).then_some(h))
            .collect()
    }

    async fn is_complete(&self, hash: &ContentHash) -> bool {
        self.blobs
            .lock()
            .unwrap()
            .get(hash)
            .is_some_and(|b| b.complete)
    }

    async fn partial_len(&self, hash: &ContentHash) -> u64 {
        self.blobs
            .lock()
            .unwrap()
            .get(hash)
            .map_or(0, |b| b.data.len() as u64)
    }

    async fn read_chunk(&self, hash: &ContentHash, offset: u64, max_len: usize) -> Result<Vec<u8>> {
        let blobs = self.blobs.lock().unwrap();
        let blob = blobs
            .get(hash)
            .filter(|b| b.complete)
            .ok_or(AgentError::NotInStore(*hash))?;
        let start = (offset as usize).min(blob.data.len());
        let end = start.saturating_add(max_len).min(blob.data.len());
        Ok(blob.data[start..end].to_vec())
    }

    async fn write_chunk(&self, hash: &ContentHash, offset: u64, data: &[u8]) -> Result<u64> {
        let mut blobs = self.blobs.lock().unwrap();
        let blob = blobs.entry(*hash).or_default();
        let expected = blob.data.len() as u64;
        if offset != expected {
            return Err(AgentError::BadOffset {
                expected,
                got: offset,
            });
        }
        blob.data.extend_from_slice(data);
        Ok(blob.data.len() as u64)
    }

    async fn finalize(&self, hash: &ContentHash) -> Result<()> {
        let mut blobs = self.blobs.lock().unwrap();
        let blob = blobs.get_mut(hash).ok_or(AgentError::NotInStore(*hash))?;
        let expected = ContentHash::from_data(&blob.data);
        if *hash != expected {
            blobs.remove(hash);
            return Err(AgentError::HashMismatch {
                expected,
                actual: *hash,
            });
        }
        blob.complete = true;
        Ok(())
    }
}
