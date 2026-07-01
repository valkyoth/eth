use super::*;

#[test]
fn transaction_typestate_advances_only_with_proofs() {
    let decoded = Transaction::decoded();
    let canonical = decoded.try_into_canonical(Ok(CanonicalValidationProof::new()));
    assert!(canonical.is_ok());

    let fork_validated =
        canonical.and_then(|tx| tx.try_into_fork_validated(Ok(ForkValidationProof::new())));
    assert!(fork_validated.is_ok());

    let sender_recovered =
        fork_validated.and_then(|tx| tx.try_into_sender_recovered(Ok(SenderRecoveryProof::new())));

    assert_eq!(sender_recovered, Ok(Transaction::<SenderRecovered>::new()));
}

#[test]
fn failed_canonical_validation_does_not_consume_decoded_state() {
    let decoded = Transaction::decoded();

    assert_eq!(
        decoded.try_into_canonical(Err(ProtocolError::InvalidStateTransition)),
        Err(ProtocolError::InvalidStateTransition)
    );

    assert!(
        decoded
            .try_into_canonical(Ok(CanonicalValidationProof::new()))
            .is_ok()
    );
}

#[test]
fn failed_fork_validation_does_not_consume_canonical_state() {
    let canonical = Transaction::<Canonical>::new();

    assert_eq!(
        canonical.try_into_fork_validated(Err(ProtocolError::InvalidStateTransition)),
        Err(ProtocolError::InvalidStateTransition)
    );

    assert!(
        canonical
            .try_into_fork_validated(Ok(ForkValidationProof::new()))
            .is_ok()
    );
}

#[test]
fn failed_sender_recovery_does_not_consume_fork_validated_state() {
    let fork_validated = Transaction::<ForkValidated>::new();

    assert_eq!(
        fork_validated.try_into_sender_recovered(Err(ProtocolError::InvalidStateTransition)),
        Err(ProtocolError::InvalidStateTransition)
    );

    assert!(
        fork_validated
            .try_into_sender_recovered(Ok(SenderRecoveryProof::new()))
            .is_ok()
    );
}
