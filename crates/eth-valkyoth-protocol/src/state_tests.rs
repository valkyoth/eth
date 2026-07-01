use super::*;

#[test]
fn transaction_typestate_advances_only_with_proofs() {
    let decoded = Transaction::decoded();
    let canonical = decoded.try_into_canonical(Ok(CanonicalValidationProof::new()));
    assert!(canonical.is_ok());
    let Ok(canonical) = canonical else {
        return;
    };

    let fork_validated = canonical.try_into_fork_validated(Ok(ForkValidationProof::new()));
    assert!(fork_validated.is_ok());
    let Ok(fork_validated) = fork_validated else {
        return;
    };

    let sender_recovered = fork_validated.try_into_sender_recovered(Ok(SenderRecoveryProof::new()));

    assert_eq!(sender_recovered, Ok(Transaction::<SenderRecovered>::new()));
}

#[test]
fn failed_canonical_validation_does_not_consume_decoded_state() {
    let decoded = Transaction::decoded();

    let error = decoded
        .try_into_canonical(Err(ProtocolError::InvalidStateTransition))
        .err();
    assert!(error.is_some());
    let Some(error) = error else {
        return;
    };
    assert_eq!(error.error(), ProtocolError::InvalidStateTransition);
    let (decoded, transition_error) = error.into_parts();
    assert_eq!(transition_error, ProtocolError::InvalidStateTransition);

    assert!(
        decoded
            .try_into_canonical(Ok(CanonicalValidationProof::new()))
            .is_ok()
    );
}

#[test]
fn failed_fork_validation_does_not_consume_canonical_state() {
    let canonical = Transaction::<Canonical>::new();

    let error = canonical
        .try_into_fork_validated(Err(ProtocolError::InvalidStateTransition))
        .err();
    assert!(error.is_some());
    let Some(error) = error else {
        return;
    };
    assert_eq!(error.error(), ProtocolError::InvalidStateTransition);
    let (canonical, transition_error) = error.into_parts();
    assert_eq!(transition_error, ProtocolError::InvalidStateTransition);

    assert!(
        canonical
            .try_into_fork_validated(Ok(ForkValidationProof::new()))
            .is_ok()
    );
}

#[test]
fn failed_sender_recovery_does_not_consume_fork_validated_state() {
    let fork_validated = Transaction::<ForkValidated>::new();

    let error = fork_validated
        .try_into_sender_recovered(Err(ProtocolError::InvalidStateTransition))
        .err();
    assert!(error.is_some());
    let Some(error) = error else {
        return;
    };
    assert_eq!(error.error(), ProtocolError::InvalidStateTransition);
    let (fork_validated, transition_error) = error.into_parts();
    assert_eq!(transition_error, ProtocolError::InvalidStateTransition);

    assert!(
        fork_validated
            .try_into_sender_recovered(Ok(SenderRecoveryProof::new()))
            .is_ok()
    );
}
