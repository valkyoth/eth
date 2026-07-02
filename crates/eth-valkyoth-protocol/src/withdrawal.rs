use eth_valkyoth_codec::{DecodeError, DecodeLimits, RlpInteger, RlpItem, RlpList, RlpScalar};
use eth_valkyoth_primitives::Address;

mod error;
pub use error::{WithdrawalDecodeError, WithdrawalDecodeErrorCategory, WithdrawalField};

const ADDRESS_BYTES: usize = 20;

/// Number of fields in an EIP-4895 withdrawal entry.
pub const WITHDRAWAL_FIELD_COUNT: usize = 4;

/// EIP-4895 global withdrawal index.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WithdrawalIndex(u64);

impl WithdrawalIndex {
    /// Creates a withdrawal index.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw integer value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Consensus-layer validator index referenced by a withdrawal.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WithdrawalValidatorIndex(u64);

impl WithdrawalValidatorIndex {
    /// Creates a validator index.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw integer value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// EIP-4895 withdrawal amount in Gwei.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WithdrawalAmountGwei(u64);

impl WithdrawalAmountGwei {
    /// Creates a nonzero Gwei amount.
    pub const fn try_new(value: u64) -> Result<Self, WithdrawalDecodeError> {
        if value == 0 {
            return Err(WithdrawalDecodeError::ZeroAmount);
        }
        Ok(Self(value))
    }

    /// Returns the raw Gwei amount.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Borrowed withdrawals list decoded only into EIP-4895 field domains.
///
/// This type is intentionally unvalidated. It does not prove consensus-layer
/// dequeue correctness, global index monotonicity, header `withdrawals_root`
/// matching, trie-root membership, or state-balance application.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnvalidatedWithdrawals<'a> {
    encoded_rlp: &'a [u8],
    list: RlpList<'a>,
}

impl<'a> UnvalidatedWithdrawals<'a> {
    /// Returns the exact canonical RLP list bytes that were decoded.
    #[must_use]
    pub const fn encoded_rlp(self) -> &'a [u8] {
        self.encoded_rlp
    }

    /// Returns the number of withdrawal entries.
    #[must_use]
    pub const fn len(self) -> usize {
        self.list.item_count()
    }

    /// Returns true when the withdrawals list is empty.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.list.is_empty()
    }

    /// Returns an iterator over decoded withdrawal entries.
    #[must_use]
    pub const fn entries(self) -> WithdrawalItems<'a> {
        WithdrawalItems {
            items: self.list.items(),
        }
    }
}

/// Unvalidated EIP-4895 withdrawal entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnvalidatedWithdrawal {
    /// Global withdrawal index.
    pub index: WithdrawalIndex,
    /// Consensus-layer validator index.
    pub validator_index: WithdrawalValidatorIndex,
    /// Recipient execution-layer address.
    pub address: Address,
    /// Nonzero amount in Gwei.
    pub amount: WithdrawalAmountGwei,
}

/// Iterator over borrowed withdrawal entries.
#[derive(Clone, Debug)]
pub struct WithdrawalItems<'a> {
    items: eth_valkyoth_codec::RlpListItems<'a>,
}

impl Iterator for WithdrawalItems<'_> {
    type Item = Result<UnvalidatedWithdrawal, WithdrawalDecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.next().map(decode_withdrawal_item)
    }
}

impl core::iter::FusedIterator for WithdrawalItems<'_> {}

/// Decodes an EIP-4895 withdrawals list under explicit limits.
///
/// This function is syntactic. It checks that the input is one canonical RLP
/// list of withdrawal entries shaped `[index, validator_index, address,
/// amount]`, with canonical `uint64` integers, a 20-byte address, and a
/// nonzero Gwei amount. It does not compute or compare `withdrawals_root`.
pub fn decode_withdrawals<'a>(
    input: &'a [u8],
    limits: DecodeLimits,
) -> Result<UnvalidatedWithdrawals<'a>, WithdrawalDecodeError> {
    let list = eth_valkyoth_codec::decode_rlp_list(input, limits)
        .map_err(|source| field_error(WithdrawalField::List, source))?;
    for item in list.items() {
        let _ = decode_withdrawal_item(item)?;
    }
    Ok(UnvalidatedWithdrawals {
        encoded_rlp: input,
        list,
    })
}

fn decode_withdrawal_item(
    item: Result<RlpItem<'_>, DecodeError>,
) -> Result<UnvalidatedWithdrawal, WithdrawalDecodeError> {
    let item = item.map_err(|source| field_error(WithdrawalField::Withdrawal, source))?;
    let RlpItem::List(list) = item else {
        return Err(field_error(
            WithdrawalField::Withdrawal,
            DecodeError::UnexpectedScalar,
        ));
    };
    if list.item_count() != WITHDRAWAL_FIELD_COUNT {
        return Err(WithdrawalDecodeError::WrongFieldCount {
            expected: WITHDRAWAL_FIELD_COUNT,
            found: list.item_count(),
        });
    }

    let mut fields = list.items();
    Ok(UnvalidatedWithdrawal {
        index: WithdrawalIndex::new(decode_u64(&mut fields, WithdrawalField::Index)?),
        validator_index: WithdrawalValidatorIndex::new(decode_u64(
            &mut fields,
            WithdrawalField::ValidatorIndex,
        )?),
        address: Address::from_bytes(decode_address(&mut fields)?),
        amount: WithdrawalAmountGwei::try_new(decode_u64(&mut fields, WithdrawalField::Amount)?)?,
    })
}

fn decode_u64<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: WithdrawalField,
) -> Result<u64, WithdrawalDecodeError> {
    RlpInteger::try_from_scalar(next_scalar(fields, field)?)
        .and_then(RlpInteger::to_u64)
        .map_err(|source| field_error(field, source))
}

fn decode_address<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
) -> Result<[u8; ADDRESS_BYTES], WithdrawalDecodeError> {
    let scalar = next_scalar(fields, WithdrawalField::Address)?;
    let found = scalar.payload().len();
    scalar
        .payload()
        .try_into()
        .map_err(|_| WithdrawalDecodeError::InvalidAddressLength { found })
}

fn next_scalar<'a>(
    fields: &mut impl Iterator<Item = Result<RlpItem<'a>, DecodeError>>,
    field: WithdrawalField,
) -> Result<RlpScalar<'a>, WithdrawalDecodeError> {
    let item = fields
        .next()
        .ok_or(field_error(field, DecodeError::Malformed))?
        .map_err(|source| field_error(field, source))?;
    match item {
        RlpItem::Scalar(scalar) => Ok(scalar),
        RlpItem::List(_) => Err(field_error(field, DecodeError::UnexpectedList)),
    }
}

const fn field_error(field: WithdrawalField, source: DecodeError) -> WithdrawalDecodeError {
    WithdrawalDecodeError::FieldDecode { field, source }
}

#[cfg(test)]
#[path = "withdrawal_tests.rs"]
mod tests;
