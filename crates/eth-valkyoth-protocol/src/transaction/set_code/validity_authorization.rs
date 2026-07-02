use eth_valkyoth_primitives::{Address, ChainId, Nonce};

use super::{
    AuthorizationValidationSummary, SetCodeAuthorityAccount, SetCodeAuthorityCode,
    SetCodeAuthorityStateView, SetCodeAuthorization, SetCodeAuthorizationAuthorityView,
    SetCodeTransactionValidityError, UnvalidatedSetCodeTransaction,
};

pub(super) fn validate_authorizations<A, S>(
    transaction: UnvalidatedSetCodeTransaction<'_>,
    authorities: &A,
    accounts: &S,
) -> AuthorizationValidationSummary
where
    A: SetCodeAuthorizationAuthorityView + ?Sized,
    S: SetCodeAuthorityStateView + ?Sized,
{
    let mut summary = AuthorizationValidationSummary::default();
    for (authorization_index, authorization) in
        transaction.authorization_list.authorizations().enumerate()
    {
        let Ok(authorization) = authorization else {
            summary.skipped = summary.skipped.saturating_add(1);
            continue;
        };
        if validate_authorization(
            transaction,
            authorization_index,
            authorization,
            authorities,
            accounts,
        )
        .is_ok()
        {
            summary.applied = summary.applied.saturating_add(1);
        } else {
            summary.skipped = summary.skipped.saturating_add(1);
        }
    }
    summary
}

fn validate_authorization<A, S>(
    transaction: UnvalidatedSetCodeTransaction<'_>,
    authorization_index: usize,
    authorization: SetCodeAuthorization,
    authorities: &A,
    accounts: &S,
) -> Result<(), SetCodeTransactionValidityError>
where
    A: SetCodeAuthorizationAuthorityView + ?Sized,
    S: SetCodeAuthorityStateView + ?Sized,
{
    if !authorization_chain_permits(authorization, transaction.chain_id) {
        return Err(SetCodeTransactionValidityError::WrongAuthorizationChain {
            authorization_index,
        });
    }
    if authorization.nonce.get() == u64::MAX {
        return Err(SetCodeTransactionValidityError::AuthorizationNonceTooHigh {
            authorization_index,
        });
    }
    let authority = authorities
        .authority_for(authorization_index, authorization)
        .ok_or(
            SetCodeTransactionValidityError::MissingAuthorizationAuthority {
                authorization_index,
            },
        )?;
    let account = accounts.authority_account(authority).ok_or(
        SetCodeTransactionValidityError::MissingAuthorityState {
            authorization_index,
        },
    )?;
    if !authority_code_can_apply(account.code) {
        return Err(SetCodeTransactionValidityError::InvalidAuthorityCode {
            authorization_index,
        });
    }
    let prior = count_prior_applications(
        transaction,
        authorization_index,
        authorities,
        authority,
        account,
    )?;
    if checked_authority_nonce(account.nonce, prior, authorization_index)? != authorization.nonce {
        return Err(SetCodeTransactionValidityError::AuthorityNonceMismatch {
            authorization_index,
        });
    }
    Ok(())
}

fn count_prior_applications<A>(
    transaction: UnvalidatedSetCodeTransaction<'_>,
    authorization_index: usize,
    authorities: &A,
    authority: Address,
    account: SetCodeAuthorityAccount,
) -> Result<u64, SetCodeTransactionValidityError>
where
    A: SetCodeAuthorizationAuthorityView + ?Sized,
{
    let mut applied = 0_u64;
    for (earlier_index, earlier) in transaction
        .authorization_list
        .authorizations()
        .take(authorization_index)
        .enumerate()
    {
        let Ok(earlier) = earlier else {
            continue;
        };
        if !prior_authorization_can_apply(
            transaction.chain_id,
            earlier_index,
            earlier,
            authorities,
            authority,
        ) {
            continue;
        }
        if checked_authority_nonce(account.nonce, applied, authorization_index)? == earlier.nonce {
            applied = applied.checked_add(1).ok_or(
                SetCodeTransactionValidityError::AuthorizationNonceTooHigh {
                    authorization_index,
                },
            )?;
        }
    }
    Ok(applied)
}

fn prior_authorization_can_apply<A>(
    chain_id: ChainId,
    authorization_index: usize,
    authorization: SetCodeAuthorization,
    authorities: &A,
    authority: Address,
) -> bool
where
    A: SetCodeAuthorizationAuthorityView + ?Sized,
{
    authorization.nonce.get() != u64::MAX
        && authorization_chain_permits(authorization, chain_id)
        && authorities.authority_for(authorization_index, authorization) == Some(authority)
}

fn checked_authority_nonce(
    base: Nonce,
    offset: u64,
    authorization_index: usize,
) -> Result<Nonce, SetCodeTransactionValidityError> {
    base.get().checked_add(offset).map(Nonce::new).ok_or(
        SetCodeTransactionValidityError::AuthorizationNonceTooHigh {
            authorization_index,
        },
    )
}

fn authority_code_can_apply(code: SetCodeAuthorityCode) -> bool {
    matches!(
        code,
        SetCodeAuthorityCode::Empty | SetCodeAuthorityCode::Delegation { .. }
    )
}

fn authorization_chain_permits(authorization: SetCodeAuthorization, chain_id: ChainId) -> bool {
    if authorization.chain_id.is_universal() {
        return true;
    }
    let bytes = authorization.chain_id.to_be_bytes();
    let Some(prefix) = bytes.get(..24) else {
        return false;
    };
    let Some(suffix) = bytes.get(24..) else {
        return false;
    };
    prefix.iter().all(|byte| *byte == 0) && suffix == chain_id.get().to_be_bytes()
}
