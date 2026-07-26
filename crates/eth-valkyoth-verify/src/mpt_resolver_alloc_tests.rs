extern crate std;

use super::*;
use crate::test_crypto::RealKeccak;
use eth_valkyoth_codec::{DecodeLimits, DecodeSessionPolicy};
use std::cell::Cell;
use std::vec;

struct Cancellation {
    checks: Cell<usize>,
    cancel_at: usize,
}

impl MptCancellation for Cancellation {
    fn is_cancelled(&self) -> bool {
        let checks = self.checks.get();
        self.checks.set(checks.saturating_add(1));
        checks >= self.cancel_at
    }
}

#[test]
fn owned_arena_deduplicates_identical_nodes() -> Result<(), MptArenaError> {
    let node = vec![0xc2, 0x20, 0x01];
    let hash = hash_one(RealKeccak::default(), &node);
    let anchor = MptSnapshotAnchor::new(crate::MptProofRoot::from_b256(hash));
    let arena = MptOwnedNodeArena::try_new(
        anchor,
        vec![node.clone(), node.clone()],
        MptResolverLimits::TEST_FIXTURE,
        128,
        &mut test_session()?,
        RealKeccak::default,
    )?;

    assert_eq!(arena.node_count(), 1);
    assert_eq!(
        arena.retained_bytes(),
        arena
            .nodes
            .capacity()
            .saturating_mul(core::mem::size_of::<OwnedNode>())
            .saturating_add(node.capacity())
    );
    Ok(())
}

#[test]
fn owned_arena_rejects_retained_byte_overflow() -> Result<(), MptArenaError> {
    let anchor =
        MptSnapshotAnchor::new(crate::MptProofRoot::from_b256(B256::from_bytes([0_u8; 32])));
    let mut session = test_session()?;
    let result = MptOwnedNodeArena::try_new(
        anchor,
        vec![vec![0_u8; 16]],
        MptResolverLimits::TEST_FIXTURE,
        8,
        &mut session,
        RealKeccak::default,
    );
    assert!(matches!(result, Err(MptArenaError::RetainedBytesExceeded)));
    Ok(())
}

#[test]
fn owned_arena_rejects_raw_node_limit_before_hashing() -> Result<(), MptArenaError> {
    let anchor = MptSnapshotAnchor::new(crate::MptProofRoot::from_b256(hash_one(
        RealKeccak::default(),
        b"x",
    )));
    let limits = MptResolverLimits::reviewed(1, 1, 128).map_err(MptArenaError::Resolver)?;
    let calls = Cell::new(0_usize);
    let mut session = test_session()?;
    let result = MptOwnedNodeArena::try_new(
        anchor,
        vec![Vec::new(), Vec::new()],
        limits,
        128,
        &mut session,
        || {
            calls.set(calls.get().saturating_add(1));
            RealKeccak::default()
        },
    );

    assert!(matches!(result, Err(MptArenaError::NodeLimitExceeded)));
    assert_eq!(calls.get(), 0);
    assert_eq!(session.hashes(), 0);
    Ok(())
}

#[test]
fn owned_arena_rejects_malformed_node_before_hashing() -> Result<(), MptArenaError> {
    let anchor = MptSnapshotAnchor::new(crate::MptProofRoot::from_b256(hash_one(
        RealKeccak::default(),
        b"x",
    )));
    let calls = Cell::new(0_usize);
    let mut session = test_session()?;
    let result = MptOwnedNodeArena::try_new(
        anchor,
        vec![Vec::new()],
        MptResolverLimits::TEST_FIXTURE,
        128,
        &mut session,
        || {
            calls.set(calls.get().saturating_add(1));
            RealKeccak::default()
        },
    );

    assert!(matches!(result, Err(MptArenaError::Resolver(_))));
    assert_eq!(calls.get(), 0);
    assert_eq!(session.hashes(), 0);
    Ok(())
}

#[test]
fn owned_arena_rejects_complete_hash_budget_before_hashing() -> Result<(), MptArenaError> {
    let encoded = vec![0xc2, 0x20, 0x01];
    let hash = hash_one(RealKeccak::default(), &encoded);
    let anchor = MptSnapshotAnchor::new(crate::MptProofRoot::from_b256(hash));
    let calls = Cell::new(0_usize);
    let mut session = test_session_with_hashes(0)?;
    let result = MptOwnedNodeArena::try_new(
        anchor,
        vec![encoded],
        MptResolverLimits::TEST_FIXTURE,
        128,
        &mut session,
        || {
            calls.set(calls.get().saturating_add(1));
            RealKeccak::default()
        },
    );

    assert!(matches!(
        result,
        Err(MptArenaError::Resolver(MptResolverError::Resource(
            eth_valkyoth_codec::DecodeError::HashCountExceeded
        )))
    ));
    assert_eq!(calls.get(), 0);
    assert_eq!(session.hashes(), 0);
    Ok(())
}

#[test]
fn owned_arena_accounts_source_vector_excess_capacity() -> Result<(), MptArenaError> {
    let expected = vec![0xc2, 0x20, 0x01];
    let mut oversized = Vec::with_capacity(4096);
    oversized.extend_from_slice(&expected);
    let payload_capacity = oversized.capacity();
    let hash = hash_one(RealKeccak::default(), &expected);
    let anchor = MptSnapshotAnchor::new(crate::MptProofRoot::from_b256(hash));
    let arena = MptOwnedNodeArena::try_new(
        anchor,
        vec![oversized],
        MptResolverLimits::TEST_FIXTURE,
        8192,
        &mut test_session()?,
        RealKeccak::default,
    )?;

    let retained = arena.nodes.first().ok_or(MptArenaError::Allocation)?;
    assert_eq!(retained.encoded.capacity(), payload_capacity);
    assert_eq!(
        arena.retained_bytes(),
        arena
            .nodes
            .capacity()
            .saturating_mul(core::mem::size_of::<OwnedNode>())
            .saturating_add(payload_capacity)
    );
    Ok(())
}

#[test]
fn owned_arena_rejects_excess_capacity_before_hashing() -> Result<(), MptArenaError> {
    let expected = vec![0xc2, 0x20, 0x01];
    let mut oversized = Vec::with_capacity(4096);
    oversized.extend_from_slice(&expected);
    let hash = hash_one(RealKeccak::default(), &expected);
    let anchor = MptSnapshotAnchor::new(crate::MptProofRoot::from_b256(hash));
    let calls = Cell::new(0_usize);
    let mut session = test_session()?;
    let result = MptOwnedNodeArena::try_new(
        anchor,
        vec![oversized],
        MptResolverLimits::TEST_FIXTURE,
        128,
        &mut session,
        || {
            calls.set(calls.get().saturating_add(1));
            RealKeccak::default()
        },
    );

    assert!(matches!(result, Err(MptArenaError::RetainedBytesExceeded)));
    assert_eq!(calls.get(), 0);
    assert_eq!(session.hashes(), 0);
    Ok(())
}

#[test]
fn schedule_is_bounded_deterministic_and_cancellable() -> Result<(), MptScheduleError> {
    let ranges = MptBatchSchedule::reviewed(10, 3)?.collect::<Vec<_>>();
    assert_eq!(ranges, vec![0..4, 4..8, 8..10]);

    let cancellation = Cancellation {
        checks: Cell::new(0),
        cancel_at: 1,
    };
    let visited = Cell::new(0_usize);
    let result = MptBatchSchedule::reviewed(10, 3)?.try_for_each(&cancellation, |_| {
        visited.set(visited.get().saturating_add(1));
        Ok(())
    });
    assert_eq!(result, Err(MptScheduleError::Cancelled));
    assert_eq!(visited.get(), 1);
    Ok(())
}

fn test_session() -> Result<DecodeSession, MptArenaError> {
    let limits = DecodeLimits {
        max_input_bytes: 4096,
        max_list_items: 64,
        max_nesting_depth: 16,
        max_total_allocation: 8192,
        max_proof_nodes: 64,
        max_total_items: 1024,
    };
    let policy = DecodeSessionPolicy::compatibility_policy(limits).map_err(arena_resource_error)?;
    DecodeSession::new(policy).map_err(arena_resource_error)
}

fn test_session_with_hashes(max_hashes: usize) -> Result<DecodeSession, MptArenaError> {
    let limits = DecodeLimits {
        max_input_bytes: 4096,
        max_list_items: 64,
        max_nesting_depth: 16,
        max_total_allocation: 8192,
        max_proof_nodes: 64,
        max_total_items: 1024,
    };
    let policy = DecodeSessionPolicy::reviewed_policy(
        limits, 16_384, 1024, max_hashes, 4096, 4096, 4096, 65_536,
    )
    .map_err(arena_resource_error)?;
    DecodeSession::new(policy).map_err(arena_resource_error)
}
