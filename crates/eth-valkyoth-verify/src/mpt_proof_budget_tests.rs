extern crate std;

use std::cell::Cell;

use eth_valkyoth_codec::{DecodeLimits, DecodeSession, DecodeSessionPolicy};

use super::tests::{TestHasher, index_key, leaf_node, test_hash, tx_value};
use super::*;

#[derive(Clone, Copy, Eq, PartialEq)]
enum BudgetDomain {
    EncodedBytes,
    RlpHeaders,
    Items,
    Nibbles,
    ValueBytes,
    TotalWork,
}

#[derive(Clone, Copy)]
struct Counts {
    encoded_bytes: usize,
    rlp_headers: usize,
    items: usize,
    nesting_depth: usize,
    allocation_capacity: usize,
    proof_nodes: usize,
    hashes: usize,
    hash_bytes: usize,
    nibbles: usize,
    value_bytes: usize,
    total_work: usize,
}

impl Counts {
    const fn read(session: &DecodeSession) -> Self {
        Self {
            encoded_bytes: session.encoded_bytes(),
            rlp_headers: session.rlp_headers(),
            items: session.items(),
            nesting_depth: session.max_nesting_depth(),
            allocation_capacity: session.allocation_capacity(),
            proof_nodes: session.proof_nodes(),
            hashes: session.hashes(),
            hash_bytes: session.hash_bytes(),
            nibbles: session.nibbles(),
            value_bytes: session.value_bytes(),
            total_work: session.total_work(),
        }
    }

    const fn selected(self, domain: BudgetDomain) -> usize {
        match domain {
            BudgetDomain::EncodedBytes => self.encoded_bytes,
            BudgetDomain::RlpHeaders => self.rlp_headers,
            BudgetDomain::Items => self.items,
            BudgetDomain::Nibbles => self.nibbles,
            BudgetDomain::ValueBytes => self.value_bytes,
            BudgetDomain::TotalWork => self.total_work,
        }
    }
}

#[test]
fn every_remaining_traversal_budget_fails_before_proof_hashing() -> Result<(), &'static str> {
    let key = index_key(0)?;
    let value = tx_value();
    let root_node = leaf_node(&key, &value);
    let root = TransactionTrieRoot::from_b256(test_hash(&root_node));
    let proof = [&root_node[..]];
    let roomy = roomy_policy(root_node.len(), value.len())?;

    let mut preflight_session = DecodeSession::new(roomy).map_err(|_| "preflight session")?;
    preflight_proof(&proof, &value, 0, 0, &mut preflight_session)
        .map_err(|_| "preflight succeeds")?;
    let preflight = Counts::read(&preflight_session);

    let mut complete_session = DecodeSession::new(roomy).map_err(|_| "complete session")?;
    verify_transaction_inclusion_in_session(
        root,
        0,
        &value,
        &proof,
        &mut complete_session,
        TestHasher::default,
    )
    .map_err(|_| "proof verifies")?;
    let complete = Counts::read(&complete_session);

    let domains = [
        BudgetDomain::EncodedBytes,
        BudgetDomain::RlpHeaders,
        BudgetDomain::Items,
        BudgetDomain::Nibbles,
        BudgetDomain::ValueBytes,
        BudgetDomain::TotalWork,
    ];
    for domain in domains {
        let maximum = complete
            .selected(domain)
            .checked_sub(1)
            .ok_or("complete work must be nonzero")?;
        if maximum < preflight.selected(domain) {
            return Err("constrained budget must admit preflight");
        }
        let policy = constrained_policy(root_node.len(), value.len(), complete, domain, maximum)?;
        let mut session = DecodeSession::new(policy).map_err(|_| "constrained session")?;
        let calls = Cell::new(0usize);

        let result =
            verify_transaction_inclusion_in_session(root, 0, &value, &proof, &mut session, || {
                calls.set(calls.get().saturating_add(1));
                TestHasher::default()
            });

        assert!(matches!(
            result,
            Err(MptProofVerificationError::MalformedNode(_))
        ));
        assert_eq!(calls.get(), 0);
        assert_eq!(session.hashes(), 0);
        assert_eq!(session.hash_bytes(), 0);

        if domain == BudgetDomain::TotalWork {
            let after_first = Counts::read(&session);
            assert!(after_first.total_work > preflight.total_work);
            let repeated = verify_transaction_inclusion_in_session(
                root,
                0,
                &value,
                &proof,
                &mut session,
                || {
                    calls.set(calls.get().saturating_add(1));
                    TestHasher::default()
                },
            );
            assert!(repeated.is_err());
            assert!(session.total_work() > after_first.total_work);
            assert_eq!(calls.get(), 0);
            assert_eq!(session.hashes(), 0);
        }
    }
    Ok(())
}

#[test]
fn near_exhaustion_stops_and_debits_dry_traversal() -> Result<(), &'static str> {
    let key = index_key(0)?;
    let value = tx_value();
    let root_node = leaf_node(&key, &value);
    let root = TransactionTrieRoot::from_b256(test_hash(&root_node));
    let proof = [&root_node[..]];
    let roomy = roomy_policy(root_node.len(), value.len())?;

    let mut measured = DecodeSession::new(roomy).map_err(|_| "measured session")?;
    preflight_proof(&proof, &value, 0, 0, &mut measured).map_err(|_| "preflight succeeds")?;
    let preflight = Counts::read(&measured);
    check_preflighted_key_inclusion_capacity(&key, &value, &proof, &mut measured)
        .map_err(|_| "planning succeeds")?;
    let planned = Counts::read(&measured);
    let planning_work = planned
        .total_work
        .checked_sub(preflight.total_work)
        .ok_or("planning work delta")?;
    let mut complete_session = DecodeSession::new(roomy).map_err(|_| "complete session")?;
    verify_transaction_inclusion_in_session(
        root,
        0,
        &value,
        &proof,
        &mut complete_session,
        TestHasher::default,
    )
    .map_err(|_| "proof verifies")?;
    let complete = Counts::read(&complete_session);
    let maximum = planned
        .nibbles
        .checked_sub(1)
        .ok_or("planned nibble work must be nonzero")?;
    let policy = constrained_policy(
        root_node.len(),
        value.len(),
        complete,
        BudgetDomain::Nibbles,
        maximum,
    )?;
    let mut session = DecodeSession::new(policy).map_err(|_| "constrained session")?;
    let calls = Cell::new(0usize);

    let first =
        verify_transaction_inclusion_in_session(root, 0, &value, &proof, &mut session, || {
            calls.set(calls.get().saturating_add(1));
            TestHasher::default()
        });
    assert!(first.is_err());
    assert!(session.total_work() > preflight.total_work);
    assert_eq!(calls.get(), 0);
    assert_eq!(session.hashes(), 0);

    let before_retry = session.total_work();
    let second =
        verify_transaction_inclusion_in_session(root, 0, &value, &proof, &mut session, || {
            calls.set(calls.get().saturating_add(1));
            TestHasher::default()
        });
    assert!(second.is_err());
    let retry_work = session.total_work().saturating_sub(before_retry);
    assert!(retry_work < planning_work);
    assert_eq!(calls.get(), 0);
    assert_eq!(session.hashes(), 0);
    Ok(())
}

fn roomy_policy(node_len: usize, value_len: usize) -> Result<DecodeSessionPolicy, &'static str> {
    let limits = DecodeLimits::reviewed_policy(node_len.max(value_len), 2, 4, 1, 1, 32);
    DecodeSessionPolicy::reviewed_policy(limits, 256, 32, 2, 256, 32, 64, 512)
        .map_err(|_| "roomy policy")
}

fn constrained_policy(
    node_len: usize,
    value_len: usize,
    complete: Counts,
    domain: BudgetDomain,
    maximum: usize,
) -> Result<DecodeSessionPolicy, &'static str> {
    let max_items = selected_or(complete.items, domain, BudgetDomain::Items, maximum);
    let limits = DecodeLimits::reviewed_policy(
        node_len.max(value_len),
        2,
        complete.nesting_depth.max(1),
        complete.allocation_capacity.max(1),
        complete.proof_nodes,
        max_items,
    );
    DecodeSessionPolicy::reviewed_policy(
        limits,
        selected_or(
            complete.encoded_bytes,
            domain,
            BudgetDomain::EncodedBytes,
            maximum,
        ),
        selected_or(
            complete.rlp_headers,
            domain,
            BudgetDomain::RlpHeaders,
            maximum,
        ),
        complete.hashes,
        complete.hash_bytes,
        selected_or(complete.nibbles, domain, BudgetDomain::Nibbles, maximum),
        selected_or(
            complete.value_bytes,
            domain,
            BudgetDomain::ValueBytes,
            maximum,
        ),
        selected_or(
            complete.total_work,
            domain,
            BudgetDomain::TotalWork,
            maximum,
        ),
    )
    .map_err(|_| "constrained policy")
}

fn selected_or(
    regular: usize,
    selected: BudgetDomain,
    candidate: BudgetDomain,
    replacement: usize,
) -> usize {
    if selected == candidate {
        replacement
    } else {
        regular
    }
}
