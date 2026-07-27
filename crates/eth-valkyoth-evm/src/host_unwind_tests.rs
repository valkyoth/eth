use std::{
    boxed::Box,
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
};

use super::*;

#[test]
fn child_unwind_poisons_host_and_blocks_mutating_capabilities() {
    request_fixture!(raw, snapshot, request);
    let mut memory = [u8::default(); 8];
    let Some(mut arena) = BorrowedTransactionArena::<1>::try_new(&mut memory).ok() else {
        return;
    };
    let mut journal = TestJournal::default();
    let mut access = TestAccess::default();
    let mut crypto = TestCrypto;
    let mut host = ExecutionHost::new(&request, &mut journal, &mut access, &mut crypto, &mut arena);
    assert!(host.begin_transaction().is_ok());

    let unwind = catch_unwind(AssertUnwindSafe(|| {
        let _: Result<crate::ChildExecution<()>, ChildLifecycleError> =
            host.with_child(frame(zero_address()), |host| {
                assert_eq!(
                    host.set_storage(zero_address(), zero_hash(), patterned_hash(3)),
                    Ok(())
                );
                resume_unwind(Box::new("child execution unwind"));
            });
    }));

    assert!(unwind.is_err());
    assert!(host.is_poisoned());
    assert_eq!(host.frame_depth(), 1);
    assert_eq!(
        host.set_storage(zero_address(), zero_hash(), patterned_hash(4)),
        Err(HostCapabilityError::HostPoisoned)
    );
    assert_eq!(
        host.warm_address(zero_address()),
        Err(HostCapabilityError::HostPoisoned)
    );
    assert_eq!(
        host.keccak256(b"unwind"),
        Err(HostCapabilityError::HostPoisoned)
    );
    assert_eq!(
        host.reserve_memory(1),
        Err(HostCapabilityError::HostPoisoned)
    );
    assert_eq!(
        host.begin_transaction(),
        Err(HostCapabilityError::HostPoisoned)
    );
}
