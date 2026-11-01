pub mod memory;

use std::future::Future;

use crate::error::Result;
use crate::types::ContentHash;

/// Trait for package stores.
///
/// Has [`Send`] and [`Sync`] as supertraits. Implementing types must have no short-lived
/// references.
pub trait Store: Send + Sync + 'static {
    fn is_complete(&self, hash: &ContentHash) -> impl Future<Output = bool> + Send;

    fn remaining_len(&self, hash: &ContentHash) -> impl Future<Output = u64> + Send;

    fn read_chunk(
        &self,
        hash: &ContentHash,
        offset: u64,
        max_len: usize,
    ) -> impl Future<Output = Result<Vec<u8>>> + Send;

    fn write_chunk(
        &self,
        hash: &ContentHash,
        offset: u64,
        data: &[u8],
    ) -> impl Future<Output = Result<u64>> + Send;

    fn finalize(&self, hash: &ContentHash) -> impl Future<Output = Result<()>> + Send;
}
