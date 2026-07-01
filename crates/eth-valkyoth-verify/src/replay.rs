use eth_valkyoth_primitives::ChainId;
use eth_valkyoth_protocol::{
    UnvalidatedAccessListTransaction, UnvalidatedBlobTransaction, UnvalidatedDynamicFeeTransaction,
    UnvalidatedLegacyTransaction, UnvalidatedSetCodeTransaction, UnvalidatedTransaction,
};

use crate::{VerifyError, require_chain};

/// Requires a legacy transaction to carry the expected EIP-155 chain domain.
///
/// Pre-EIP-155 legacy transactions do not carry a chain-bound replay domain
/// and are rejected before sender recovery can be trusted.
pub fn require_legacy_replay_domain(
    expected: ChainId,
    transaction: &UnvalidatedLegacyTransaction<'_>,
) -> Result<(), VerifyError> {
    let actual = transaction
        .eip155_chain_id()
        .ok_or(VerifyError::MissingReplayDomain)?;
    require_chain(expected, actual)
}

/// Requires an EIP-2930 transaction to carry the expected chain domain.
pub fn require_access_list_replay_domain(
    expected: ChainId,
    transaction: &UnvalidatedAccessListTransaction<'_>,
) -> Result<(), VerifyError> {
    require_chain(expected, transaction.chain_id)
}

/// Requires an EIP-1559 transaction to carry the expected chain domain.
pub fn require_dynamic_fee_replay_domain(
    expected: ChainId,
    transaction: &UnvalidatedDynamicFeeTransaction<'_>,
) -> Result<(), VerifyError> {
    require_chain(expected, transaction.chain_id)
}

/// Requires an EIP-4844 transaction to carry the expected chain domain.
pub fn require_blob_replay_domain(
    expected: ChainId,
    transaction: &UnvalidatedBlobTransaction<'_>,
) -> Result<(), VerifyError> {
    require_chain(expected, transaction.chain_id)
}

/// Requires an EIP-7702 transaction to carry the expected chain domain.
pub fn require_set_code_replay_domain(
    expected: ChainId,
    transaction: &UnvalidatedSetCodeTransaction<'_>,
) -> Result<(), VerifyError> {
    require_chain(expected, transaction.chain_id)
}

/// Requires any decoded transaction domain to match the expected chain.
///
/// This is a replay-domain gate only. It does not perform signature recovery,
/// low-s checks, fork checks, account-state checks, gas checks, fee checks, or
/// blob/KZG validation.
pub fn require_transaction_replay_domain(
    expected: ChainId,
    transaction: UnvalidatedTransaction<'_>,
) -> Result<(), VerifyError> {
    match transaction {
        UnvalidatedTransaction::Legacy(tx) => require_legacy_replay_domain(expected, &tx),
        UnvalidatedTransaction::AccessList(tx) => require_access_list_replay_domain(expected, &tx),
        UnvalidatedTransaction::DynamicFee(tx) => require_dynamic_fee_replay_domain(expected, &tx),
        UnvalidatedTransaction::Blob(tx) => require_blob_replay_domain(expected, &tx),
        UnvalidatedTransaction::SetCode(tx) => require_set_code_replay_domain(expected, &tx),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eth_valkyoth_codec::DecodeLimits;
    use eth_valkyoth_protocol::{
        decode_access_list_transaction, decode_blob_transaction, decode_dynamic_fee_transaction,
        decode_legacy_transaction, decode_set_code_transaction,
    };

    const TEST_LIMITS: DecodeLimits = DecodeLimits {
        max_input_bytes: 256,
        max_list_items: 16,
        max_nesting_depth: 8,
        max_total_allocation: 256,
        max_proof_nodes: 4,
        max_total_items: 32,
    };

    const LEGACY_EIP155_CHAIN_1: &[u8] = &[
        0xcb, 0x01, 0x02, 0x82, 0x52, 0x08, 0x80, 0x80, 0x80, 0x25, 0x01, 0x02,
    ];
    const LEGACY_PRE_EIP155: &[u8] = &[
        0xcb, 0x01, 0x02, 0x82, 0x52, 0x08, 0x80, 0x80, 0x80, 0x1b, 0x01, 0x02,
    ];
    const ACCESS_LIST_CHAIN_1: &[u8] = &[
        0x01, 0xf8, 0x46, 0x01, 0x02, 0x03, 0x82, 0x52, 0x08, 0x80, 0x80, 0x80, 0xf8, 0x38, 0xf7,
        0x94, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
        0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0xe1, 0xa0, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,
        0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,
        0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x01, 0x01, 0x02,
    ];
    const DYNAMIC_FEE_CHAIN_1: &[u8] = &[
        0x02, 0xce, 0x01, 0x02, 0x03, 0x04, 0x82, 0x52, 0x08, 0x80, 0x05, 0x80, 0xc0, 0x01, 0x01,
        0x02,
    ];
    const BLOB_CHAIN_1: &[u8] = &[
        0x03, 0xf8, 0x45, 0x01, 0x02, 0x03, 0x04, 0x82, 0x52, 0x08, 0x94, 0x11, 0x11, 0x11, 0x11,
        0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
        0x11, 0x05, 0x80, 0xc0, 0x06, 0xe1, 0xa0, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
        0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
        0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x02,
    ];
    const SET_CODE_CHAIN_1: &[u8] = &[
        0x04, 0xe3, 0x01, 0x02, 0x03, 0x04, 0x82, 0x52, 0x08, 0x94, 0x11, 0x11, 0x11, 0x11, 0x11,
        0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
        0x05, 0x80, 0xc0, 0xc0, 0x01, 0x01, 0x02,
    ];

    #[test]
    fn legacy_replay_domain_requires_eip155_chain_binding() {
        let transaction = decode_legacy_transaction(LEGACY_EIP155_CHAIN_1, TEST_LIMITS);
        assert!(transaction.is_ok());
        if let Ok(transaction) = transaction {
            assert_eq!(
                require_legacy_replay_domain(ChainId::new(1), &transaction),
                Ok(())
            );
            assert_eq!(
                require_legacy_replay_domain(ChainId::new(5), &transaction),
                Err(VerifyError::WrongChain)
            );
        }

        let transaction = decode_legacy_transaction(LEGACY_PRE_EIP155, TEST_LIMITS);
        assert!(transaction.is_ok());
        if let Ok(transaction) = transaction {
            assert_eq!(
                require_legacy_replay_domain(ChainId::new(1), &transaction),
                Err(VerifyError::MissingReplayDomain)
            );
        }
    }

    #[test]
    fn typed_transactions_require_matching_chain_domain() {
        let access_list = decode_access_list_transaction(ACCESS_LIST_CHAIN_1, TEST_LIMITS);
        assert!(access_list.is_ok());
        if let Ok(access_list) = access_list {
            assert_eq!(
                require_access_list_replay_domain(ChainId::new(1), &access_list),
                Ok(())
            );
            assert_eq!(
                require_access_list_replay_domain(ChainId::new(5), &access_list),
                Err(VerifyError::WrongChain)
            );
        }

        let dynamic_fee = decode_dynamic_fee_transaction(DYNAMIC_FEE_CHAIN_1, TEST_LIMITS);
        assert!(dynamic_fee.is_ok());
        if let Ok(dynamic_fee) = dynamic_fee {
            assert_eq!(
                require_dynamic_fee_replay_domain(ChainId::new(1), &dynamic_fee),
                Ok(())
            );
            assert_eq!(
                require_transaction_replay_domain(
                    ChainId::new(5),
                    UnvalidatedTransaction::DynamicFee(dynamic_fee),
                ),
                Err(VerifyError::WrongChain)
            );
        }

        let blob = decode_blob_transaction(BLOB_CHAIN_1, TEST_LIMITS);
        assert!(blob.is_ok());
        if let Ok(blob) = blob {
            assert_eq!(require_blob_replay_domain(ChainId::new(1), &blob), Ok(()));
            assert_eq!(
                require_transaction_replay_domain(
                    ChainId::new(5),
                    UnvalidatedTransaction::Blob(blob)
                ),
                Err(VerifyError::WrongChain)
            );
        }

        let set_code = decode_set_code_transaction(SET_CODE_CHAIN_1, TEST_LIMITS);
        assert!(set_code.is_ok());
        if let Ok(set_code) = set_code {
            assert_eq!(
                require_set_code_replay_domain(ChainId::new(1), &set_code),
                Ok(())
            );
            assert_eq!(
                require_transaction_replay_domain(
                    ChainId::new(5),
                    UnvalidatedTransaction::SetCode(set_code),
                ),
                Err(VerifyError::WrongChain)
            );
        }
    }
}
