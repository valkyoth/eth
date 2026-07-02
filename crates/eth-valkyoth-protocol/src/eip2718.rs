use eth_valkyoth_primitives::TransactionType;

/// EIP-2718 byte value reserved by this crate for the legacy domain.
pub(crate) const EIP_2718_TYPED_ZERO_PREFIX: u8 = 0x00;
/// Largest single-byte EIP-2718 typed prefix.
pub(crate) const EIP_2718_MAX_TYPED_PREFIX: u8 = TransactionType::MAX_TYPED;
/// First RLP scalar prefix that cannot be a typed envelope or a legacy list.
pub(crate) const EIP_2718_SCALAR_PREFIX_START: u8 = 0x80;
/// First byte used by canonical RLP short-list legacy envelopes.
pub(crate) const LEGACY_PREFIX_START: u8 = 0xc0;
/// Prefix reserved by EIP-2718 as a future extension sentinel.
pub(crate) const EIP_2718_RESERVED_PREFIX: u8 = 0xff;

/// Shared EIP-2718 first-byte classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Eip2718Prefix<'a> {
    /// Legacy-zero typed prefix.
    TypedZero,
    /// Typed envelope payload.
    Typed { type_byte: u8, payload: &'a [u8] },
    /// RLP scalar prefix.
    ScalarPrefix { prefix: u8 },
    /// Legacy RLP list envelope.
    Legacy,
    /// Reserved extension prefix.
    Reserved,
}

/// Classifies the first byte according to EIP-2718 shared envelope rules.
#[must_use]
pub(crate) fn classify_eip2718_prefix(input: &[u8]) -> Option<Eip2718Prefix<'_>> {
    let (&prefix, payload) = input.split_first()?;
    let classified = match prefix {
        EIP_2718_TYPED_ZERO_PREFIX => Eip2718Prefix::TypedZero,
        0x01..=EIP_2718_MAX_TYPED_PREFIX => Eip2718Prefix::Typed {
            type_byte: prefix,
            payload,
        },
        EIP_2718_SCALAR_PREFIX_START..=0xbf => Eip2718Prefix::ScalarPrefix { prefix },
        LEGACY_PREFIX_START..=0xfe => Eip2718Prefix::Legacy,
        EIP_2718_RESERVED_PREFIX => Eip2718Prefix::Reserved,
    };
    Some(classified)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_all_prefix_boundaries() {
        assert_eq!(classify(&[0x00]), Some(Eip2718Prefix::TypedZero));
        assert_eq!(
            classify(&[0x01, 0xaa]),
            Some(Eip2718Prefix::Typed {
                type_byte: 0x01,
                payload: &[0xaa]
            })
        );
        assert_eq!(
            classify(&[0x7f]),
            Some(Eip2718Prefix::Typed {
                type_byte: 0x7f,
                payload: &[]
            })
        );
        assert_eq!(
            classify(&[0x80]),
            Some(Eip2718Prefix::ScalarPrefix { prefix: 0x80 })
        );
        assert_eq!(
            classify(&[0xbf]),
            Some(Eip2718Prefix::ScalarPrefix { prefix: 0xbf })
        );
        assert_eq!(classify(&[0xc0]), Some(Eip2718Prefix::Legacy));
        assert_eq!(classify(&[0xfe]), Some(Eip2718Prefix::Legacy));
        assert_eq!(classify(&[0xff]), Some(Eip2718Prefix::Reserved));
        assert_eq!(classify(&[]), None);
    }

    fn classify(input: &[u8]) -> Option<Eip2718Prefix<'_>> {
        classify_eip2718_prefix(input)
    }
}
