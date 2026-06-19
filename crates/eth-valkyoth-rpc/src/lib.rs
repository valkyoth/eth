#![no_std]
#![forbid(unsafe_code)]
//! Future RPC trust-policy boundary for `eth`.

/// RPC response trust model.
///
/// Prefer [`RpcTrustModel::Verified`] for any state the caller acts on.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RpcTrustModel {
    /// Proofs are verified against a trusted header or checkpoint.
    Verified,
    /// Matching responses are required across independent nodes.
    Quorum,
    /// Data is accepted from one configured node.
    ///
    /// Use only when provider identity and transport integrity are established
    /// by other means.
    Trusted,
}
