use eth_valkyoth_codec::DecodeLimits;
use eth_valkyoth_primitives::{Address, ChainId};
use eth_valkyoth_protocol::{
    SignatureYParity, UnvalidatedAccessListTransaction, UnvalidatedBlobTransaction,
    UnvalidatedDynamicFeeTransaction, UnvalidatedLegacyTransaction, UnvalidatedSetCodeTransaction,
    UnvalidatedTransaction, decode_access_list_transaction, decode_blob_transaction,
    decode_dynamic_fee_transaction, decode_legacy_transaction, decode_set_code_transaction,
};
use k256::ecdsa::SigningKey;

use super::*;
use crate::{
    set_code_transaction_signing_hash,
    test_crypto::{RealKeccak, TestSecp256k1Backend},
};

const TEST_LIMITS: DecodeLimits = DecodeLimits {
    max_input_bytes: 128,
    max_list_items: 16,
    max_nesting_depth: 8,
    max_total_allocation: 128,
    max_proof_nodes: 4,
    max_total_items: 32,
};

const LEGACY_TX: [u8; 45] = [
    0xec, 0x09, 0x85, 0x04, 0xa8, 0x17, 0xc8, 0x00, 0x82, 0x52, 0x08, 0x94, 0x35, 0x35, 0x35, 0x35,
    0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35, 0x35,
    0x88, 0x0d, 0xe0, 0xb6, 0xb3, 0xa7, 0x64, 0x00, 0x00, 0x80, 0x25, 0x01, 0x02,
];
const ACCESS_LIST_TX: [u8; 15] = [
    0x01, 0xcd, 0x01, 0x02, 0x03, 0x82, 0x52, 0x08, 0x80, 0x05, 0x80, 0xc0, 0x01, 0x01, 0x02,
];
const DYNAMIC_FEE_TX: [u8; 16] = [
    0x02, 0xce, 0x01, 0x02, 0x03, 0x04, 0x82, 0x52, 0x08, 0x80, 0x05, 0x80, 0xc0, 0x01, 0x01, 0x02,
];
const BLOB_TX: [u8; 72] = [
    0x03, 0xf8, 0x45, 0x01, 0x02, 0x03, 0x04, 0x82, 0x52, 0x08, 0x94, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x05,
    0x80, 0xc0, 0x06, 0xe1, 0xa0, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
    0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
    0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x02,
];
const SET_CODE_TX: [u8; 37] = [
    0x04, 0xe3, 0x01, 0x02, 0x03, 0x04, 0x82, 0x52, 0x08, 0x94, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x05, 0x80,
    0xc0, 0xc0, 0x01, 0x01, 0x02,
];

fn signing_key() -> Result<SigningKey, TransactionSignatureValidationError> {
    SigningKey::from_bytes(
        (&[
            0x4c, 0x08, 0x83, 0xa6, 0x91, 0x02, 0x93, 0x7d, 0x62, 0x31, 0x47, 0x1b, 0x5d, 0xbb,
            0x62, 0x04, 0xfe, 0x51, 0x29, 0x61, 0x70, 0x82, 0x79, 0x2a, 0xe4, 0x68, 0xd0, 0x1a,
            0x3f, 0x36, 0x23, 0x18,
        ])
            .into(),
    )
    .map_err(|_| TransactionSignatureValidationError::InvalidSignature)
}

fn expected_sender() -> Result<Address, TransactionSignatureValidationError> {
    let key = signing_key()?;
    let encoded = key.verifying_key().to_encoded_point(false);
    let public_key = encoded
        .as_bytes()
        .get(1..)
        .ok_or(TransactionSignatureValidationError::InvalidSignature)?;
    let digest = eth_valkyoth_hash::hash_one(RealKeccak::default(), public_key);
    let bytes = <[u8; 32]>::from(digest);
    let address = bytes
        .get(12..)
        .and_then(|value| <[u8; 20]>::try_from(value).ok())
        .ok_or(TransactionSignatureValidationError::InvalidSignature)?;
    Ok(Address::from_bytes(address))
}

fn sign_hash(
    signing_hash: TransactionSigningHash,
) -> Result<([u8; 32], [u8; 32], SignatureYParity), TransactionSignatureValidationError> {
    let key = signing_key()?;
    let (signature, recovery_id) = key
        .sign_prehash_recoverable(&<[u8; 32]>::from(signing_hash.to_b256()))
        .map_err(|_| TransactionSignatureValidationError::InvalidSignature)?;
    let bytes = signature.to_bytes();
    let r = bytes
        .get(..32)
        .and_then(|value| <[u8; 32]>::try_from(value).ok())
        .ok_or(TransactionSignatureValidationError::InvalidSignature)?;
    let s = bytes
        .get(32..)
        .and_then(|value| <[u8; 32]>::try_from(value).ok())
        .ok_or(TransactionSignatureValidationError::InvalidSignature)?;
    let y_parity = SignatureYParity::try_new(u64::from(recovery_id.to_byte()))
        .map_err(|_| TransactionSignatureValidationError::InvalidSignature)?;
    Ok((r, s, y_parity))
}

fn legacy_fixture()
-> Result<UnvalidatedLegacyTransaction<'static>, TransactionSignatureValidationError> {
    decode_legacy_transaction(&LEGACY_TX, TEST_LIMITS)
        .map_err(|_| TransactionSignatureValidationError::InvalidSignature)
}

fn access_list_fixture()
-> Result<UnvalidatedAccessListTransaction<'static>, TransactionSignatureValidationError> {
    decode_access_list_transaction(&ACCESS_LIST_TX, TEST_LIMITS)
        .map_err(|_| TransactionSignatureValidationError::InvalidSignature)
}

fn dynamic_fee_fixture()
-> Result<UnvalidatedDynamicFeeTransaction<'static>, TransactionSignatureValidationError> {
    decode_dynamic_fee_transaction(&DYNAMIC_FEE_TX, TEST_LIMITS)
        .map_err(|_| TransactionSignatureValidationError::InvalidSignature)
}

fn blob_fixture() -> Result<UnvalidatedBlobTransaction<'static>, TransactionSignatureValidationError>
{
    decode_blob_transaction(&BLOB_TX, TEST_LIMITS)
        .map_err(|_| TransactionSignatureValidationError::InvalidSignature)
}

fn set_code_fixture()
-> Result<UnvalidatedSetCodeTransaction<'static>, TransactionSignatureValidationError> {
    decode_set_code_transaction(&SET_CODE_TX, TEST_LIMITS)
        .map_err(|_| TransactionSignatureValidationError::InvalidSignature)
}

fn signed_legacy()
-> Result<UnvalidatedLegacyTransaction<'static>, TransactionSignatureValidationError> {
    let tx = legacy_fixture()?;
    let mut scratch = [0_u8; 128];
    let signing_hash =
        legacy_eip155_transaction_signing_hash(&tx, &mut scratch, RealKeccak::default())
            .map_err(TransactionSignatureValidationError::SigningHash)?;
    let (r, s, y_parity) = sign_hash(signing_hash)?;
    let mut v = [0_u8; 32];
    let v_value = 35_u64
        .checked_add(2)
        .and_then(|value| value.checked_add(u64::from(y_parity.get())))
        .ok_or(TransactionSignatureValidationError::InvalidSignature)?;
    if let Some(target) = v.get_mut(24..) {
        target.copy_from_slice(&v_value.to_be_bytes());
    } else {
        return Err(TransactionSignatureValidationError::InvalidSignature);
    }
    Ok(UnvalidatedLegacyTransaction { v, r, s, ..tx })
}

fn signed_access_list()
-> Result<UnvalidatedAccessListTransaction<'static>, TransactionSignatureValidationError> {
    let tx = access_list_fixture()?;
    let mut scratch = [0_u8; 128];
    let signing_hash =
        access_list_transaction_signing_hash(&tx, &mut scratch, RealKeccak::default())
            .map_err(TransactionSignatureValidationError::SigningHash)?;
    let (r, s, y_parity) = sign_hash(signing_hash)?;
    Ok(UnvalidatedAccessListTransaction {
        y_parity,
        r,
        s,
        ..tx
    })
}

fn signed_dynamic_fee()
-> Result<UnvalidatedDynamicFeeTransaction<'static>, TransactionSignatureValidationError> {
    let tx = dynamic_fee_fixture()?;
    let mut scratch = [0_u8; 128];
    let signing_hash =
        dynamic_fee_transaction_signing_hash(&tx, &mut scratch, RealKeccak::default())
            .map_err(TransactionSignatureValidationError::SigningHash)?;
    let (r, s, y_parity) = sign_hash(signing_hash)?;
    Ok(UnvalidatedDynamicFeeTransaction {
        y_parity,
        r,
        s,
        ..tx
    })
}

fn signed_blob() -> Result<UnvalidatedBlobTransaction<'static>, TransactionSignatureValidationError>
{
    let tx = blob_fixture()?;
    let mut scratch = [0_u8; 128];
    let signing_hash = blob_transaction_signing_hash(&tx, &mut scratch, RealKeccak::default())
        .map_err(TransactionSignatureValidationError::SigningHash)?;
    let (r, s, y_parity) = sign_hash(signing_hash)?;
    Ok(UnvalidatedBlobTransaction {
        y_parity,
        r,
        s,
        ..tx
    })
}

fn signed_set_code()
-> Result<UnvalidatedSetCodeTransaction<'static>, TransactionSignatureValidationError> {
    let tx = set_code_fixture()?;
    let mut scratch = [0_u8; 128];
    let signing_hash = set_code_transaction_signing_hash(&tx, &mut scratch, RealKeccak::default())
        .map_err(TransactionSignatureValidationError::SigningHash)?;
    let (r, s, y_parity) = sign_hash(signing_hash)?;
    Ok(UnvalidatedSetCodeTransaction {
        y_parity,
        r,
        s,
        ..tx
    })
}

#[test]
fn validates_supported_transaction_signatures() {
    let expected = expected_sender();
    let legacy = signed_legacy();
    let access_list = signed_access_list();
    let dynamic_fee = signed_dynamic_fee();
    let blob = signed_blob();
    let set_code = signed_set_code();
    assert!(expected.is_ok());
    assert!(legacy.is_ok());
    assert!(access_list.is_ok());
    assert!(dynamic_fee.is_ok());
    assert!(blob.is_ok());
    assert!(set_code.is_ok());

    if let (Ok(expected), Ok(legacy), Ok(access_list), Ok(dynamic_fee), Ok(blob), Ok(set_code)) =
        (expected, legacy, access_list, dynamic_fee, blob, set_code)
    {
        let mut scratch = [0_u8; 128];

        assert_eq!(
            validate_legacy_transaction_signature_with_backend(
                ChainId::new(1),
                &legacy,
                Some(expected),
                &mut scratch,
                RealKeccak::default(),
                TestSecp256k1Backend,
                RealKeccak::default(),
            )
            .map(ValidatedTransactionSignature::sender),
            Ok(expected)
        );
        assert_eq!(
            validate_transaction_signature_with_backend(
                ChainId::new(1),
                UnvalidatedTransaction::AccessList(access_list),
                Some(expected),
                &mut scratch,
                RealKeccak::default(),
                TestSecp256k1Backend,
                RealKeccak::default(),
            )
            .map(ValidatedTransactionSignature::sender),
            Ok(expected)
        );
        assert_eq!(
            validate_dynamic_fee_transaction_signature_with_backend(
                ChainId::new(1),
                &dynamic_fee,
                Some(expected),
                &mut scratch,
                RealKeccak::default(),
                TestSecp256k1Backend,
                RealKeccak::default(),
            )
            .map(ValidatedTransactionSignature::sender),
            Ok(expected)
        );
        assert_eq!(
            validate_blob_transaction_signature_with_backend(
                ChainId::new(1),
                &blob,
                Some(expected),
                &mut scratch,
                RealKeccak::default(),
                TestSecp256k1Backend,
                RealKeccak::default(),
            )
            .map(ValidatedTransactionSignature::sender),
            Ok(expected)
        );
        assert_eq!(
            validate_set_code_transaction_signature_with_backend(
                ChainId::new(1),
                &set_code,
                Some(expected),
                &mut scratch,
                RealKeccak::default(),
                TestSecp256k1Backend,
                RealKeccak::default(),
            )
            .map(ValidatedTransactionSignature::sender),
            Ok(expected)
        );
        assert_eq!(
            validate_transaction_signature_with_backend(
                ChainId::new(1),
                UnvalidatedTransaction::SetCode(set_code),
                Some(expected),
                &mut scratch,
                RealKeccak::default(),
                TestSecp256k1Backend,
                RealKeccak::default(),
            )
            .map(ValidatedTransactionSignature::sender),
            Ok(expected)
        );
    }
}

#[test]
fn rejects_wrong_chain_and_wrong_sender() {
    let tx = signed_access_list();
    assert!(tx.is_ok());
    if let Ok(tx) = tx {
        let mut scratch = [0_u8; 128];
        assert_eq!(
            validate_access_list_transaction_signature_with_backend(
                ChainId::new(5),
                &tx,
                None,
                &mut scratch,
                RealKeccak::default(),
                TestSecp256k1Backend,
                RealKeccak::default(),
            ),
            Err(TransactionSignatureValidationError::ReplayDomain(
                VerifyError::WrongChain
            ))
        );
        assert_eq!(
            validate_access_list_transaction_signature_with_backend(
                ChainId::new(1),
                &tx,
                Some(Address::from_bytes([0x44; 20])),
                &mut scratch,
                RealKeccak::default(),
                TestSecp256k1Backend,
                RealKeccak::default(),
            ),
            Err(TransactionSignatureValidationError::WrongSender)
        );
    }
}

#[test]
fn rejects_high_s_and_malformed_scalars() {
    let tx = signed_dynamic_fee();
    assert!(tx.is_ok());
    if let Ok(tx) = tx {
        let mut scratch = [0_u8; 128];
        let high_s = UnvalidatedDynamicFeeTransaction {
            s: [
                0xee, 0x2f, 0x11, 0xef, 0x8c, 0xb0, 0x0a, 0x49, 0x61, 0x7d, 0x13, 0x57, 0xf4, 0xd5,
                0x56, 0x41, 0x09, 0x0a, 0x48, 0xf2, 0x01, 0xe9, 0xb9, 0x59, 0xc4, 0x8f, 0x6f, 0x6b,
                0xec, 0x6f, 0x93, 0x8f,
            ],
            ..tx
        };
        assert_eq!(
            validate_dynamic_fee_transaction_signature_with_backend(
                ChainId::new(1),
                &high_s,
                None,
                &mut scratch,
                RealKeccak::default(),
                TestSecp256k1Backend,
                RealKeccak::default(),
            ),
            Err(TransactionSignatureValidationError::InvalidSignature)
        );

        let malformed = UnvalidatedDynamicFeeTransaction {
            r: [0_u8; 32],
            ..tx
        };
        assert_eq!(
            validate_dynamic_fee_transaction_signature_with_backend(
                ChainId::new(1),
                &malformed,
                None,
                &mut scratch,
                RealKeccak::default(),
                TestSecp256k1Backend,
                RealKeccak::default(),
            ),
            Err(TransactionSignatureValidationError::InvalidSignature)
        );
    }
}

#[test]
fn reports_signing_hash_construction_failures() {
    let tx = signed_blob();
    assert!(tx.is_ok());
    if let Ok(tx) = tx {
        let mut scratch = [0_u8; 8];
        let result = validate_blob_transaction_signature_with_backend(
            ChainId::new(1),
            &tx,
            None,
            &mut scratch,
            RealKeccak::default(),
            TestSecp256k1Backend,
            RealKeccak::default(),
        );
        assert!(matches!(
            result,
            Err(TransactionSignatureValidationError::SigningHash(_))
        ));
    }
}

#[test]
fn set_code_transaction_signature_uses_transaction_domain() {
    let tx = signed_set_code();
    assert!(tx.is_ok());
    if let Ok(tx) = tx {
        let mut scratch = [0_u8; 128];
        let result = validate_set_code_transaction_signature_with_backend(
            ChainId::new(5),
            &tx,
            None,
            &mut scratch,
            RealKeccak::default(),
            TestSecp256k1Backend,
            RealKeccak::default(),
        );
        assert_eq!(
            result,
            Err(TransactionSignatureValidationError::ReplayDomain(
                VerifyError::WrongChain
            ))
        );
    }
}
