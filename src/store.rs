pub mod memory;

use std::future::Future;

use crate::error::Result;
use crate::types::ContentHash;

/// Trait for package stores.
///
/// Has [`Send`] and [`Sync`] as supertraits. Implementing types must have no short-lived
/// references.
pub trait Store: Send + Sync + 'static {
    /// Returns a vector of hashes of complete packages.
    fn have(&self) -> impl Future<Output = Vec<ContentHash>> + Send;

    /// Returns true if the package corresponding to `hash` is found and complete, false otherwise.
    fn is_complete(&self, hash: &ContentHash) -> impl Future<Output = bool> + Send;

    /// Returns the number of stored bytes of the package corresponding to `hash`.
    fn remaining_len(&self, hash: &ContentHash) -> impl Future<Output = u64> + Send;

    /// Returns the bytes of the package corresponding to `hash` at the set `offset` for
    /// a maximum length of `max_len`. Errors if `hash` is not found.
    fn read_chunk(
        &self,
        hash: &ContentHash,
        offset: u64,
        max_len: usize,
    ) -> impl Future<Output = Result<Vec<u8>>> + Send;

    /// Appends the bytes of `data` to the package blob at the set `offset`.
    /// `offset` must be equal to the current partial length. Returns the new length.
    fn write_chunk(
        &self,
        hash: &ContentHash,
        offset: u64,
        data: &[u8],
    ) -> impl Future<Output = Result<u64>> + Send;

    /// Verifies the hash of the assembled package blob. Marks it as complete or discards it.
    fn finalize(&self, hash: &ContentHash) -> impl Future<Output = Result<()>> + Send;
}

/// Raw package blob struct.
///
/// Derives the [`Default`] trait.
#[derive(Default)]
struct Blob {
    data: Vec<u8>,
    complete: bool,
}
