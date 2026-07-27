use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AccessUnwind {
    None,
    Reset,
    WarmAddress,
    WarmStorage,
}

struct UnwindingAccess {
    inner: TestAccess,
    point: AccessUnwind,
    reset_calls: usize,
}

impl AccessTracker for UnwindingAccess {
    fn reset_transaction(&mut self) -> Result<(), HostCapabilityError> {
        self.reset_calls = self.reset_calls.saturating_add(1);
        if self.point == AccessUnwind::Reset && self.reset_calls > 1 {
            resume_unwind(Box::new("access reset unwind"));
        }
        self.inner.reset_transaction()
    }

    fn warm_address(&mut self, address: Address) -> Result<AccessStatus, HostCapabilityError> {
        if self.point == AccessUnwind::WarmAddress {
            resume_unwind(Box::new("warm address unwind"));
        }
        self.inner.warm_address(address)
    }

    fn warm_storage(
        &mut self,
        address: Address,
        slot: B256,
    ) -> Result<AccessStatus, HostCapabilityError> {
        if self.point == AccessUnwind::WarmStorage {
            resume_unwind(Box::new("warm storage unwind"));
        }
        self.inner.warm_storage(address, slot)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CryptoUnwind {
    None,
    Keccak,
    Recover,
}

struct UnwindingCrypto {
    inner: TestCrypto,
    point: CryptoUnwind,
}

impl CryptoProvider for UnwindingCrypto {
    fn keccak256(&mut self, input: &[u8]) -> Result<B256, HostCapabilityError> {
        if self.point == CryptoUnwind::Keccak {
            resume_unwind(Box::new("keccak unwind"));
        }
        self.inner.keccak256(input)
    }

    fn recover_address(
        &mut self,
        digest: B256,
        signature: &[u8; 65],
    ) -> Result<Address, HostCapabilityError> {
        if self.point == CryptoUnwind::Recover {
            resume_unwind(Box::new("recovery unwind"));
        }
        self.inner.recover_address(digest, signature)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ResetBackend {
    Journal,
    Access,
    Arena,
}

#[test]
fn every_transaction_reset_unwind_poisons_and_stops_host() {
    request_fixture!(raw, snapshot, request);
    for backend in [
        ResetBackend::Journal,
        ResetBackend::Access,
        ResetBackend::Arena,
    ] {
        let mut journal = UnwindingJournal {
            inner: TestJournal::default(),
            point: if backend == ResetBackend::Journal {
                JournalUnwind::Reset
            } else {
                JournalUnwind::None
            },
            reset_calls: 0,
        };
        let mut access = UnwindingAccess {
            inner: TestAccess::default(),
            point: if backend == ResetBackend::Access {
                AccessUnwind::Reset
            } else {
                AccessUnwind::None
            },
            reset_calls: 0,
        };
        let mut crypto = UnwindingCrypto {
            inner: TestCrypto,
            point: CryptoUnwind::None,
        };
        let mut arena = UnwindingArena {
            behavior: if backend == ResetBackend::Arena {
                ArenaBehavior::ResetUnwind
            } else {
                ArenaBehavior::Normal
            },
            frame: None,
            reset_calls: 0,
        };
        let mut host =
            ExecutionHost::new(&request, &mut journal, &mut access, &mut crypto, &mut arena);
        assert!(host.begin_transaction().is_ok());

        let unwind = catch_unwind(AssertUnwindSafe(|| {
            let _ = host.begin_transaction();
        }));

        assert!(unwind.is_err(), "expected reset unwind from {backend:?}");
        assert!(host.is_poisoned());
        assert_eq!(
            host.set_storage(zero_address(), zero_hash(), patterned_hash(3)),
            Err(HostCapabilityError::HostPoisoned)
        );
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RootMutation {
    SetStorage,
    WarmAddress,
    WarmStorage,
    Keccak,
    Recover,
    ReserveMemory,
}

#[test]
fn every_root_mutation_unwind_poisons_and_stops_host() {
    request_fixture!(raw, snapshot, request);
    for mutation in [
        RootMutation::SetStorage,
        RootMutation::WarmAddress,
        RootMutation::WarmStorage,
        RootMutation::Keccak,
        RootMutation::Recover,
        RootMutation::ReserveMemory,
    ] {
        let mut journal = UnwindingJournal {
            inner: TestJournal::default(),
            point: if mutation == RootMutation::SetStorage {
                JournalUnwind::SetStorage
            } else {
                JournalUnwind::None
            },
            reset_calls: 0,
        };
        let mut access = UnwindingAccess {
            inner: TestAccess::default(),
            point: match mutation {
                RootMutation::WarmAddress => AccessUnwind::WarmAddress,
                RootMutation::WarmStorage => AccessUnwind::WarmStorage,
                _ => AccessUnwind::None,
            },
            reset_calls: 0,
        };
        let mut crypto = UnwindingCrypto {
            inner: TestCrypto,
            point: match mutation {
                RootMutation::Keccak => CryptoUnwind::Keccak,
                RootMutation::Recover => CryptoUnwind::Recover,
                _ => CryptoUnwind::None,
            },
        };
        let mut arena = UnwindingArena {
            behavior: if mutation == RootMutation::ReserveMemory {
                ArenaBehavior::ReserveUnwind
            } else {
                ArenaBehavior::Normal
            },
            frame: None,
            reset_calls: 0,
        };
        let mut host =
            ExecutionHost::new(&request, &mut journal, &mut access, &mut crypto, &mut arena);
        assert!(host.begin_transaction().is_ok());
        let signature = core::array::from_fn(|index| u8::try_from(index).unwrap_or(u8::MAX));

        let unwind = catch_unwind(AssertUnwindSafe(|| match mutation {
            RootMutation::SetStorage => {
                let _ = host.set_storage(zero_address(), zero_hash(), patterned_hash(3));
            }
            RootMutation::WarmAddress => {
                let _ = host.warm_address(zero_address());
            }
            RootMutation::WarmStorage => {
                let _ = host.warm_storage(zero_address(), zero_hash());
            }
            RootMutation::Keccak => {
                let _ = host.keccak256(b"root mutation unwind");
            }
            RootMutation::Recover => {
                let _ = host.recover_address(patterned_hash(3), &signature);
            }
            RootMutation::ReserveMemory => {
                let _ = host.reserve_memory(1);
            }
        }));

        assert!(
            unwind.is_err(),
            "expected root mutation unwind from {mutation:?}"
        );
        assert!(host.is_poisoned());
        assert_eq!(
            host.begin_transaction(),
            Err(HostCapabilityError::HostPoisoned)
        );
    }
}
