use core::cmp::Ordering;
use core::fmt;

use eth_valkyoth_codec::{DecodeError, DecodeSession};
use eth_valkyoth_hash::{Keccak256, hash_one};
use eth_valkyoth_primitives::B256;

use crate::mpt::{MptNodeField, decode_mpt_node_body_in_session};
use crate::mpt_proof::{
    MptProofRoot, MptProofVerificationError, proof_resource_error, verify_key_in_resolver,
};

/// Immutable trusted root binding for one resolver-backed proof batch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MptSnapshotAnchor {
    root: MptProofRoot,
}

impl MptSnapshotAnchor {
    /// Creates an anchor from a root obtained through an independent trust path.
    #[must_use]
    pub const fn new(root: MptProofRoot) -> Self {
        Self { root }
    }

    /// Returns the root bound to this snapshot.
    #[must_use]
    pub const fn root(self) -> MptProofRoot {
        self.root
    }
}

/// One hash-addressed encoded MPT node.
#[derive(Clone, Copy, Debug)]
pub struct MptResolvedNode<'a> {
    hash: B256,
    encoded: &'a [u8],
}

impl<'a> MptResolvedNode<'a> {
    /// Creates an entry. Resolver admission verifies `hash` against `encoded`.
    #[must_use]
    pub const fn new(hash: B256, encoded: &'a [u8]) -> Self {
        Self { hash, encoded }
    }

    /// Returns the claimed node hash.
    #[must_use]
    pub const fn hash(self) -> B256 {
        self.hash
    }

    /// Returns the encoded node.
    #[must_use]
    pub const fn encoded(self) -> &'a [u8] {
        self.encoded
    }
}

/// Resolver and batch output limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MptResolverLimits {
    max_nodes: usize,
    max_queries: usize,
    max_output_bytes: usize,
}

impl MptResolverLimits {
    /// Conservative test policy.
    pub const TEST_FIXTURE: Self = Self {
        max_nodes: 64,
        max_queries: 16,
        max_output_bytes: 4096,
    };

    /// Creates an explicit resolver policy.
    pub const fn reviewed(
        max_nodes: usize,
        max_queries: usize,
        max_output_bytes: usize,
    ) -> Result<Self, MptResolverError> {
        if max_nodes == 0 || max_queries == 0 || max_output_bytes == 0 {
            return Err(MptResolverError::InvalidLimits);
        }
        Ok(Self {
            max_nodes,
            max_queries,
            max_output_bytes,
        })
    }

    #[cfg(feature = "alloc")]
    pub(crate) const fn max_nodes(self) -> usize {
        self.max_nodes
    }
}

/// Borrowed, bounded, hash-addressed MPT node resolver.
///
/// Entries must be strictly sorted by hash and therefore cannot contain
/// duplicates. This keeps lookup logarithmic and admission allocation-free.
pub struct MptNodeResolver<'a> {
    anchor: MptSnapshotAnchor,
    nodes: &'a [MptResolvedNode<'a>],
    limits: MptResolverLimits,
}

impl<'a> MptNodeResolver<'a> {
    /// Admits a node set after complete decode, ordering, and digest checks.
    ///
    /// Every node is decoded before the first node is hashed.
    pub fn try_new<H>(
        anchor: MptSnapshotAnchor,
        nodes: &'a [MptResolvedNode<'a>],
        limits: MptResolverLimits,
        session: &mut DecodeSession,
        mut new_hasher: impl FnMut() -> H,
    ) -> Result<Self, MptResolverError>
    where
        H: Keccak256,
    {
        if nodes.is_empty() {
            return Err(MptResolverError::MissingRoot);
        }
        if nodes.len() > limits.max_nodes {
            return Err(MptResolverError::NodeLimitExceeded);
        }
        session
            .account_proof_nodes(nodes.len())
            .map_err(resource_error)?;

        let mut hash_bytes = 0_usize;
        for node in nodes {
            session
                .check_input_len(node.encoded.len())
                .map_err(resource_error)?;
            hash_bytes = hash_bytes
                .checked_add(node.encoded.len())
                .ok_or(MptResolverError::Resource(DecodeError::HashBytesExceeded))?;
            decode_mpt_node_body_in_session(node.encoded, session).map_err(|error| {
                MptResolverError::Proof(MptProofVerificationError::MalformedNode(error))
            })?;
        }
        session
            .check_hash_capacity(nodes.len(), hash_bytes)
            .map_err(resource_error)?;

        let mut previous: Option<B256> = None;
        for node in nodes {
            if let Some(prior) = previous {
                match compare_hash(prior, node.hash) {
                    Ordering::Equal => return Err(MptResolverError::DuplicateNode),
                    Ordering::Greater => return Err(MptResolverError::UnsortedNodes),
                    Ordering::Less => {}
                }
            }
            session
                .account_hashes(1, node.encoded.len())
                .map_err(resource_error)?;
            if hash_one(new_hasher(), node.encoded) != node.hash {
                return Err(MptResolverError::HashMismatch);
            }
            previous = Some(node.hash);
        }

        let resolver = Self {
            anchor,
            nodes,
            limits,
        };
        if resolver.resolve(anchor.root.to_b256()).is_none() {
            return Err(MptResolverError::MissingRoot);
        }
        Ok(resolver)
    }

    /// Returns the immutable snapshot anchor.
    #[must_use]
    pub const fn anchor(&self) -> MptSnapshotAnchor {
        self.anchor
    }

    /// Resolves one encoded node by Keccak hash.
    #[must_use]
    pub fn resolve(&self, hash: B256) -> Option<&'a [u8]> {
        self.nodes
            .binary_search_by(|node| compare_hash(node.hash, hash))
            .ok()
            .and_then(|index| self.nodes.get(index))
            .map(|node| node.encoded)
    }
}

/// One expected key/value inclusion in a resolver-backed multiproof.
#[derive(Clone, Copy, Debug)]
pub struct MptBatchQuery<'a> {
    key: &'a [u8],
    expected_value: &'a [u8],
}

impl<'a> MptBatchQuery<'a> {
    /// Creates an inclusion query.
    #[must_use]
    pub const fn inclusion(key: &'a [u8], expected_value: &'a [u8]) -> Self {
        Self {
            key,
            expected_value,
        }
    }
}

/// Successful resolver-backed batch verification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerifiedMptBatch {
    anchor: MptSnapshotAnchor,
    query_count: usize,
    output_bytes: usize,
}

impl VerifiedMptBatch {
    /// Returns the snapshot root shared by every query.
    #[must_use]
    pub const fn anchor(self) -> MptSnapshotAnchor {
        self.anchor
    }

    /// Returns the number of verified inclusions.
    #[must_use]
    pub const fn query_count(self) -> usize {
        self.query_count
    }

    /// Returns cumulative verified value bytes.
    #[must_use]
    pub const fn output_bytes(self) -> usize {
        self.output_bytes
    }
}

/// Verifies a bounded batch against one resolver and snapshot.
pub fn verify_mpt_multiproof(
    anchor: MptSnapshotAnchor,
    resolver: &MptNodeResolver<'_>,
    queries: &[MptBatchQuery<'_>],
    session: &mut DecodeSession,
) -> Result<VerifiedMptBatch, MptResolverError> {
    if anchor != resolver.anchor {
        return Err(MptResolverError::SnapshotMismatch);
    }
    if queries.is_empty() || queries.len() > resolver.limits.max_queries {
        return Err(MptResolverError::QueryLimitExceeded);
    }
    let mut output_bytes = 0_usize;
    for query in queries {
        output_bytes = output_bytes
            .checked_add(query.expected_value.len())
            .ok_or(MptResolverError::OutputLimitExceeded)?;
        if output_bytes > resolver.limits.max_output_bytes {
            return Err(MptResolverError::OutputLimitExceeded);
        }
    }
    for query in queries {
        verify_key_in_resolver(resolver, query.key, query.expected_value, session)
            .map_err(MptResolverError::Proof)?;
    }
    Ok(VerifiedMptBatch {
        anchor,
        query_count: queries.len(),
        output_bytes,
    })
}

/// Resolver-backed multiproof failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MptResolverError {
    /// A limit was zero.
    InvalidLimits,
    /// The node set exceeded its configured bound.
    NodeLimitExceeded,
    /// The query batch was empty or exceeded its configured bound.
    QueryLimitExceeded,
    /// Cumulative verified value bytes exceeded the configured bound.
    OutputLimitExceeded,
    /// Entries were not strictly hash sorted.
    UnsortedNodes,
    /// The node set contained a duplicate hash.
    DuplicateNode,
    /// An encoded node did not match its claimed hash.
    HashMismatch,
    /// The resolver did not contain its anchored root.
    MissingRoot,
    /// A query used an anchor different from the resolver snapshot.
    SnapshotMismatch,
    /// A decode-session resource limit rejected admission.
    Resource(DecodeError),
    /// MPT traversal failed.
    Proof(MptProofVerificationError),
}

impl MptResolverError {
    /// Returns the stable high-level category.
    #[must_use]
    pub const fn category(self) -> MptResolverErrorCategory {
        match self {
            Self::InvalidLimits
            | Self::NodeLimitExceeded
            | Self::QueryLimitExceeded
            | Self::OutputLimitExceeded
            | Self::Resource(_) => MptResolverErrorCategory::Resource,
            Self::UnsortedNodes
            | Self::DuplicateNode
            | Self::HashMismatch
            | Self::MissingRoot
            | Self::Proof(_) => MptResolverErrorCategory::Malformed,
            Self::SnapshotMismatch => MptResolverErrorCategory::Snapshot,
        }
    }
}

impl fmt::Display for MptResolverError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "MPT resolver error: {:?}", self.category())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for MptResolverError {}

/// Stable resolver failure category.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MptResolverErrorCategory {
    /// Invalid or exceeded resource policy.
    Resource,
    /// Malformed, missing, duplicated, or mismatched node data.
    Malformed,
    /// Resolver and query batch used different snapshots.
    Snapshot,
}

fn compare_hash(left: B256, right: B256) -> Ordering {
    left.to_bytes().iter().cmp(right.to_bytes().iter())
}

fn resource_error(error: DecodeError) -> MptResolverError {
    let proof = proof_resource_error(error);
    match proof {
        MptProofVerificationError::MalformedNode(crate::mpt::MptNodeDecodeError::FieldDecode {
            field: MptNodeField::ProofNode,
            source,
        }) => MptResolverError::Resource(source),
        _ => MptResolverError::Proof(proof),
    }
}

#[cfg(test)]
#[path = "mpt_resolver_tests.rs"]
mod tests;
