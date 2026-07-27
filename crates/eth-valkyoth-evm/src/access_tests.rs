use eth_valkyoth_primitives::{Address, B256};

use crate::{AccessStatus, AccessTracker, EmbeddedAccessTracker, HostCapabilityError};

#[test]
fn embedded_core_tracker_satisfies_host_access_contract() {
    let Ok(mut tracker) = EmbeddedAccessTracker::<2, 2>::try_new() else {
        return;
    };
    let address = Address::from_bytes([1; 20]);
    let slot = B256::from_bytes([2; 32]);

    assert_eq!(
        AccessTracker::warm_address(&mut tracker, address),
        Ok(AccessStatus::Cold)
    );
    assert_eq!(
        AccessTracker::warm_address(&mut tracker, address),
        Ok(AccessStatus::Warm)
    );
    assert_eq!(
        AccessTracker::warm_storage(&mut tracker, address, slot),
        Ok(AccessStatus::Cold)
    );
    assert_eq!(AccessTracker::reset_transaction(&mut tracker), Ok(()));
    assert_eq!(
        AccessTracker::warm_storage(&mut tracker, address, slot),
        Ok(AccessStatus::Cold)
    );
}

#[test]
fn embedded_adapter_maps_capacity_failure_without_partial_slot_insert() {
    let Ok(mut tracker) = EmbeddedAccessTracker::<1, 1>::try_new() else {
        return;
    };
    let first = Address::from_bytes([1; 20]);
    let second = Address::from_bytes([2; 20]);
    let slot = B256::from_bytes([3; 32]);

    assert_eq!(
        AccessTracker::warm_address(&mut tracker, first),
        Ok(AccessStatus::Cold)
    );
    assert_eq!(
        AccessTracker::warm_storage(&mut tracker, second, slot),
        Err(HostCapabilityError::AccessTrackingFailed)
    );
    assert_eq!(tracker.storage_len(), 0);
}

#[cfg(feature = "alloc")]
#[test]
fn node_core_tracker_satisfies_host_access_contract() {
    let Ok(mut tracker) = crate::NodeAccessTracker::try_new(2, 2) else {
        return;
    };
    let address = Address::from_bytes([4; 20]);
    let slot = B256::from_bytes([5; 32]);

    assert_eq!(
        AccessTracker::warm_storage(&mut tracker, address, slot),
        Ok(AccessStatus::Cold)
    );
    assert_eq!(
        AccessTracker::warm_storage(&mut tracker, address, slot),
        Ok(AccessStatus::Warm)
    );
}
