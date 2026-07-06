use crate::{EvmCoreError, EvmFork, EvmGas, EvmGasSchedule, EvmOpcode};

#[test]
fn pre_tangerine_state_reads_use_frontier_prices() -> Result<(), EvmCoreError> {
    let frontier = EvmGasSchedule::for_fork(EvmFork::FRONTIER)?;
    let homestead = EvmGasSchedule::for_fork(EvmFork::HOMESTEAD)?;

    for schedule in [frontier, homestead] {
        assert_eq!(
            schedule.account_access_cost(EvmOpcode::BALANCE, false)?,
            EvmGas::new(20)
        );
        assert_eq!(
            schedule.account_access_cost(EvmOpcode::EXTCODESIZE, false)?,
            EvmGas::new(20)
        );
        assert_eq!(
            schedule.account_access_cost(EvmOpcode::EXTCODECOPY, false)?,
            EvmGas::new(20)
        );
        assert_eq!(schedule.storage_access_cost(false), EvmGas::new(50));
        assert!(!schedule.tracks_warm_cold_state_access());
    }
    Ok(())
}

#[test]
fn tangerine_through_petersburg_use_io_repriced_state_reads() -> Result<(), EvmCoreError> {
    for fork in [
        EvmFork::TANGERINE_WHISTLE,
        EvmFork::SPURIOUS_DRAGON,
        EvmFork::BYZANTIUM,
        EvmFork::CONSTANTINOPLE,
        EvmFork::PETERSBURG,
    ] {
        let schedule = EvmGasSchedule::for_fork(fork)?;
        assert_eq!(
            schedule.account_access_cost(EvmOpcode::BALANCE, false)?,
            EvmGas::new(400)
        );
        assert_eq!(
            schedule.account_access_cost(EvmOpcode::EXTCODESIZE, false)?,
            EvmGas::new(700)
        );
        assert_eq!(
            schedule.account_access_cost(EvmOpcode::EXTCODECOPY, false)?,
            EvmGas::new(700)
        );
        assert_eq!(schedule.storage_access_cost(false), EvmGas::new(200));
        assert!(!schedule.tracks_warm_cold_state_access());
    }
    Ok(())
}

#[test]
fn extcodehash_and_istanbul_repricing_have_separate_boundaries() -> Result<(), EvmCoreError> {
    let constantinople = EvmGasSchedule::for_fork(EvmFork::CONSTANTINOPLE)?;
    let istanbul = EvmGasSchedule::for_fork(EvmFork::ISTANBUL)?;

    assert_eq!(
        constantinople.account_access_cost(EvmOpcode::EXTCODEHASH, false)?,
        EvmGas::new(400)
    );
    assert_eq!(
        istanbul.account_access_cost(EvmOpcode::BALANCE, false)?,
        EvmGas::new(700)
    );
    assert_eq!(
        istanbul.account_access_cost(EvmOpcode::EXTCODEHASH, false)?,
        EvmGas::new(700)
    );
    assert_eq!(istanbul.storage_access_cost(false), EvmGas::new(800));
    assert_eq!(istanbul.selfbalance_cost(), EvmGas::new(5));
    assert!(!istanbul.tracks_warm_cold_state_access());
    Ok(())
}

#[test]
fn berlin_introduces_warm_cold_state_access_accounting() -> Result<(), EvmCoreError> {
    let berlin = EvmGasSchedule::for_fork(EvmFork::BERLIN)?;

    assert!(berlin.tracks_warm_cold_state_access());
    assert_eq!(
        berlin.account_access_cost(EvmOpcode::BALANCE, false)?,
        EvmGas::new(2_600)
    );
    assert_eq!(
        berlin.account_access_cost(EvmOpcode::EXTCODECOPY, true)?,
        EvmGas::new(100)
    );
    assert_eq!(berlin.storage_access_cost(false), EvmGas::new(2_100));
    assert_eq!(berlin.storage_access_cost(true), EvmGas::new(100));
    Ok(())
}

#[test]
fn account_access_cost_rejects_non_account_opcodes() -> Result<(), EvmCoreError> {
    let frontier = EvmGasSchedule::for_fork(EvmFork::FRONTIER)?;
    let berlin = EvmGasSchedule::for_fork(EvmFork::BERLIN)?;

    assert_eq!(
        frontier.account_access_cost(EvmOpcode::SLOAD, false),
        Err(EvmCoreError::UnsupportedOpcode)
    );
    assert_eq!(
        berlin.account_access_cost(EvmOpcode::SLOAD, true),
        Err(EvmCoreError::UnsupportedOpcode)
    );
    Ok(())
}
