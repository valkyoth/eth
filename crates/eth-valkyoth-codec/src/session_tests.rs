use super::*;

fn policy() -> DecodeSessionPolicy {
    DecodeSessionPolicy::reviewed_policy(
        DecodeLimits::reviewed_policy(16, 4, 4, 16, 4, 8),
        32,
        8,
        4,
        32,
        16,
        16,
        96,
    )
    .unwrap_or(DecodeSessionPolicy::TEST_FIXTURE)
}

#[test]
fn session_accounts_every_work_domain() -> Result<(), DecodeError> {
    let mut session = DecodeSession::new(policy())?;
    session.check_input_len(16)?;
    session.check_list_count(4)?;
    session.check_nesting_depth(3)?;
    session.account_encoded_bytes(7)?;
    session.account_rlp_headers(2)?;
    session.account_items(3)?;
    session.account_allocation_capacity(5)?;
    session.account_proof_nodes(1)?;
    session.account_hashes(1, 6)?;
    session.account_nibbles(4)?;
    session.account_value_bytes(5)?;

    assert_eq!(session.encoded_bytes(), 7);
    assert_eq!(session.rlp_headers(), 2);
    assert_eq!(session.items(), 3);
    assert_eq!(session.max_nesting_depth(), 3);
    assert_eq!(session.allocation_capacity(), 5);
    assert_eq!(session.proof_nodes(), 1);
    assert_eq!(session.hashes(), 1);
    assert_eq!(session.hash_bytes(), 6);
    assert_eq!(session.nibbles(), 4);
    assert_eq!(session.value_bytes(), 5);
    assert_eq!(session.total_work(), 37);
    Ok(())
}

#[test]
fn failed_multi_counter_charge_is_atomic() -> Result<(), DecodeError> {
    let limits = DecodeLimits::reviewed_policy(4, 1, 1, 1, 1, 1);
    let policy = DecodeSessionPolicy::reviewed_policy(limits, 4, 1, 1, 1, 1, 1, 8)?;
    let mut session = DecodeSession::new(policy)?;

    session.check_hash_capacity(1, 1)?;
    assert_eq!(session.hashes(), 0);
    assert_eq!(session.hash_bytes(), 0);
    assert_eq!(session.total_work(), 0);
    assert_eq!(
        session.check_hash_capacity(2, 1),
        Err(DecodeError::HashCountExceeded)
    );
    assert_eq!(
        session.account_hashes(1, 2),
        Err(DecodeError::HashBytesExceeded)
    );
    assert_eq!(session.hashes(), 0);
    assert_eq!(session.hash_bytes(), 0);
    assert_eq!(session.total_work(), 0);
    Ok(())
}

#[test]
fn aggregate_work_limit_fails_without_component_commit() -> Result<(), DecodeError> {
    let limits = DecodeLimits::reviewed_policy(1, 1, 1, 1, 1, 1);
    let policy = DecodeSessionPolicy::reviewed_policy(limits, 1, 1, 1, 1, 1, 1, 1)?;
    let mut session = DecodeSession::new(policy)?;
    session.account_encoded_bytes(1)?;

    assert_eq!(session.account_items(1), Err(DecodeError::WorkExceeded));
    assert_eq!(session.items(), 0);
    assert_eq!(session.total_work(), 1);
    Ok(())
}

#[test]
fn policy_rejects_inconsistent_cross_limits() {
    let limits = DecodeLimits::reviewed_policy(8, 4, 2, 8, 2, 3);
    assert_eq!(
        DecodeSessionPolicy::reviewed_policy(limits, 8, 8, 2, 8, 8, 8, 32),
        Err(DecodeError::InvalidSessionPolicy)
    );
}

#[test]
fn policy_rejects_item_and_proof_ceilings_above_total_work() {
    let item_limits = DecodeLimits::reviewed_policy(1, 1, 1, 1, 1, 3);
    assert_eq!(
        DecodeSessionPolicy::reviewed_policy(item_limits, 1, 1, 1, 1, 1, 1, 2),
        Err(DecodeError::InvalidSessionPolicy)
    );

    let proof_limits = DecodeLimits::reviewed_policy(1, 1, 1, 1, 3, 3);
    assert_eq!(
        DecodeSessionPolicy::reviewed_policy(proof_limits, 1, 1, 1, 1, 1, 1, 2),
        Err(DecodeError::InvalidSessionPolicy)
    );
}

#[test]
fn trie_work_domains_fail_without_partial_commit() -> Result<(), DecodeError> {
    let limits = DecodeLimits::reviewed_policy(8, 4, 2, 8, 2, 4);
    let policy = DecodeSessionPolicy::reviewed_policy(limits, 8, 4, 2, 8, 2, 3, 64)?;
    let mut session = DecodeSession::new(policy)?;

    session.account_nibbles(2)?;
    assert_eq!(
        session.account_nibbles(1),
        Err(DecodeError::NibbleCountExceeded)
    );
    assert_eq!(session.nibbles(), 2);
    session.account_value_bytes(3)?;
    assert_eq!(
        session.account_value_bytes(1),
        Err(DecodeError::ValueBytesExceeded)
    );
    assert_eq!(session.value_bytes(), 3);
    Ok(())
}

#[test]
fn deployment_starter_requires_complete_review() {
    assert_eq!(
        DecodeSessionPolicy::DEPLOYMENT_STARTING_POINT.validate_deployment_policy(),
        Err(DecodeError::UnreviewedDeploymentPolicy)
    );
}
