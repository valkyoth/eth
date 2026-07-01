use super::*;

const TEST_CHAIN_ID: ChainId = ChainId::new(1);
const TEST_FORKS: &[ForkSpec] = &[
    ForkSpec {
        chain_id: TEST_CHAIN_ID,
        hardfork: Hardfork::Frontier,
        activation: ForkActivation::BlockOnly {
            activation_block: BlockNumber::new(0),
        },
    },
    ForkSpec {
        chain_id: TEST_CHAIN_ID,
        hardfork: Hardfork::Shanghai,
        activation: ForkActivation::BlockAndTimestamp {
            activation_block: BlockNumber::new(10),
            activation_timestamp: UnixTimestamp::new(20),
        },
    },
    ForkSpec {
        chain_id: TEST_CHAIN_ID,
        hardfork: Hardfork::Cancun,
        activation: ForkActivation::BlockAndTimestamp {
            activation_block: BlockNumber::new(20),
            activation_timestamp: UnixTimestamp::new(30),
        },
    },
];
const TEST_CHAIN: ChainSpec<'static> = ChainSpec::new(TEST_CHAIN_ID, TEST_FORKS);

#[test]
fn hardfork_declaration_order_is_chronological() {
    let chronological = [
        Hardfork::Frontier,
        Hardfork::Homestead,
        Hardfork::Byzantium,
        Hardfork::London,
        Hardfork::Shanghai,
        Hardfork::Cancun,
        Hardfork::Prague,
        Hardfork::Amsterdam,
    ];

    assert!(
        chronological
            .windows(2)
            .all(|window| matches!(window, [previous, next] if previous < next))
    );
}

#[test]
fn activation_requires_block_and_time() {
    let context = ValidationContext {
        fork: ForkSpec {
            chain_id: ChainId::new(1),
            hardfork: Hardfork::Shanghai,
            activation: ForkActivation::BlockAndTimestamp {
                activation_block: BlockNumber::new(10),
                activation_timestamp: UnixTimestamp::new(20),
            },
        },
        block_number: BlockNumber::new(10),
        timestamp: UnixTimestamp::new(19),
    };
    assert!(!context.fork_is_active());
}

#[test]
fn block_only_activation_ignores_timestamp() {
    let context = ValidationContext {
        fork: ForkSpec {
            chain_id: ChainId::new(1),
            hardfork: Hardfork::Frontier,
            activation: ForkActivation::BlockOnly {
                activation_block: BlockNumber::new(10),
            },
        },
        block_number: BlockNumber::new(10),
        timestamp: UnixTimestamp::new(0),
    };
    assert!(context.fork_is_active());
}

#[test]
fn chain_spec_builds_context_only_for_active_fork() {
    assert_eq!(
        TEST_CHAIN.validation_context(
            Hardfork::Cancun,
            BlockNumber::new(20),
            UnixTimestamp::new(29),
        ),
        Err(ForkError::Inactive)
    );

    let context = TEST_CHAIN.validation_context(
        Hardfork::Cancun,
        BlockNumber::new(20),
        UnixTimestamp::new(30),
    );
    assert!(matches!(context, Ok(context) if context.fork.hardfork == Hardfork::Cancun));
}

#[test]
fn active_fork_returns_highest_active_hardfork() {
    let fork = TEST_CHAIN.active_fork(BlockNumber::new(20), UnixTimestamp::new(30));
    assert!(matches!(fork, Ok(fork) if fork.hardfork == Hardfork::Cancun));
}

#[test]
fn active_fork_rejects_non_monotonic_specs() {
    let forks = [
        ForkSpec {
            chain_id: TEST_CHAIN_ID,
            hardfork: Hardfork::Cancun,
            activation: ForkActivation::BlockOnly {
                activation_block: BlockNumber::new(20),
            },
        },
        ForkSpec {
            chain_id: TEST_CHAIN_ID,
            hardfork: Hardfork::London,
            activation: ForkActivation::BlockOnly {
                activation_block: BlockNumber::new(10),
            },
        },
    ];
    let spec = ChainSpec::new(TEST_CHAIN_ID, &forks);

    assert_eq!(
        spec.active_fork(BlockNumber::new(30), UnixTimestamp::new(30)),
        Err(ForkError::NonMonotonicForkOrder)
    );
}

#[test]
fn fork_spec_rejects_duplicate_hardfork_entries() {
    let forks = [
        ForkSpec {
            chain_id: TEST_CHAIN_ID,
            hardfork: Hardfork::London,
            activation: ForkActivation::BlockOnly {
                activation_block: BlockNumber::new(10),
            },
        },
        ForkSpec {
            chain_id: TEST_CHAIN_ID,
            hardfork: Hardfork::London,
            activation: ForkActivation::BlockOnly {
                activation_block: BlockNumber::new(20),
            },
        },
    ];
    let spec = ChainSpec::new(TEST_CHAIN_ID, &forks);

    assert_eq!(
        spec.fork_spec(Hardfork::London),
        Err(ForkError::DuplicateFork)
    );
}

#[test]
fn try_new_rejects_malformed_specs_eagerly() {
    let forks = [
        ForkSpec {
            chain_id: TEST_CHAIN_ID,
            hardfork: Hardfork::Shanghai,
            activation: ForkActivation::BlockOnly {
                activation_block: BlockNumber::new(20),
            },
        },
        ForkSpec {
            chain_id: TEST_CHAIN_ID,
            hardfork: Hardfork::London,
            activation: ForkActivation::BlockOnly {
                activation_block: BlockNumber::new(10),
            },
        },
    ];

    assert_eq!(
        ChainSpec::try_new(TEST_CHAIN_ID, &forks),
        Err(ForkError::NonMonotonicForkOrder)
    );
}

#[test]
fn try_new_rejects_non_monotonic_activation_thresholds() {
    let forks = [
        ForkSpec {
            chain_id: TEST_CHAIN_ID,
            hardfork: Hardfork::London,
            activation: ForkActivation::BlockOnly {
                activation_block: BlockNumber::new(20),
            },
        },
        ForkSpec {
            chain_id: TEST_CHAIN_ID,
            hardfork: Hardfork::Shanghai,
            activation: ForkActivation::BlockOnly {
                activation_block: BlockNumber::new(10),
            },
        },
    ];

    assert_eq!(
        ChainSpec::try_new(TEST_CHAIN_ID, &forks),
        Err(ForkError::NonMonotonicForkOrder)
    );
}

#[test]
fn timestamp_based_fork_can_be_followed_by_block_only_fork() {
    let forks = [
        ForkSpec {
            chain_id: TEST_CHAIN_ID,
            hardfork: Hardfork::Shanghai,
            activation: ForkActivation::BlockAndTimestamp {
                activation_block: BlockNumber::new(10),
                activation_timestamp: UnixTimestamp::new(20),
            },
        },
        ForkSpec {
            chain_id: TEST_CHAIN_ID,
            hardfork: Hardfork::Cancun,
            activation: ForkActivation::BlockOnly {
                activation_block: BlockNumber::new(30),
            },
        },
    ];

    assert!(ChainSpec::try_new(TEST_CHAIN_ID, &forks).is_ok());
}

#[test]
fn unsupported_fork_is_explicit() {
    assert_eq!(
        TEST_CHAIN.fork_spec(Hardfork::Amsterdam),
        Err(ForkError::Unsupported)
    );
}

#[test]
fn chain_mismatch_is_rejected() {
    let forks = [ForkSpec {
        chain_id: ChainId::new(5),
        hardfork: Hardfork::Frontier,
        activation: ForkActivation::BlockOnly {
            activation_block: BlockNumber::new(0),
        },
    }];
    let spec = ChainSpec::new(TEST_CHAIN_ID, &forks);

    assert_eq!(
        spec.fork_spec(Hardfork::Frontier),
        Err(ForkError::ChainMismatch)
    );
}
