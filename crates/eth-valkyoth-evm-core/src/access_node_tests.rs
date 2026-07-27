use crate::{
    EVM_DEFAULT_GAS_LIMIT, EVM_DEFAULT_STEP_LIMIT, EVM_MAX_WARM_ADDRESSES, EvmAccessAttempt,
    EvmAccessProfile, EvmAccessStatus, EvmAccessTracker, EvmAccount, EvmAddress, EvmCoreError,
    EvmEmbeddedAccessTracker, EvmExecution, EvmFork, EvmNodeAccessTracker, EvmState,
    EvmStateContext, EvmWord, ExecutionLimits, ExecutionStatus,
};

#[test]
fn node_and_embedded_profiles_have_identical_warmth_semantics() -> Result<(), EvmCoreError> {
    let mut embedded = EvmEmbeddedAccessTracker::<64, 64>::try_new()?;
    let mut node = EvmNodeAccessTracker::try_new(64, 64)?;
    assert_eq!(node.profile(), EvmAccessProfile::NodeFixedWidthRadix);

    for value in sequence() {
        let address = address(value);
        assert_eq!(embedded.warm_address(address)?, node.warm_address(address)?);
        let key = EvmWord::from_usize(value % 7);
        assert_eq!(
            embedded.warm_storage(address, key)?,
            node.warm_storage(address, key)?
        );
    }
    assert_eq!(embedded.address_len(), node.address_len());
    assert_eq!(embedded.storage_len(), node.storage_len());
    Ok(())
}

#[test]
fn sorted_and_reverse_inputs_remain_within_key_width() -> Result<(), EvmCoreError> {
    const ITEMS: usize = 4_096;
    let mut ascending = EvmNodeAccessTracker::try_new(ITEMS, ITEMS)?;
    let mut descending = EvmNodeAccessTracker::try_new(ITEMS, ITEMS)?;
    for value in 0..ITEMS {
        let _ = ascending.warm_storage(address(value), EvmWord::from_usize(value))?;
        let reverse = ITEMS
            .checked_sub(value)
            .and_then(|item| item.checked_sub(1))
            .ok_or(EvmCoreError::StateAccessTrackerCorrupt)?;
        let _ = descending.warm_storage(address(reverse), EvmWord::from_usize(reverse))?;
    }

    let (ascending_addresses, ascending_storage) = ascending.max_lookup_depths()?;
    let (descending_addresses, descending_storage) = descending.max_lookup_depths()?;
    assert!(ascending_addresses <= 161);
    assert!(ascending_storage <= 417);
    assert!(descending_addresses <= 161);
    assert!(descending_storage <= 417);
    Ok(())
}

#[test]
fn node_capacity_failure_is_atomic() -> Result<(), EvmCoreError> {
    let mut tracker = EvmNodeAccessTracker::try_new(1, 1)?;
    let first = address(1);
    let second = address(2);
    assert_eq!(
        tracker.warm_storage(first, EvmWord::from_usize(1))?,
        EvmAccessStatus::Cold
    );
    assert_eq!(
        tracker.warm_storage(second, EvmWord::from_usize(2)),
        Err(EvmCoreError::StateAccessListFull)
    );
    assert_eq!(tracker.address_len(), 1);
    assert_eq!(tracker.storage_len(), 1);
    assert_eq!(
        tracker.warm_storage(first, EvmWord::from_usize(1))?,
        EvmAccessStatus::Warm
    );
    Ok(())
}

#[test]
fn rollback_restores_exact_pre_attempt_state() -> Result<(), EvmCoreError> {
    let mut tracker = EvmNodeAccessTracker::try_new(32, 32)?;
    let retained_address = address(9);
    let retained_key = EvmWord::from_usize(9);
    let _ = tracker.warm_storage(retained_address, retained_key)?;
    tracker.begin_attempt()?;
    for value in 10..30 {
        let _ = tracker.warm_storage(address(value), EvmWord::from_usize(value))?;
    }
    tracker.finish_attempt(EvmAccessAttempt::Rollback)?;

    assert_eq!(tracker.address_len(), 1);
    assert_eq!(tracker.storage_len(), 1);
    assert_eq!(
        tracker.warm_storage(retained_address, retained_key)?,
        EvmAccessStatus::Warm
    );
    assert_eq!(tracker.warm_address(address(10))?, EvmAccessStatus::Cold);
    Ok(())
}

#[test]
fn node_nested_checkpoint_revert_is_lifo() -> Result<(), EvmCoreError> {
    let mut tracker = EvmNodeAccessTracker::try_new(8, 8)?;
    let outer = tracker.checkpoint()?;
    let _ = tracker.warm_address(address(1))?;
    let inner = tracker.checkpoint()?;
    let _ = tracker.warm_storage(address(2), EvmWord::from_usize(2))?;

    tracker.revert(inner)?;
    assert_eq!(tracker.warm_address(address(1))?, EvmAccessStatus::Warm);
    assert_eq!(tracker.warm_address(address(2))?, EvmAccessStatus::Cold);
    tracker.commit(outer)?;
    Ok(())
}

#[test]
fn repeated_empty_and_single_insert_reverts_preserve_large_outer_state() -> Result<(), EvmCoreError>
{
    const RETAINED: usize = 2_048;
    const REVERTS: usize = 128;
    let mut tracker = EvmNodeAccessTracker::try_new(RETAINED.saturating_add(1), 1)?;
    for value in 0..RETAINED {
        let _ = tracker.warm_address(address(value))?;
    }

    for _ in 0..REVERTS {
        let empty = tracker.checkpoint()?;
        tracker.revert(empty)?;

        let one_insert = tracker.checkpoint()?;
        let _ = tracker.warm_address(address(RETAINED))?;
        tracker.revert(one_insert)?;
    }

    assert_eq!(tracker.address_len(), RETAINED);
    assert_eq!(
        tracker.warm_address(address(RETAINED))?,
        EvmAccessStatus::Cold
    );
    Ok(())
}

#[test]
fn reset_retains_only_the_bounded_initial_allocation() -> Result<(), EvmCoreError> {
    let mut tracker = EvmNodeAccessTracker::try_new(32, 64)?;
    let initial = tracker.allocation_capacities();
    for value in 0..32 {
        let _ = tracker.warm_storage(address(value), EvmWord::from_usize(value))?;
    }
    tracker.reset_transaction()?;
    assert_eq!(tracker.address_len(), 0);
    assert_eq!(tracker.storage_len(), 0);
    assert_eq!(tracker.allocation_capacities(), initial);
    Ok(())
}

#[test]
fn node_constructor_rejects_unbounded_policy() {
    assert_eq!(
        EvmNodeAccessTracker::try_new(0, 1),
        Err(EvmCoreError::StateAccessListTooSmall)
    );
    assert_eq!(
        EvmNodeAccessTracker::try_new(EVM_MAX_WARM_ADDRESSES.saturating_add(1), 1),
        Err(EvmCoreError::StateAccessCapacityTooLarge)
    );
}

#[test]
fn node_tracker_executes_through_the_generic_state_boundary() -> Result<(), EvmCoreError> {
    let mut memory = [];
    let mut execution = EvmExecution::<4>::try_new(&mut memory)?;
    let mut state = NodeState;
    let mut tracker = EvmNodeAccessTracker::try_new(4, 4)?;
    let limits = ExecutionLimits::try_new(
        EVM_DEFAULT_STEP_LIMIT,
        EVM_DEFAULT_GAS_LIMIT,
        EvmFork::CANCUN,
    )?;
    let report = execution.run_with_state(
        &[0x60, 0x01, 0x31, 0x00],
        limits,
        EvmStateContext::new(EvmAddress::ZERO),
        &mut state,
        &mut tracker,
    )?;

    assert_eq!(report.status, ExecutionStatus::Stopped);
    assert_eq!(tracker.address_len(), 1);
    assert_eq!(tracker.storage_len(), 0);
    Ok(())
}

struct NodeState;

impl EvmState for NodeState {
    fn account(&mut self, _address: EvmAddress) -> Result<EvmAccount, EvmCoreError> {
        EvmAccount::try_new(true, EvmWord::from_usize(1), EvmWord::ZERO, 0)
    }

    fn storage(&mut self, _address: EvmAddress, _key: EvmWord) -> Result<EvmWord, EvmCoreError> {
        Ok(EvmWord::ZERO)
    }

    fn copy_code(
        &mut self,
        _address: EvmAddress,
        _code_offset: usize,
        _output: &mut [u8],
    ) -> Result<(), EvmCoreError> {
        Ok(())
    }
}

fn sequence() -> [usize; 16] {
    [7, 1, 9, 3, 7, 15, 2, 9, 4, 11, 0, 3, 13, 5, 8, 6]
}

fn address(value: usize) -> EvmAddress {
    let mut bytes = [0u8; EvmAddress::LEN];
    for (source, output) in value.to_be_bytes().iter().rev().zip(bytes.iter_mut().rev()) {
        *output = *source;
    }
    EvmAddress::from_bytes(bytes)
}
