use alloc::{boxed::Box, vec::Vec};
use core::cmp::Ordering;
use core::fmt;
use core::mem::size_of;
use core::ops::Range;

use eth_valkyoth_codec::DecodeSession;
use eth_valkyoth_hash::{Keccak256, hash_one};
use eth_valkyoth_primitives::B256;

use crate::mpt::decode_mpt_node_body_in_session;
use crate::{
    MptNodeResolver, MptProofVerificationError, MptResolvedNode, MptResolverError,
    MptResolverLimits, MptSnapshotAnchor,
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
    encoded: Box<[u8]>,
}

impl MptOwnedNodeArena {
    /// Builds an owned arena under explicit node and retained-byte limits.
    ///
    /// Decode, hash, and retained-allocation work is charged to `session`
    /// before the corresponding cryptographic work or allocation begins.
    pub fn try_new<H>(
        anchor: MptSnapshotAnchor,
        encoded_nodes: Vec<Vec<u8>>,
        limits: MptResolverLimits,
        max_retained_bytes: usize,
        session: &mut DecodeSession,
        mut new_hasher: impl FnMut() -> H,
    ) -> Result<Self, MptArenaError>
    where
        H: Keccak256,
    {
        if max_retained_bytes == 0 {
            return Err(MptArenaError::InvalidRetainedByteLimit);
        }
        let node_count = encoded_nodes.len();
        if node_count == 0 || node_count > limits.max_nodes() {
            return Err(MptArenaError::NodeLimitExceeded);
        }
        let payload_bytes = encoded_nodes.iter().try_fold(0_usize, |total, encoded| {
            total
                .checked_add(encoded.len())
                .ok_or(MptArenaError::RetainedBytesExceeded)
        })?;
        let retained_upper_bound = retained_memory(node_count, payload_bytes)?;
        if retained_upper_bound > max_retained_bytes {
            return Err(MptArenaError::RetainedBytesExceeded);
        }
        session
            .account_proof_nodes(node_count)
            .map_err(arena_resource_error)?;
        for encoded in &encoded_nodes {
            session
                .check_input_len(encoded.len())
                .map_err(arena_resource_error)?;
            decode_mpt_node_body_in_session(encoded, session).map_err(|error| {
                MptArenaError::Resolver(MptResolverError::Proof(
                    MptProofVerificationError::MalformedNode(error),
                ))
            })?;
        }
        session
            .check_hash_capacity(node_count, payload_bytes)
            .map_err(arena_resource_error)?;
        session
            .account_allocation_capacity(retained_upper_bound)
            .map_err(arena_resource_error)?;

        let mut nodes = Vec::new();
        nodes
            .try_reserve_exact(node_count)
            .map_err(|_| MptArenaError::Allocation)?;
        for encoded in encoded_nodes {
            session
                .account_hashes(1, encoded.len())
                .map_err(arena_resource_error)?;
            nodes.push(OwnedNode {
                hash: hash_one(new_hasher(), &encoded),
                encoded: encoded.into_boxed_slice(),
            });
        }
        nodes.sort_by(|left, right| compare_hash(left.hash, right.hash));

        for pair in nodes.windows(2) {
            if let [left, right] = pair
                && left.hash == right.hash
                && left.encoded != right.encoded
            {
                return Err(MptArenaError::HashCollision);
            }
        }
        nodes.dedup_by(|left, right| left.hash == right.hash);
        nodes.shrink_to_fit();
        let retained_payload = nodes.iter().try_fold(0_usize, |total, node| {
            total
                .checked_add(node.encoded.len())
                .ok_or(MptArenaError::RetainedBytesExceeded)
        })?;
        let retained_bytes = retained_memory(nodes.capacity(), retained_payload)?;
        if retained_bytes > max_retained_bytes {
            return Err(MptArenaError::RetainedBytesExceeded);
        }
        Ok(Self {
            anchor,
            nodes,
            limits,
            retained_bytes,
        })
    }

    /// Returns accounted payload and node-vector bytes after deduplication.
    ///
    /// Allocator bookkeeping overhead is not portable and is excluded.
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
    /// Temporary resolver-entry allocation is charged to `session`.
    pub fn with_resolver<H, T>(
        &self,
        session: &mut DecodeSession,
        new_hasher: impl FnMut() -> H,
        operation: impl FnOnce(&MptNodeResolver<'_>, &mut DecodeSession) -> Result<T, MptResolverError>,
    ) -> Result<T, MptArenaError>
    where
        H: Keccak256,
    {
        let entry_bytes = self
            .nodes
            .len()
            .checked_mul(size_of::<MptResolvedNode<'_>>())
            .ok_or(MptArenaError::Allocation)?;
        session
            .account_allocation_capacity(entry_bytes)
            .map_err(arena_resource_error)?;
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
    /// The raw input node count was empty or exceeded its configured bound.
    NodeLimitExceeded,
    /// Accounted retained payload and node-vector bytes exceeded the bound.
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

fn retained_memory(node_capacity: usize, payload_bytes: usize) -> Result<usize, MptArenaError> {
    node_capacity
        .checked_mul(size_of::<OwnedNode>())
        .and_then(|metadata| metadata.checked_add(payload_bytes))
        .ok_or(MptArenaError::RetainedBytesExceeded)
}

fn arena_resource_error(error: eth_valkyoth_codec::DecodeError) -> MptArenaError {
    MptArenaError::Resolver(MptResolverError::Resource(error))
}

#[cfg(test)]
#[path = "mpt_resolver_alloc_tests.rs"]
mod tests;
