use crate::{
    EVM_MAX_GAS_LIMIT, EVM_MAX_WARM_ADDRESSES, EVM_MAX_WARM_STORAGE_SLOTS, EvmAccessAttempt,
    EvmAccessProfile, EvmAccessStatus, EvmAccessTracker, EvmAddress, EvmCoreError,
    EvmEmbeddedAccessTracker, EvmWord,
};

#[test]
fn embedded_profile_is_explicit_and_capacity_is_bounded() -> Result<(), EvmCoreError> {
    let mut tracker = EvmEmbeddedAccessTracker::<2, 2>::try_new()?;
    assert_eq!(tracker.profile(), EvmAccessProfile::EmbeddedLinear);
    assert_eq!(
        EvmEmbeddedAccessTracker::<0, 1>::try_new(),
        Err(EvmCoreError::StateAccessListTooSmall)
    );
    assert_eq!(
        EvmEmbeddedAccessTracker::<1, 0>::try_new(),
        Err(EvmCoreError::StateAccessListTooSmall)
    );

    assert_eq!(tracker.warm_address(address(1))?, EvmAccessStatus::Cold);
    assert_eq!(tracker.warm_address(address(1))?, EvmAccessStatus::Warm);
    Ok(())
}

#[test]
fn embedded_attempt_lifecycle_is_non_nested_and_resettable() -> Result<(), EvmCoreError> {
    let mut tracker = EvmEmbeddedAccessTracker::<4, 4>::try_new()?;
    tracker.begin_attempt()?;
    assert_eq!(
        tracker.begin_attempt(),
        Err(EvmCoreError::StateAccessAttemptAlreadyActive)
    );
    let _ = tracker.warm_storage(address(1), EvmWord::from_usize(2))?;
    tracker.finish_attempt(EvmAccessAttempt::Rollback)?;
    assert_eq!(tracker.address_len(), 0);
    assert_eq!(tracker.storage_len(), 0);
    assert_eq!(
        tracker.finish_attempt(EvmAccessAttempt::Commit),
        Err(EvmCoreError::StateAccessAttemptMissing)
    );

    tracker.begin_attempt()?;
    let _ = tracker.warm_address(address(3))?;
    tracker.finish_attempt(EvmAccessAttempt::Commit)?;
    assert_eq!(tracker.address_len(), 1);
    tracker.reset_transaction()?;
    assert_eq!(tracker.address_len(), 0);
    Ok(())
}

#[test]
fn gas_derived_hard_ceilings_are_nonzero() {
    const {
        assert!(EVM_MAX_WARM_ADDRESSES > 1);
        assert!(EVM_MAX_WARM_STORAGE_SLOTS > EVM_MAX_WARM_ADDRESSES);
    }
    assert_eq!(
        usize::try_from((EVM_MAX_GAS_LIMIT - 21_000) / 2_400 + 20),
        Ok(EVM_MAX_WARM_ADDRESSES)
    );
    assert_eq!(
        usize::try_from(EVM_MAX_GAS_LIMIT / 1_900),
        Ok(EVM_MAX_WARM_STORAGE_SLOTS)
    );
}

#[test]
fn nested_checkpoint_revert_restores_only_inner_warmth() -> Result<(), EvmCoreError> {
    let mut tracker = EvmEmbeddedAccessTracker::<4, 4>::try_new()?;
    let outer = tracker.checkpoint()?;
    assert_eq!(tracker.warm_address(address(1))?, EvmAccessStatus::Cold);
    let inner = tracker.checkpoint()?;
    assert_eq!(tracker.warm_address(address(2))?, EvmAccessStatus::Cold);

    tracker.revert(inner)?;
    assert_eq!(tracker.warm_address(address(1))?, EvmAccessStatus::Warm);
    assert_eq!(tracker.warm_address(address(2))?, EvmAccessStatus::Cold);
    tracker.commit(outer)?;
    Ok(())
}

fn address(value: u8) -> EvmAddress {
    let mut bytes = [0u8; EvmAddress::LEN];
    if let Some(last) = bytes.last_mut() {
        *last = value;
    }
    EvmAddress::from_bytes(bytes)
}
