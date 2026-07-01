use core::marker::PhantomData;

use crate::ProtocolError;

/// Raw wire input was accepted by the codec.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Decoded;

/// Canonical wire form and type-specific structure were checked.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Canonical;

/// Fork-specific validity was checked.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ForkValidated;

/// Sender recovery succeeded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SenderRecovered;

/// Proof that canonical transaction structure was checked.
///
/// This proof is intentionally not publicly constructible yet. Public
/// constructors will be added only with validators that can prove the state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanonicalValidationProof {
    _private: (),
}

impl CanonicalValidationProof {
    #[must_use]
    #[cfg(test)]
    pub(crate) const fn new() -> Self {
        Self { _private: () }
    }
}

/// Proof that fork-specific transaction validity was checked.
///
/// This proof is intentionally not publicly constructible yet. Public
/// constructors will be added only with validators that can prove the state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ForkValidationProof {
    _private: (),
}

impl ForkValidationProof {
    #[must_use]
    #[cfg(test)]
    pub(crate) const fn new() -> Self {
        Self { _private: () }
    }
}

/// Proof that sender recovery succeeded.
///
/// This proof is intentionally not publicly constructible yet. Public
/// constructors will be added only with validators that can prove the state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SenderRecoveryProof {
    _private: (),
}

impl SenderRecoveryProof {
    #[must_use]
    #[cfg(test)]
    pub(crate) const fn new() -> Self {
        Self { _private: () }
    }
}

/// A transaction token whose validation state is tracked at compile time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Transaction<State> {
    _state: PhantomData<State>,
}

impl<State> Transaction<State> {
    const fn new() -> Self {
        Self {
            _state: PhantomData,
        }
    }
}

impl Transaction<Decoded> {
    /// Creates a token for a decoded transaction in internal tests.
    ///
    /// A public decoded-transaction entry point will be added only together
    /// with the real codec output that proves the decoded state.
    #[must_use]
    #[cfg(test)]
    pub(crate) const fn decoded() -> Self {
        Self::new()
    }

    /// Advances to canonical form after canonical checks pass.
    pub fn try_into_canonical(
        &self,
        proof: Result<CanonicalValidationProof, ProtocolError>,
    ) -> Result<Transaction<Canonical>, ProtocolError> {
        proof?;
        Ok(Transaction::new())
    }
}

impl Transaction<Canonical> {
    /// Advances after fork-specific validation passes.
    pub fn try_into_fork_validated(
        &self,
        proof: Result<ForkValidationProof, ProtocolError>,
    ) -> Result<Transaction<ForkValidated>, ProtocolError> {
        proof?;
        Ok(Transaction::new())
    }
}

impl Transaction<ForkValidated> {
    /// Advances after sender recovery succeeds.
    pub fn try_into_sender_recovered(
        &self,
        proof: Result<SenderRecoveryProof, ProtocolError>,
    ) -> Result<Transaction<SenderRecovered>, ProtocolError> {
        proof?;
        Ok(Transaction::new())
    }
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
