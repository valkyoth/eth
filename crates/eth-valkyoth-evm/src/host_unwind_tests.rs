use std::{
    boxed::Box,
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
};

use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum JournalUnwind {
    None,
    Checkpoint,
    Commit,
    Revert,
}

struct UnwindingJournal {
    inner: TestJournal,
    point: JournalUnwind,
}

impl UnwindingJournal {
    fn unwind_at(&self, point: JournalUnwind) {
        if self.point == point {
            resume_unwind(Box::new("journal lifecycle unwind"));
        }
    }
}

impl StateJournal for UnwindingJournal {
    type View = TestSnapshot;
    type Checkpoint = JournalCheckpoint;

    fn reset_transaction(&mut self) -> Result<(), HostCapabilityError> {
        self.inner.reset_transaction()
    }

    fn checkpoint(&mut self) -> Result<Self::Checkpoint, HostCapabilityError> {
        self.unwind_at(JournalUnwind::Checkpoint);
        self.inner.checkpoint()
    }

    fn commit(&mut self, checkpoint: Self::Checkpoint) -> Result<(), HostCapabilityError> {
        self.unwind_at(JournalUnwind::Commit);
        self.inner.commit(checkpoint)
    }

    fn revert(&mut self, checkpoint: Self::Checkpoint) -> Result<(), HostCapabilityError> {
        self.unwind_at(JournalUnwind::Revert);
        self.inner.revert(checkpoint)
    }

    fn current_storage(
        &self,
        base: &Self::View,
        address: Address,
        slot: B256,
    ) -> Result<B256, HostCapabilityError> {
        self.inner.current_storage(base, address, slot)
    }

    fn set_storage(
        &mut self,
        address: Address,
        slot: B256,
        value: B256,
    ) -> Result<(), HostCapabilityError> {
        self.inner.set_storage(address, slot, value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ArenaBehavior {
    Normal,
    EnterUnwind,
    EnterRejection,
    LeaveUnwind,
}

struct UnwindingArena {
    behavior: ArenaBehavior,
    frame: Option<IterativeCallFrame>,
}

impl TransactionArena for UnwindingArena {
    fn reset_transaction(&mut self) -> Result<(), HostCapabilityError> {
        self.frame = None;
        Ok(())
    }

    fn reserve_memory(&mut self, _required_len: usize) -> Result<(), HostCapabilityError> {
        Ok(())
    }

    fn enter_frame(&mut self, frame: IterativeCallFrame) -> Result<usize, HostCapabilityError> {
        match self.behavior {
            ArenaBehavior::EnterUnwind => resume_unwind(Box::new("arena enter unwind")),
            ArenaBehavior::EnterRejection => Err(HostCapabilityError::CallDepthExceeded),
            ArenaBehavior::Normal | ArenaBehavior::LeaveUnwind => {
                self.frame = Some(frame);
                Ok(1)
            }
        }
    }

    fn leave_frame(&mut self) -> Result<IterativeCallFrame, HostCapabilityError> {
        if self.behavior == ArenaBehavior::LeaveUnwind {
            resume_unwind(Box::new("arena leave unwind"));
        }
        self.frame
            .take()
            .ok_or(HostCapabilityError::CallFrameMissing)
    }

    fn frame_depth(&self) -> usize {
        usize::from(self.frame.is_some())
    }
}

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

#[test]
fn every_child_backend_unwind_poisons_host() {
    request_fixture!(raw, snapshot, request);
    let cases = [
        (
            JournalUnwind::Checkpoint,
            ArenaBehavior::Normal,
            ChildFinalizeAction::Commit,
        ),
        (
            JournalUnwind::None,
            ArenaBehavior::EnterUnwind,
            ChildFinalizeAction::Commit,
        ),
        (
            JournalUnwind::Revert,
            ArenaBehavior::EnterRejection,
            ChildFinalizeAction::Commit,
        ),
        (
            JournalUnwind::Commit,
            ArenaBehavior::Normal,
            ChildFinalizeAction::Commit,
        ),
        (
            JournalUnwind::Revert,
            ArenaBehavior::Normal,
            ChildFinalizeAction::Revert,
        ),
        (
            JournalUnwind::None,
            ArenaBehavior::LeaveUnwind,
            ChildFinalizeAction::Commit,
        ),
    ];

    for (journal_point, arena_behavior, action) in cases {
        let mut journal = UnwindingJournal {
            inner: TestJournal::default(),
            point: journal_point,
        };
        let mut arena = UnwindingArena {
            behavior: arena_behavior,
            frame: None,
        };
        let mut access = TestAccess::default();
        let mut crypto = TestCrypto;
        let mut host =
            ExecutionHost::new(&request, &mut journal, &mut access, &mut crypto, &mut arena);
        assert!(host.begin_transaction().is_ok());

        let unwind = catch_unwind(AssertUnwindSafe(|| {
            let _: Result<crate::ChildExecution<()>, ChildLifecycleError> =
                host.with_child(frame(zero_address()), |_| match action {
                    ChildFinalizeAction::Commit => ChildDecision::Commit(()),
                    ChildFinalizeAction::Revert => ChildDecision::Revert(()),
                });
        }));

        assert!(
            unwind.is_err(),
            "expected unwind from {journal_point:?}/{arena_behavior:?}/{action:?}"
        );
        assert!(host.is_poisoned());
        assert_eq!(
            host.set_storage(zero_address(), zero_hash(), patterned_hash(3)),
            Err(HostCapabilityError::HostPoisoned)
        );
    }
}
