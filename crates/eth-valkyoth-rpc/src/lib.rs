#![no_std]
#![forbid(unsafe_code)]
//! Future RPC trust-policy boundary for `eth`.

/// RPC response trust model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RpcTrustModel {
    /// Data is accepted from one configured node.
    Trusted,
    /// Matching responses are required across independent nodes.
    Quorum,
    /// Proofs are verified against a trusted header or checkpoint.
    Verified,
}
