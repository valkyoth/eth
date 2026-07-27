use crate::{
    CumulativeExecutionResource, ExecutionGovernor, ExecutionGovernorError, ExecutionResource,
    ExecutionResourceLimits, ExecutionResourceRequest, HighWaterExecutionResource,
    MAX_EXECUTION_WORK_UNITS,
};

fn limits() -> Option<ExecutionResourceLimits> {
    let request = ExecutionResourceRequest {
        warm_addresses: 2,
        warm_storage_slots: 3,
        journal_entries: 4,
        journal_checkpoints: 2,
        frames: 2,
        memory_bytes: 64,
        reusable_arena_bytes: 128,
        cache_entries: 8,
        work_units: 20,
    };
    ExecutionResourceLimits::try_new(request).ok()
}

#[test]
fn every_resource_is_bounded_and_failure_is_atomic() {
    let Some(limits) = limits() else {
        return;
    };
    let mut governor = ExecutionGovernor::new(limits);
    assert_eq!(governor.reset_transaction(), Ok(()));
    let cumulative = [
        (
            CumulativeExecutionResource::WarmAddress,
            ExecutionResource::WarmAddress,
            2,
        ),
        (
            CumulativeExecutionResource::WarmStorageSlot,
            ExecutionResource::WarmStorageSlot,
            3,
        ),
        (
            CumulativeExecutionResource::JournalEntry,
            ExecutionResource::JournalEntry,
            4,
        ),
    ];

    for (resource, reported, limit) in cumulative {
        assert_eq!(governor.charge(resource, limit), Ok(()));
        assert_eq!(
            governor.charge(resource, 1),
            Err(ExecutionGovernorError::ResourceExhausted(reported))
        );
        assert_eq!(governor.used(reported), limit);
    }

    let high_water = [
        (
            HighWaterExecutionResource::JournalCheckpoint,
            ExecutionResource::JournalCheckpoint,
            2,
        ),
        (
            HighWaterExecutionResource::Frame,
            ExecutionResource::Frame,
            2,
        ),
        (
            HighWaterExecutionResource::MemoryByte,
            ExecutionResource::MemoryByte,
            64,
        ),
        (
            HighWaterExecutionResource::ReusableArenaByte,
            ExecutionResource::ReusableArenaByte,
            128,
        ),
        (
            HighWaterExecutionResource::CacheEntry,
            ExecutionResource::CacheEntry,
            8,
        ),
    ];
    for (resource, reported, limit) in high_water {
        assert_eq!(governor.observe_capacity(resource, limit), Ok(()));
        assert_eq!(
            governor.observe_capacity(resource, limit.saturating_add(1)),
            Err(ExecutionGovernorError::ResourceExhausted(reported))
        );
        assert_eq!(governor.used(reported), limit);
    }
}

#[test]
fn work_delegation_conserves_authority_and_cancelled_work_stays_charged() {
    let Some(limits) = limits() else {
        return;
    };
    let mut governor = ExecutionGovernor::new(limits);
    assert_eq!(governor.reset_transaction(), Ok(()));
    let Ok(mut root) = governor.issue_work(12) else {
        return;
    };
    let child_generation = {
        let Ok(child) = root.delegate(5) else {
            return;
        };
        assert_eq!(child.units(), 5);
        child.generation()
    };

    assert_eq!(root.units(), 7);
    assert_eq!(root.generation(), child_generation);
    assert_eq!(governor.work_remaining(), 8);
    assert_eq!(
        root.delegate(8),
        Err(ExecutionGovernorError::InvalidDelegation)
    );
    assert_eq!(root.units(), 7);
}

#[test]
fn reusable_capacity_records_high_water_instead_of_cumulative_calls() {
    let Some(limits) = limits() else {
        return;
    };
    let mut governor = ExecutionGovernor::new(limits);
    assert_eq!(governor.reset_transaction(), Ok(()));
    assert_eq!(
        governor.observe_capacity(HighWaterExecutionResource::Frame, 2),
        Ok(())
    );
    assert_eq!(
        governor.observe_capacity(HighWaterExecutionResource::Frame, 1),
        Ok(())
    );
    assert_eq!(governor.used(ExecutionResource::Frame), 2);
    assert_eq!(
        governor.observe_capacity(HighWaterExecutionResource::Frame, 3),
        Err(ExecutionGovernorError::ResourceExhausted(
            ExecutionResource::Frame
        ))
    );
    assert_eq!(governor.used(ExecutionResource::Frame), 2);
}

#[test]
fn destructive_reset_clears_usage_and_invalidates_generation_identity() {
    let Some(limits) = limits() else {
        return;
    };
    let mut governor = ExecutionGovernor::new(limits);
    assert_eq!(governor.reset_transaction(), Ok(()));
    assert_eq!(
        governor.observe_capacity(HighWaterExecutionResource::CacheEntry, 7),
        Ok(())
    );
    let Ok(before) = governor.issue_work(4) else {
        return;
    };

    assert_eq!(governor.reset_transaction(), Ok(()));
    let Ok(after) = governor.issue_work(4) else {
        return;
    };
    assert_eq!(governor.used(ExecutionResource::CacheEntry), 0);
    assert_eq!(governor.work_remaining(), 16);
    assert_ne!(before.generation(), after.generation());
}

#[test]
fn invalid_limits_and_zero_charges_fail_closed() {
    let Some(limits) = limits() else {
        return;
    };
    let mut request = limits.request();
    request.frames = 0;
    assert_eq!(
        ExecutionResourceLimits::try_new(request),
        Err(ExecutionGovernorError::ZeroLimit)
    );

    let mut governor = ExecutionGovernor::new(limits);
    assert_eq!(
        governor.observe_capacity(HighWaterExecutionResource::Frame, 0),
        Err(ExecutionGovernorError::ZeroCharge)
    );
    assert_eq!(
        governor.observe_capacity(HighWaterExecutionResource::Frame, 1),
        Err(ExecutionGovernorError::TransactionNotStarted)
    );
    assert_eq!(
        governor.issue_work(0),
        Err(ExecutionGovernorError::ZeroCharge)
    );
    assert_eq!(
        governor.issue_work(1),
        Err(ExecutionGovernorError::TransactionNotStarted)
    );

    request.frames = 1;
    request.work_units = MAX_EXECUTION_WORK_UNITS.saturating_add(1);
    assert_eq!(
        ExecutionResourceLimits::try_new(request),
        Err(ExecutionGovernorError::LimitTooLarge)
    );
}
