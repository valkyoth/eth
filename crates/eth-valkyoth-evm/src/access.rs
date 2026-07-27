use eth_valkyoth_evm_core::{EvmAccessStatus, EvmAddress, EvmEmbeddedAccessTracker, EvmWord};
use eth_valkyoth_primitives::{Address, B256};

use crate::{AccessStatus, AccessTracker, HostCapabilityError};

/// Allocation-free fixed-array access tracker for explicit embedded profiles.
pub type EmbeddedAccessTracker<const ADDRESSES: usize, const STORAGE: usize> =
    EvmEmbeddedAccessTracker<ADDRESSES, STORAGE>;

#[cfg(feature = "alloc")]
/// Pre-reserved logarithmic access tracker for node-scale execution.
pub type NodeAccessTracker = eth_valkyoth_evm_core::EvmNodeAccessTracker;

fn map_status(status: EvmAccessStatus) -> AccessStatus {
    match status {
        EvmAccessStatus::Warm => AccessStatus::Warm,
        EvmAccessStatus::Cold => AccessStatus::Cold,
    }
}

fn map_address(address: Address) -> EvmAddress {
    EvmAddress::from_bytes(address.to_bytes())
}

fn map_slot(slot: B256) -> EvmWord {
    EvmWord::from_be_bytes(slot.to_bytes())
}

impl<const ADDRESSES: usize, const STORAGE: usize> AccessTracker
    for EvmEmbeddedAccessTracker<ADDRESSES, STORAGE>
{
    fn reset_transaction(&mut self) -> Result<(), HostCapabilityError> {
        self.reset_transaction()
            .map_err(|_| HostCapabilityError::AccessTrackingFailed)
    }

    fn warm_address(&mut self, address: Address) -> Result<AccessStatus, HostCapabilityError> {
        self.warm_address(map_address(address))
            .map(map_status)
            .map_err(|_| HostCapabilityError::AccessTrackingFailed)
    }

    fn warm_storage(
        &mut self,
        address: Address,
        slot: B256,
    ) -> Result<AccessStatus, HostCapabilityError> {
        self.warm_storage(map_address(address), map_slot(slot))
            .map(map_status)
            .map_err(|_| HostCapabilityError::AccessTrackingFailed)
    }
}

#[cfg(feature = "alloc")]
impl AccessTracker for eth_valkyoth_evm_core::EvmNodeAccessTracker {
    fn reset_transaction(&mut self) -> Result<(), HostCapabilityError> {
        self.reset_transaction()
            .map_err(|_| HostCapabilityError::AccessTrackingFailed)
    }

    fn warm_address(&mut self, address: Address) -> Result<AccessStatus, HostCapabilityError> {
        self.warm_address(map_address(address))
            .map(map_status)
            .map_err(|_| HostCapabilityError::AccessTrackingFailed)
    }

    fn warm_storage(
        &mut self,
        address: Address,
        slot: B256,
    ) -> Result<AccessStatus, HostCapabilityError> {
        self.warm_storage(map_address(address), map_slot(slot))
            .map(map_status)
            .map_err(|_| HostCapabilityError::AccessTrackingFailed)
    }
}
