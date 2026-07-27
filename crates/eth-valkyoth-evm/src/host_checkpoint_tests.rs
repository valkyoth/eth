use super::*;

#[test]
fn child_revert_restores_state_and_scope_warmth() {
    request_fixture!(raw, snapshot, request);
    let address = patterned_address();
    let slot = patterned_hash(1);
    let changed = patterned_hash(2);
    let mut memory = [0u8; 64];
    let Some(mut arena) = BorrowedTransactionArena::<4>::try_new(&mut memory).ok() else {
        return;
    };
    let mut journal = TestJournal::default();
    let mut access = TestAccess::default();
    let mut crypto = TestCrypto;
    let mut inspector = TestInspector::default();

    {
        let mut host =
            ExecutionHost::new(&request, &mut journal, &mut access, &mut crypto, &mut arena);
        let started = host.begin_transaction();
        assert_eq!(started, Ok(InspectorEvent::TransactionStarted));
        inspector.observe(started.unwrap_or(InspectorEvent::TransactionStarted));
        let child = host.with_child(frame(address), |host| {
            assert_eq!(host.warm_storage(address, slot), Ok(AccessStatus::Cold));
            assert_eq!(host.set_storage(address, slot, changed), Ok(()));
            ChildDecision::Revert(())
        });
        assert!(child.is_ok(), "{child:?}");
        if let Ok(child) = child {
            for event in child.events() {
                inspector.observe(*event);
            }
        }
    }

    assert_eq!(journal.value, None);
    assert_eq!(access.warm_storage(address, slot), Ok(AccessStatus::Cold));
    assert_eq!(
        inspector.last,
        Some(InspectorEvent::ChildReverted { depth: 1 })
    );
    assert_eq!(snapshot.snapshot_id(), request.snapshot().snapshot_id());
}

#[test]
fn nested_revert_restores_only_inner_scope_warmth() {
    request_fixture!(raw, snapshot, request);
    let outer_address = patterned_address();
    let inner_address = zero_address();
    let mut memory = [0u8; 16];
    let Some(mut arena) = BorrowedTransactionArena::<2>::try_new(&mut memory).ok() else {
        return;
    };
    let mut journal = TestJournal::default();
    let mut access = TestAccess::default();
    let mut crypto = TestCrypto;
    let mut host = ExecutionHost::new(&request, &mut journal, &mut access, &mut crypto, &mut arena);
    assert!(host.begin_transaction().is_ok());

    let outer = host.with_child(frame(outer_address), |host| {
        assert_eq!(host.warm_address(outer_address), Ok(AccessStatus::Cold));
        let inner = host.with_child(frame(inner_address), |host| {
            assert_eq!(host.warm_address(inner_address), Ok(AccessStatus::Cold));
            ChildDecision::Revert(())
        });
        assert!(inner.is_ok(), "{inner:?}");
        assert_eq!(host.warm_address(outer_address), Ok(AccessStatus::Warm));
        assert_eq!(host.warm_address(inner_address), Ok(AccessStatus::Cold));
        ChildDecision::Commit(())
    });
    assert!(outer.is_ok(), "{outer:?}");
}

#[test]
fn access_finalization_failure_poisons_partially_finalized_host() {
    request_fixture!(raw, snapshot, request);
    let mut memory = [0u8; 8];
    let Some(mut arena) = BorrowedTransactionArena::<1>::try_new(&mut memory).ok() else {
        return;
    };
    let mut journal = TestJournal::default();
    let mut access = TestAccess {
        fail_commit: true,
        ..TestAccess::default()
    };
    let mut crypto = TestCrypto;
    let mut host = ExecutionHost::new(&request, &mut journal, &mut access, &mut crypto, &mut arena);
    assert!(host.begin_transaction().is_ok());

    let result = host.with_child(frame(zero_address()), |_| ChildDecision::Commit(()));
    assert_eq!(
        result,
        Err(ChildLifecycleError::CapabilityConsistencyUnknown {
            action: ChildFinalizeAction::Commit,
            journal_error: None,
            access_error: Some(HostCapabilityError::AccessTrackingFailed),
        })
    );
    assert!(host.is_poisoned());
    assert_eq!(host.frame_depth(), 1);
}

#[test]
fn access_checkpoint_error_poisons_host_after_journal_cleanup() {
    request_fixture!(raw, snapshot, request);
    let mut memory = [0u8; 8];
    let Some(mut arena) = BorrowedTransactionArena::<1>::try_new(&mut memory).ok() else {
        return;
    };
    let mut journal = TestJournal::default();
    let mut access = TestAccess {
        fail_checkpoint: true,
        ..TestAccess::default()
    };
    let mut crypto = TestCrypto;
    let mut host = ExecutionHost::new(&request, &mut journal, &mut access, &mut crypto, &mut arena);
    assert!(host.begin_transaction().is_ok());

    let result = host.with_child(frame(zero_address()), |_| ChildDecision::Commit(()));
    assert_eq!(
        result,
        Err(ChildLifecycleError::Begin(
            BeginChildError::AccessCheckpointFailed {
                error: HostCapabilityError::AccessTrackingFailed,
                journal_revert_error: None,
            }
        ))
    );
    assert!(host.is_poisoned());
    assert_eq!(host.frame_depth(), 0);
}
