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
    /// Data is accepted from one configured node after explicit review.
    ///
    /// Use only when provider identity and transport integrity are established
    /// by other means.
    Trusted {
        /// Static reason documenting the identity and transport controls.
        acknowledgment: &'static str,
    },
}

impl RpcTrustModel {
    /// Creates an explicitly acknowledged trusted-provider model.
    ///
    /// The acknowledgment must name the identity and transport controls that
    /// make a single-provider response acceptable for this caller.
    #[must_use = "verify provider identity and transport integrity before trusting one RPC node"]
    pub const fn trusted_with_explicit_acknowledgment(acknowledgment: &'static str) -> Self {
        Self::Trusted { acknowledgment }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trusted_requires_explicit_acknowledgment() {
        assert_eq!(
            RpcTrustModel::trusted_with_explicit_acknowledgment("mTLS-pinned internal node"),
            RpcTrustModel::Trusted {
                acknowledgment: "mTLS-pinned internal node"
            }
        );
    }
}
