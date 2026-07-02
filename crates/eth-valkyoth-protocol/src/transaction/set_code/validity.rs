use core::fmt;

use eth_valkyoth_primitives::{Address, Gas, Nonce};

use super::{SetCodeAuthorization, SetCodeTransactionDecodeError, UnvalidatedSetCodeTransaction};
use crate::{Hardfork, ValidationContext};

#[path = "validity_authorization.rs"]
mod authorization_validity;

use authorization_validity::validate_authorizations;

/// EIP-7702 delegation indicator prefix bytes.
pub const EIP_7702_DELEGATION_INDICATOR_PREFIX: [u8; 3] = [0xef, 0x01, 0x00];

/// Validation context for EIP-7702 set-code transaction validity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SetCodeTransactionValidationContext {
    /// Active fork context for the block that would include this transaction.
    pub fork: ValidationContext,
    /// Caller-computed minimum gas limit, usually intrinsic gas.
    ///
    /// This crate does not bundle execution-state gas accounting. Supplying a
    /// value here lets callers bind their own gas calculation to the same
    /// transaction-validity gate.
    pub minimum_gas_limit: Option<Gas>,
}

/// Authority recovered for one authorization tuple.
///
/// Callers should create this value only from a successful
/// `eth-valkyoth-verify` authorization signature validation result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SetCodeAuthorizationAuthority {
    /// Index of the authorization tuple in the transaction authorization list.
    pub authorization_index: usize,
    /// Recovered authorizing account.
    pub authority: Address,
}

/// Caller-provided source of recovered authorization authorities.
///
/// The slice implementation is intended for tests and small fixtures. Production
/// callers with large authorization lists should use an indexed implementation
/// to avoid repeated linear scans.
pub trait SetCodeAuthorizationAuthorityView {
    /// Returns the recovered authority for `authorization` at
    /// `authorization_index`.
    fn authority_for(
        &self,
        authorization_index: usize,
        authorization: SetCodeAuthorization,
    ) -> Option<Address>;
}

impl SetCodeAuthorizationAuthorityView for [SetCodeAuthorizationAuthority] {
    fn authority_for(
        &self,
        authorization_index: usize,
        _authorization: SetCodeAuthorization,
    ) -> Option<Address> {
        self.iter()
            .find(|entry| entry.authorization_index == authorization_index)
            .map(|entry| entry.authority)
    }
}

/// Caller-reviewed authority account code state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SetCodeAuthorityCode {
    /// The authority account has no code.
    Empty,
    /// The authority account already contains an EIP-7702 delegation indicator.
    ///
    /// This crate does not inspect account bytecode or verify the
    /// [`EIP_7702_DELEGATION_INDICATOR_PREFIX`] itself. The `Delegation`
    /// classification is caller-trusted account-state input.
    Delegation {
        /// Delegation target encoded after `0xef0100`.
        target: Address,
    },
    /// The authority account contains non-delegation code.
    Other,
}

/// Caller-provided authority account state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SetCodeAuthorityAccount {
    /// Authority account address.
    pub authority: Address,
    /// Current authority account nonce.
    pub nonce: Nonce,
    /// Current authority account code classification.
    pub code: SetCodeAuthorityCode,
}

/// Caller-provided authority account-state view.
///
/// EIP-7702 treats an authority account that is absent from the state trie as an
/// empty account with nonce `0`. Implementations must synthesize
/// `SetCodeAuthorityAccount { nonce: Nonce::new(0), code: Empty, .. }` for that
/// case. Return `None` only when state is genuinely unavailable.
///
/// The slice implementation is intended for tests and small fixtures. Production
/// callers with large authorization lists should use an indexed implementation
/// to avoid repeated linear scans.
pub trait SetCodeAuthorityStateView {
    /// Returns account state for `authority`.
    fn authority_account(&self, authority: Address) -> Option<SetCodeAuthorityAccount>;
}

impl SetCodeAuthorityStateView for [SetCodeAuthorityAccount] {
    fn authority_account(&self, authority: Address) -> Option<SetCodeAuthorityAccount> {
        self.iter()
            .find(|entry| entry.authority == authority)
            .copied()
    }
}

/// EIP-7702 set-code transaction that passed context validity checks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidSetCodeTransaction<'a> {
    transaction: UnvalidatedSetCodeTransaction<'a>,
    authorization_count: usize,
    applied_authorization_count: usize,
    skipped_authorization_count: usize,
}

impl<'a> ValidSetCodeTransaction<'a> {
    const fn new(
        transaction: UnvalidatedSetCodeTransaction<'a>,
        authorization_count: usize,
        applied_authorization_count: usize,
        skipped_authorization_count: usize,
    ) -> Self {
        Self {
            transaction,
            authorization_count,
            applied_authorization_count,
            skipped_authorization_count,
        }
    }

    /// Returns the underlying decoded transaction.
    #[must_use]
    pub const fn transaction(self) -> UnvalidatedSetCodeTransaction<'a> {
        self.transaction
    }

    /// Returns the number of checked authorization tuples.
    #[must_use]
    pub const fn authorization_count(self) -> usize {
        self.authorization_count
    }

    /// Returns the number of authorization tuples that passed per-tuple checks.
    #[must_use]
    pub const fn applied_authorization_count(self) -> usize {
        self.applied_authorization_count
    }

    /// Returns the number of authorization tuples skipped by EIP-7702 rules.
    #[must_use]
    pub const fn skipped_authorization_count(self) -> usize {
        self.skipped_authorization_count
    }
}

/// EIP-7702 set-code transaction validity failure.
///
/// Per-authorization-tuple failures are exposed for diagnostics, but the
/// transaction validity gate does not return them as fatal errors. EIP-7702
/// skips invalid authorization tuples and continues processing later tuples.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SetCodeTransactionValidityError {
    /// The selected fork is inactive for this validation context.
    InactiveFork,
    /// Set-code transactions are not admitted before Prague/Pectra.
    ForkBeforePrague,
    /// The transaction chain ID does not match the validation context.
    WrongTransactionChain,
    /// The priority fee exceeds the max fee.
    PriorityFeeExceedsMaxFee,
    /// The gas limit is below the caller-computed minimum.
    GasLimitTooLow,
    /// EIP-7702 transactions must have at least one authorization tuple.
    EmptyAuthorizationList,
    /// A previously decoded authorization tuple failed to iterate.
    AuthorizationDecode(SetCodeTransactionDecodeError),
    /// No verified authority was supplied for this authorization tuple.
    MissingAuthorizationAuthority {
        /// Authorization tuple index.
        authorization_index: usize,
    },
    /// Authorization chain ID is neither universal nor the expected chain.
    WrongAuthorizationChain {
        /// Authorization tuple index.
        authorization_index: usize,
    },
    /// Authorization nonce is `u64::MAX` and cannot be incremented.
    AuthorizationNonceTooHigh {
        /// Authorization tuple index.
        authorization_index: usize,
    },
    /// No account state was supplied for the recovered authority.
    MissingAuthorityState {
        /// Authorization tuple index.
        authorization_index: usize,
    },
    /// Recovered authority nonce does not match the authorization nonce.
    AuthorityNonceMismatch {
        /// Authorization tuple index.
        authorization_index: usize,
    },
    /// Authority account code is neither empty nor an EIP-7702 delegation.
    InvalidAuthorityCode {
        /// Authorization tuple index.
        authorization_index: usize,
    },
}

impl SetCodeTransactionValidityError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::InactiveFork => "ETH_SET_CODE_VALIDITY_INACTIVE_FORK",
            Self::ForkBeforePrague => "ETH_SET_CODE_VALIDITY_PRE_PRAGUE",
            Self::WrongTransactionChain => "ETH_SET_CODE_VALIDITY_WRONG_TX_CHAIN",
            Self::PriorityFeeExceedsMaxFee => "ETH_SET_CODE_VALIDITY_FEE_ORDER",
            Self::GasLimitTooLow => "ETH_SET_CODE_VALIDITY_GAS_LIMIT",
            Self::EmptyAuthorizationList => "ETH_SET_CODE_VALIDITY_EMPTY_AUTH_LIST",
            Self::AuthorizationDecode(error) => error.code(),
            Self::MissingAuthorizationAuthority { .. } => "ETH_SET_CODE_VALIDITY_MISSING_AUTHORITY",
            Self::WrongAuthorizationChain { .. } => "ETH_SET_CODE_VALIDITY_WRONG_AUTH_CHAIN",
            Self::AuthorizationNonceTooHigh { .. } => "ETH_SET_CODE_VALIDITY_AUTH_NONCE_TOO_HIGH",
            Self::MissingAuthorityState { .. } => "ETH_SET_CODE_VALIDITY_MISSING_AUTHORITY_STATE",
            Self::AuthorityNonceMismatch { .. } => "ETH_SET_CODE_VALIDITY_AUTHORITY_NONCE_MISMATCH",
            Self::InvalidAuthorityCode { .. } => "ETH_SET_CODE_VALIDITY_INVALID_AUTHORITY_CODE",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::InactiveFork => "set-code transaction fork is not active",
            Self::ForkBeforePrague => "set-code transaction requires Prague/Pectra or later",
            Self::WrongTransactionChain => {
                "set-code transaction chain id does not match validation context"
            }
            Self::PriorityFeeExceedsMaxFee => "set-code transaction priority fee exceeds max fee",
            Self::GasLimitTooLow => "set-code transaction gas limit is below required minimum",
            Self::EmptyAuthorizationList => {
                "set-code transaction authorization list must not be empty"
            }
            Self::AuthorizationDecode(error) => error.message(),
            Self::MissingAuthorizationAuthority { .. } => {
                "set-code authorization has no verified recovered authority"
            }
            Self::WrongAuthorizationChain { .. } => {
                "set-code authorization chain id is not valid for this chain"
            }
            Self::AuthorizationNonceTooHigh { .. } => {
                "set-code authorization nonce is too high to increment"
            }
            Self::MissingAuthorityState { .. } => "set-code authority account state is unavailable",
            Self::AuthorityNonceMismatch { .. } => {
                "set-code authority nonce does not match authorization nonce"
            }
            Self::InvalidAuthorityCode { .. } => {
                "set-code authority code is neither empty nor delegated"
            }
        }
    }

    /// Stable high-level category for policy decisions.
    #[must_use]
    pub const fn category(self) -> SetCodeTransactionValidityErrorCategory {
        match self {
            Self::InactiveFork | Self::ForkBeforePrague | Self::WrongTransactionChain => {
                SetCodeTransactionValidityErrorCategory::Fork
            }
            Self::PriorityFeeExceedsMaxFee => SetCodeTransactionValidityErrorCategory::Fee,
            Self::GasLimitTooLow => SetCodeTransactionValidityErrorCategory::Gas,
            Self::EmptyAuthorizationList
            | Self::AuthorizationDecode(_)
            | Self::MissingAuthorizationAuthority { .. }
            | Self::WrongAuthorizationChain { .. }
            | Self::AuthorizationNonceTooHigh { .. } => {
                SetCodeTransactionValidityErrorCategory::Authorization
            }
            Self::MissingAuthorityState { .. }
            | Self::AuthorityNonceMismatch { .. }
            | Self::InvalidAuthorityCode { .. } => {
                SetCodeTransactionValidityErrorCategory::AccountState
            }
        }
    }
}

impl fmt::Display for SetCodeTransactionValidityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SetCodeTransactionValidityError {}

/// Stable high-level set-code transaction validity categories.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SetCodeTransactionValidityErrorCategory {
    /// Fork or chain context rejected the transaction.
    Fork,
    /// Fee policy rejected the transaction.
    Fee,
    /// Gas policy rejected the transaction.
    Gas,
    /// Authorization-list policy rejected the transaction.
    Authorization,
    /// Authority account-state policy rejected the transaction.
    AccountState,
}

/// Validates EIP-7702 context rules for a decoded set-code transaction.
///
/// This is the non-cryptographic validity gate. It expects `authorities` to be
/// produced by validating every authorization tuple signature through the
/// verify crate. It does not recover signatures itself and does not execute the
/// transaction. Invalid authorization tuples are counted as skipped instead of
/// rejecting the whole transaction, matching EIP-7702 tuple processing.
pub fn validate_set_code_transaction_context<'a, A, S>(
    transaction: UnvalidatedSetCodeTransaction<'a>,
    context: SetCodeTransactionValidationContext,
    authorities: &A,
    accounts: &S,
) -> Result<ValidSetCodeTransaction<'a>, SetCodeTransactionValidityError>
where
    A: SetCodeAuthorizationAuthorityView + ?Sized,
    S: SetCodeAuthorityStateView + ?Sized,
{
    validate_outer_context(&transaction, context)?;
    let summary = validate_authorizations(transaction, authorities, accounts);
    Ok(ValidSetCodeTransaction::new(
        transaction,
        transaction.authorization_list.len(),
        summary.applied,
        summary.skipped,
    ))
}

fn validate_outer_context(
    transaction: &UnvalidatedSetCodeTransaction<'_>,
    context: SetCodeTransactionValidationContext,
) -> Result<(), SetCodeTransactionValidityError> {
    if !context.fork.fork_is_active() {
        return Err(SetCodeTransactionValidityError::InactiveFork);
    }
    if context.fork.fork.hardfork < Hardfork::Prague {
        return Err(SetCodeTransactionValidityError::ForkBeforePrague);
    }
    if context.fork.fork.chain_id != transaction.chain_id {
        return Err(SetCodeTransactionValidityError::WrongTransactionChain);
    }
    if wei_exceeds(
        transaction.max_priority_fee_per_gas,
        transaction.max_fee_per_gas,
    ) {
        return Err(SetCodeTransactionValidityError::PriorityFeeExceedsMaxFee);
    }
    if let Some(minimum_gas_limit) = context.minimum_gas_limit
        && transaction.gas_limit < minimum_gas_limit
    {
        return Err(SetCodeTransactionValidityError::GasLimitTooLow);
    }
    if transaction.authorization_list.is_empty() {
        return Err(SetCodeTransactionValidityError::EmptyAuthorizationList);
    }
    Ok(())
}

fn wei_exceeds(left: eth_valkyoth_primitives::Wei, right: eth_valkyoth_primitives::Wei) -> bool {
    left.to_be_bytes() > right.to_be_bytes()
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct AuthorizationValidationSummary {
    pub(super) applied: usize,
    pub(super) skipped: usize,
}

#[cfg(test)]
#[path = "validity_tests.rs"]
mod tests;
