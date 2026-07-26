extern crate std;

use super::*;
use crate::MptNodeDecodeError;
use eth_valkyoth_codec::{DecodeLimits, DecodeSession, DecodeSessionPolicy};
use eth_valkyoth_hash::{Keccak256, hash_one};
use std::cell::Cell;
use std::vec;
use std::vec::Vec;

const TEST_LIMITS: DecodeLimits = DecodeLimits {
    max_input_bytes: 4096,
    max_list_items: 64,
    max_nesting_depth: 16,
    max_total_allocation: 8192,
    max_proof_nodes: 64,
    max_total_items: 1024,
};

#[test]
fn verifies_reordered_paths_with_one_shared_root() -> Result<(), MptResolverError> {
    let fixture = Fixture::new();
    let entries = fixture.entries();
    let mut session = test_session()?;
    let resolver = MptNodeResolver::try_new(
        fixture.anchor,
        &entries,
        MptResolverLimits::TEST_FIXTURE,
        &mut session,
        TestHasher::default,
    )?;
    let queries = [
        MptBatchQuery::inclusion(&fixture.first_key, &fixture.first_value),
        MptBatchQuery::inclusion(&fixture.second_key, &fixture.second_value),
    ];

    let verified = verify_mpt_multiproof(fixture.anchor, &resolver, &queries, &mut session)?;

    assert_eq!(verified.anchor(), fixture.anchor);
    assert_eq!(verified.query_count(), 2);
    assert_eq!(
        verified.output_bytes(),
        fixture
            .first_value
            .len()
            .saturating_add(fixture.second_value.len())
    );
    Ok(())
}

#[test]
fn rejects_missing_shared_child() -> Result<(), MptResolverError> {
    let fixture = Fixture::new();
    let nodes = fixture.entries_without_first_leaf();
    let mut session = test_session()?;
    let resolver = MptNodeResolver::try_new(
        fixture.anchor,
        &nodes,
        MptResolverLimits::TEST_FIXTURE,
        &mut session,
        TestHasher::default,
    )?;
    let queries = [MptBatchQuery::inclusion(
        &fixture.first_key,
        &fixture.first_value,
    )];

    assert_eq!(
        verify_mpt_multiproof(fixture.anchor, &resolver, &queries, &mut session),
        Err(MptResolverError::Proof(
            MptProofVerificationError::MissingProofNode
        ))
    );
    Ok(())
}

#[test]
fn empty_branch_terminal_is_not_an_inclusion() -> Result<(), MptResolverError> {
    let root = branch(
        hash_one(TestHasher::default(), b"left child"),
        hash_one(TestHasher::default(), b"right child"),
    );
    let root_hash = hash_one(TestHasher::default(), &root);
    let anchor = MptSnapshotAnchor::new(MptProofRoot::from_b256(root_hash));
    let entries = [MptResolvedNode::new(root_hash, &root)];
    let mut session = test_session()?;
    let resolver = MptNodeResolver::try_new(
        anchor,
        &entries,
        MptResolverLimits::TEST_FIXTURE,
        &mut session,
        TestHasher::default,
    )?;
    let queries = [MptBatchQuery::inclusion(&[], &[])];

    assert_eq!(
        verify_mpt_multiproof(anchor, &resolver, &queries, &mut session),
        Err(MptResolverError::Proof(MptProofVerificationError::Absent))
    );
    Ok(())
}

#[test]
fn rejects_short_hash_referenced_child() -> Result<(), MptResolverError> {
    let value = [0x01_u8];
    let short_leaf = leaf(0x30, &value);
    assert!(short_leaf.len() < crate::mpt::MPT_MAX_INLINE_REFERENCE_BYTES);
    let short_leaf_hash = hash_one(TestHasher::default(), &short_leaf);
    let root = branch(
        short_leaf_hash,
        hash_one(TestHasher::default(), b"unused child"),
    );
    let root_hash = hash_one(TestHasher::default(), &root);
    let anchor = MptSnapshotAnchor::new(MptProofRoot::from_b256(root_hash));
    let mut entries = vec![node(&root), node(&short_leaf)];
    entries.sort_by(|left, right| compare_hash(left.hash(), right.hash()));
    let mut session = test_session()?;
    let resolver = MptNodeResolver::try_new(
        anchor,
        &entries,
        MptResolverLimits::TEST_FIXTURE,
        &mut session,
        TestHasher::default,
    )?;
    let queries = [MptBatchQuery::inclusion(&[0x10], &value)];

    assert_eq!(
        verify_mpt_multiproof(anchor, &resolver, &queries, &mut session),
        Err(MptResolverError::Proof(
            MptProofVerificationError::MalformedNode(MptNodeDecodeError::HashedNodeTooShort {
                found: short_leaf.len(),
            })
        ))
    );
    Ok(())
}

#[test]
fn rejects_hash_mismatch_and_missing_root() -> Result<(), MptResolverError> {
    let fixture = Fixture::new();
    let wrong_hash = hash_one(TestHasher::default(), b"incorrect node claim");
    let mismatched = [MptResolvedNode::new(wrong_hash, &fixture.root)];
    let mismatch_anchor = MptSnapshotAnchor::new(MptProofRoot::from_b256(wrong_hash));
    let mut session = test_session()?;

    assert_eq!(
        MptNodeResolver::try_new(
            mismatch_anchor,
            &mismatched,
            MptResolverLimits::TEST_FIXTURE,
            &mut session,
            TestHasher::default,
        )
        .map(|_| ()),
        Err(MptResolverError::HashMismatch)
    );

    let entries = fixture.entries();
    let absent_hash = hash_one(TestHasher::default(), b"absent resolver root");
    let absent_anchor = MptSnapshotAnchor::new(MptProofRoot::from_b256(absent_hash));
    let mut session = test_session()?;
    assert_eq!(
        MptNodeResolver::try_new(
            absent_anchor,
            &entries,
            MptResolverLimits::TEST_FIXTURE,
            &mut session,
            TestHasher::default,
        )
        .map(|_| ()),
        Err(MptResolverError::MissingRoot)
    );
    Ok(())
}

#[test]
fn rejects_node_query_and_zero_limits() -> Result<(), MptResolverError> {
    assert_eq!(
        MptResolverLimits::reviewed(0, 1, 1),
        Err(MptResolverError::InvalidLimits)
    );
    assert_eq!(
        MptResolverLimits::reviewed(1, 0, 1),
        Err(MptResolverError::InvalidLimits)
    );
    assert_eq!(
        MptResolverLimits::reviewed(1, 1, 0),
        Err(MptResolverError::InvalidLimits)
    );

    let fixture = Fixture::new();
    let entries = fixture.entries();
    let node_limits = MptResolverLimits::reviewed(1, 2, 128)?;
    let mut session = test_session()?;
    assert_eq!(
        MptNodeResolver::try_new(
            fixture.anchor,
            &entries,
            node_limits,
            &mut session,
            TestHasher::default,
        )
        .map(|_| ()),
        Err(MptResolverError::NodeLimitExceeded)
    );

    let query_limits = MptResolverLimits::reviewed(entries.len(), 1, 128)?;
    let mut session = test_session()?;
    let resolver = MptNodeResolver::try_new(
        fixture.anchor,
        &entries,
        query_limits,
        &mut session,
        TestHasher::default,
    )?;
    let queries = [
        MptBatchQuery::inclusion(&fixture.first_key, &fixture.first_value),
        MptBatchQuery::inclusion(&fixture.second_key, &fixture.second_value),
    ];
    assert_eq!(
        verify_mpt_multiproof(fixture.anchor, &resolver, &queries, &mut session),
        Err(MptResolverError::QueryLimitExceeded)
    );
    Ok(())
}

#[test]
fn rejects_duplicate_and_unsorted_entries() -> Result<(), MptResolverError> {
    let fixture = Fixture::new();
    let mut entries = fixture.entries();
    let duplicate = entries.first().copied().into_iter().collect::<Vec<_>>();
    entries.extend(duplicate);
    entries.sort_by(|left, right| compare_hash(left.hash(), right.hash()));
    let mut session = test_session()?;

    assert_eq!(
        MptNodeResolver::try_new(
            fixture.anchor,
            &entries,
            MptResolverLimits::TEST_FIXTURE,
            &mut session,
            TestHasher::default,
        )
        .map(|_| ()),
        Err(MptResolverError::DuplicateNode)
    );

    let mut reversed = fixture.entries();
    reversed.reverse();
    let mut session = test_session()?;
    assert_eq!(
        MptNodeResolver::try_new(
            fixture.anchor,
            &reversed,
            MptResolverLimits::TEST_FIXTURE,
            &mut session,
            TestHasher::default,
        )
        .map(|_| ()),
        Err(MptResolverError::UnsortedNodes)
    );
    Ok(())
}

#[test]
fn rejects_snapshot_mixing_and_output_overflow() -> Result<(), MptResolverError> {
    let fixture = Fixture::new();
    let entries = fixture.entries();
    let mut session = test_session()?;
    let resolver = MptNodeResolver::try_new(
        fixture.anchor,
        &entries,
        MptResolverLimits::TEST_FIXTURE,
        &mut session,
        TestHasher::default,
    )?;
    let query = [MptBatchQuery::inclusion(
        &fixture.first_key,
        &fixture.first_value,
    )];
    let other_anchor = MptSnapshotAnchor::new(MptProofRoot::from_b256(hash_one(
        TestHasher::default(),
        b"other snapshot",
    )));

    assert_eq!(
        verify_mpt_multiproof(other_anchor, &resolver, &query, &mut session),
        Err(MptResolverError::SnapshotMismatch)
    );

    let limits = MptResolverLimits::reviewed(8, 2, 1)?;
    let entries = fixture.entries();
    let mut session = test_session()?;
    let resolver = MptNodeResolver::try_new(
        fixture.anchor,
        &entries,
        limits,
        &mut session,
        TestHasher::default,
    )?;
    assert_eq!(
        verify_mpt_multiproof(fixture.anchor, &resolver, &query, &mut session),
        Err(MptResolverError::OutputLimitExceeded)
    );
    Ok(())
}

#[test]
fn malformed_node_fails_before_first_hash() -> Result<(), MptResolverError> {
    let malformed = [0xc0_u8];
    let hash = hash_one(TestHasher::default(), &malformed);
    let entries = [MptResolvedNode::new(hash, &malformed)];
    let anchor = MptSnapshotAnchor::new(MptProofRoot::from_b256(hash));
    let calls = Cell::new(0_usize);
    let mut session = test_session()?;

    let result = MptNodeResolver::try_new(
        anchor,
        &entries,
        MptResolverLimits::TEST_FIXTURE,
        &mut session,
        || {
            calls.set(calls.get().saturating_add(1));
            TestHasher::default()
        },
    );

    assert!(matches!(result, Err(MptResolverError::Proof(_))));
    assert_eq!(calls.get(), 0);
    Ok(())
}

struct Fixture {
    anchor: MptSnapshotAnchor,
    root: Vec<u8>,
    first_leaf: Vec<u8>,
    second_leaf: Vec<u8>,
    first_key: [u8; 1],
    second_key: [u8; 1],
    first_value: [u8; 40],
    second_value: [u8; 40],
}

impl Fixture {
    fn new() -> Self {
        let first_value = [0x11_u8; 40];
        let second_value = [0x22_u8; 40];
        let first_leaf = leaf(0x30, &first_value);
        let second_leaf = leaf(0x30, &second_value);
        let first_hash = hash_one(TestHasher::default(), &first_leaf);
        let second_hash = hash_one(TestHasher::default(), &second_leaf);
        let root = branch(first_hash, second_hash);
        let root_hash = hash_one(TestHasher::default(), &root);
        Self {
            anchor: MptSnapshotAnchor::new(MptProofRoot::from_b256(root_hash)),
            root,
            first_leaf,
            second_leaf,
            first_key: [0x10],
            second_key: [0x20],
            first_value,
            second_value,
        }
    }

    fn entries(&self) -> Vec<MptResolvedNode<'_>> {
        let mut entries = vec![
            node(&self.root),
            node(&self.first_leaf),
            node(&self.second_leaf),
        ];
        entries.sort_by(|left, right| compare_hash(left.hash(), right.hash()));
        entries
    }

    fn entries_without_first_leaf(&self) -> Vec<MptResolvedNode<'_>> {
        let first_hash = hash_one(TestHasher::default(), &self.first_leaf);
        let mut entries = self.entries();
        entries.retain(|entry| entry.hash() != first_hash);
        entries
    }
}

fn node(encoded: &[u8]) -> MptResolvedNode<'_> {
    MptResolvedNode::new(hash_one(TestHasher::default(), encoded), encoded)
}

fn leaf(compact: u8, value: &[u8]) -> Vec<u8> {
    list(&[scalar(&[compact]), scalar(value)])
}

fn branch(first: B256, second: B256) -> Vec<u8> {
    let empty = scalar(&[]);
    let mut fields = Vec::with_capacity(17);
    for index in 0..16 {
        match index {
            1 => fields.push(scalar(&first.to_bytes())),
            2 => fields.push(scalar(&second.to_bytes())),
            _ => fields.push(empty.clone()),
        }
    }
    fields.push(empty);
    list(&fields)
}

fn scalar(payload: &[u8]) -> Vec<u8> {
    if let [byte] = payload
        && *byte < 0x80
    {
        return vec![*byte];
    }
    let mut output = Vec::with_capacity(payload.len().saturating_add(1));
    output.push(0x80_u8.saturating_add(u8::try_from(payload.len()).unwrap_or(u8::MAX)));
    output.extend_from_slice(payload);
    output
}

fn list(fields: &[Vec<u8>]) -> Vec<u8> {
    let payload_len = fields.iter().map(Vec::len).sum::<usize>();
    let mut output = Vec::new();
    if payload_len < 56 {
        output.push(0xc0_u8.saturating_add(u8::try_from(payload_len).unwrap_or(u8::MAX)));
    } else {
        output.push(0xf8);
        output.push(u8::try_from(payload_len).unwrap_or(u8::MAX));
    }
    for field in fields {
        output.extend_from_slice(field);
    }
    output
}

#[derive(Default)]
struct TestHasher {
    state: [u8; 32],
    position: usize,
}

impl Keccak256 for TestHasher {
    fn update(&mut self, input: &[u8]) {
        for byte in input {
            let index = self
                .position
                .checked_rem(self.state.len())
                .unwrap_or_default();
            if let Some(slot) = self.state.get_mut(index) {
                *slot = slot.wrapping_mul(31).wrapping_add(*byte);
            }
            self.position = self.position.saturating_add(1);
        }
        if let Some(first) = self.state.first_mut() {
            *first ^= u8::try_from(input.len()).unwrap_or(u8::MAX);
        }
    }

    fn finalize(self) -> B256 {
        B256::from_bytes(self.state)
    }
}

fn test_session() -> Result<DecodeSession, MptResolverError> {
    let policy = DecodeSessionPolicy::compatibility_policy(TEST_LIMITS).map_err(resource_error)?;
    DecodeSession::new(policy).map_err(resource_error)
}
