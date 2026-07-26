use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt;
use core::ops::Range;

use eth_valkyoth_codec::DecodeSession;
use eth_valkyoth_hash::{Keccak256, hash_one};
use eth_valkyoth_primitives::B256;

use crate::{
    MptNodeResolver, MptResolvedNode, MptResolverError, MptResolverLimits, MptSnapshotAnchor,
};

/// Owned, byte-bounded cache of encoded MPT nodes.
///
/// Construction hashes, sorts, and deduplicates nodes. The arena never changes
/// its snapshot anchor. Call [`Self::with_resolver`] to expose a temporary
/// allocation-free resolver to verification code.
pub struct MptOwnedNodeArena {
    anchor: MptSnapshotAnchor,
    nodes: Vec<OwnedNode>,
    limits: MptResolverLimits,
    retained_bytes: usize,
}

struct OwnedNode {
    hash: B256,
    encoded: Vec<u8>,
}

impl MptOwnedNodeArena {
    /// Builds an owned arena under explicit node and retained-byte limits.
    pub fn try_new<H>(
        anchor: MptSnapshotAnchor,
        encoded_nodes: Vec<Vec<u8>>,
        limits: MptResolverLimits,
        max_retained_bytes: usize,
        mut new_hasher: impl FnMut() -> H,
    ) -> Result<Self, MptArenaError>
    where
        H: Keccak256,
    {
        if max_retained_bytes == 0 {
            return Err(MptArenaError::InvalidRetainedByteLimit);
        }
        let mut retained_bytes = 0_usize;
        let mut nodes = Vec::new();
        nodes
            .try_reserve_exact(encoded_nodes.len())
            .map_err(|_| MptArenaError::Allocation)?;
        for encoded in encoded_nodes {
            retained_bytes = retained_bytes
                .checked_add(encoded.len())
                .ok_or(MptArenaError::RetainedBytesExceeded)?;
            if retained_bytes > max_retained_bytes {
                return Err(MptArenaError::RetainedBytesExceeded);
            }
            nodes.push(OwnedNode {
                hash: hash_one(new_hasher(), &encoded),
                encoded,
            });
        }
        nodes.sort_by(|left, right| compare_hash(left.hash, right.hash));

        let mut deduplicated: Vec<OwnedNode> = Vec::new();
        deduplicated
            .try_reserve_exact(nodes.len())
            .map_err(|_| MptArenaError::Allocation)?;
        for node in nodes {
            match deduplicated.last() {
                Some(previous) if previous.hash == node.hash => {
                    if previous.encoded != node.encoded {
                        return Err(MptArenaError::HashCollision);
                    }
                    retained_bytes = retained_bytes.saturating_sub(node.encoded.len());
                }
                _ => deduplicated.push(node),
            }
        }
        Ok(Self {
            anchor,
            nodes: deduplicated,
            limits,
            retained_bytes,
        })
    }

    /// Returns retained encoded-node bytes after deduplication.
    #[must_use]
    pub const fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }

    /// Returns unique cached node count.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Admits a temporary borrowed resolver and invokes `operation`.
    ///
    /// The resolver cannot escape the closure or outlive this immutable arena.
    pub fn with_resolver<H, T>(
        &self,
        session: &mut DecodeSession,
        new_hasher: impl FnMut() -> H,
        operation: impl FnOnce(&MptNodeResolver<'_>, &mut DecodeSession) -> Result<T, MptResolverError>,
    ) -> Result<T, MptArenaError>
    where
        H: Keccak256,
    {
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(self.nodes.len())
            .map_err(|_| MptArenaError::Allocation)?;
        for node in &self.nodes {
            entries.push(MptResolvedNode::new(node.hash, &node.encoded));
        }
        let resolver =
            MptNodeResolver::try_new(self.anchor, &entries, self.limits, session, new_hasher)
                .map_err(MptArenaError::Resolver)?;
        operation(&resolver, session).map_err(MptArenaError::Resolver)
    }
}

/// Cooperative cancellation boundary for host-side batch scheduling.
pub trait MptCancellation {
    /// Returns whether pending work should stop before the next range.
    fn is_cancelled(&self) -> bool;
}

/// Deterministic bounded query-range schedule.
pub struct MptBatchSchedule {
    query_count: usize,
    chunk_size: usize,
    next: usize,
}

impl MptBatchSchedule {
    /// Creates a schedule with at most `max_parallelism` independent ranges.
    pub fn reviewed(query_count: usize, max_parallelism: usize) -> Result<Self, MptScheduleError> {
        if query_count == 0 || max_parallelism == 0 {
            return Err(MptScheduleError::InvalidLimit);
        }
        let workers = core::cmp::min(query_count, max_parallelism);
        let chunk_size = query_count
            .checked_add(workers.saturating_sub(1))
            .and_then(|rounded| rounded.checked_div(workers))
            .ok_or(MptScheduleError::InvalidLimit)?;
        Ok(Self {
            query_count,
            chunk_size,
            next: 0,
        })
    }

    /// Drives ranges in deterministic order and checks cancellation before
    /// each range. Hosts may submit returned ranges to parallel workers, but
    /// must merge results in range order.
    pub fn try_for_each(
        self,
        cancellation: &impl MptCancellation,
        mut operation: impl FnMut(Range<usize>) -> Result<(), MptScheduleError>,
    ) -> Result<(), MptScheduleError> {
        for range in self {
            if cancellation.is_cancelled() {
                return Err(MptScheduleError::Cancelled);
            }
            operation(range)?;
        }
        Ok(())
    }
}

impl Iterator for MptBatchSchedule {
    type Item = Range<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next >= self.query_count {
            return None;
        }
        let start = self.next;
        let end = core::cmp::min(start.saturating_add(self.chunk_size), self.query_count);
        self.next = end;
        Some(start..end)
    }
}

/// Owned arena construction or resolver failure.
#[non_exhaustive]
#[derive(Debug)]
pub enum MptArenaError {
    /// Retained-byte bound was zero.
    InvalidRetainedByteLimit,
    /// Retained encoded bytes exceeded the configured bound.
    RetainedBytesExceeded,
    /// Allocation failed.
    Allocation,
    /// Equal hashes were observed for unequal node bytes.
    HashCollision,
    /// Borrowed resolver admission or verification failed.
    Resolver(MptResolverError),
}

impl fmt::Display for MptArenaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("MPT owned node arena failed")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for MptArenaError {}

/// Host-side work scheduling failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MptScheduleError {
    /// Query count or parallelism was zero.
    InvalidLimit,
    /// Cancellation was observed before the next work range.
    Cancelled,
    /// Host range processing failed.
    Operation,
}

impl fmt::Display for MptScheduleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("MPT batch scheduling failed")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for MptScheduleError {}

fn compare_hash(left: B256, right: B256) -> Ordering {
    left.to_bytes().iter().cmp(right.to_bytes().iter())
}

#[cfg(test)]
#[path = "mpt_resolver_alloc_tests.rs"]
mod tests;
