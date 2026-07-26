use super::*;
use crate::test_crypto::RealKeccak;
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
        RealKeccak::default,
    )?;

    assert_eq!(arena.node_count(), 1);
    assert_eq!(arena.retained_bytes(), node.len());
    Ok(())
}

#[test]
fn owned_arena_rejects_retained_byte_overflow() {
    let anchor =
        MptSnapshotAnchor::new(crate::MptProofRoot::from_b256(B256::from_bytes([0_u8; 32])));
    let result = MptOwnedNodeArena::try_new(
        anchor,
        vec![vec![0_u8; 16]],
        MptResolverLimits::TEST_FIXTURE,
        8,
        RealKeccak::default,
    );
    assert!(matches!(result, Err(MptArenaError::RetainedBytesExceeded)));
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
