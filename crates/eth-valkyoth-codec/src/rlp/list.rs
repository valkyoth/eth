use crate::{
    DecodeAccumulator, DecodeError, DecodeLimits, checked_len_add, require_exact_consumption,
};

use super::{
    LONG_LIST_OFFSET, RlpScalar, SHORT_LIST_OFFSET, SHORT_STRING_LIMIT, parse_payload_len, payload,
};

mod item;
use item::{ParsedItemKind, parse_item};

/// Hard cap on RLP list traversal depth regardless of the active decode limits.
///
/// Inputs nested deeper than this return [`DecodeError::NestingTooDeep`] even
/// when [`DecodeLimits::max_nesting_depth`] is higher.
pub const MAX_RLP_LIST_TRAVERSAL_DEPTH: usize = 128;

/// Canonical RLP list form used by the decoder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RlpListForm {
    /// A list with a one-byte RLP header.
    ShortList,
    /// A list with a length-of-length RLP header.
    LongList,
}

/// Borrowed RLP list payload.
///
/// Fields are private so downstream crates cannot construct unvalidated
/// decoded values and feed them into trusted re-encoding paths.
#[derive(Clone, Copy, Debug)]
pub struct RlpList<'a> {
    payload: &'a [u8],
    encoded_len: usize,
    header_len: usize,
    item_count: usize,
    form: RlpListForm,
    limits: DecodeLimits,
    iteration_depth_remaining: usize,
}

impl<'a> RlpList<'a> {
    /// Returns the concatenated encoded child items.
    #[must_use]
    pub const fn payload(self) -> &'a [u8] {
        self.payload
    }

    /// Returns the total encoded list length in bytes.
    #[must_use]
    pub const fn encoded_len(self) -> usize {
        self.encoded_len
    }

    /// Returns the RLP header length in bytes.
    #[must_use]
    pub const fn header_len(self) -> usize {
        self.header_len
    }

    /// Returns the number of immediate child items.
    #[must_use]
    pub const fn item_count(self) -> usize {
        self.item_count
    }

    /// Returns the canonical list form that was decoded.
    #[must_use]
    pub const fn form(self) -> RlpListForm {
        self.form
    }

    /// Returns true when the list has no child items.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.item_count == 0
    }

    /// Returns an iterator over the immediate child items in this list.
    ///
    /// Nested child list item counts are computed lazily when those child lists
    /// are yielded. Recursive callers that walk every nested list can therefore
    /// perform additional bounded traversal work proportional to the walked
    /// nesting depth. Use [`Self::items_with_depth_limit`] when processing
    /// externally reachable input with a caller-specific recursion budget.
    #[must_use]
    pub const fn items(self) -> RlpListItems<'a> {
        self.items_with_depth_limit(self.iteration_depth_remaining)
    }

    /// Returns an iterator that rejects nested lists after `depth` child-list
    /// transitions below this list.
    ///
    /// A `depth` of `0` still permits scalar children but yields
    /// [`DecodeError::NestingTooDeep`] for a child list. This gives recursive
    /// callers an explicit iteration-phase budget independent from decode-time
    /// validation.
    #[must_use]
    pub const fn items_with_depth_limit(self, depth: usize) -> RlpListItems<'a> {
        RlpListItems {
            input: self.payload,
            cursor: 0,
            remaining: self.item_count,
            limits: self.limits,
            depth_remaining: depth,
        }
    }
}

impl PartialEq for RlpList<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.payload == other.payload
            && self.encoded_len == other.encoded_len
            && self.header_len == other.header_len
            && self.item_count == other.item_count
            && self.form == other.form
    }
}

impl Eq for RlpList<'_> {}

/// Borrowed RLP item yielded by [`RlpListItems`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RlpItem<'a> {
    /// A scalar byte string item.
    Scalar(RlpScalar<'a>),
    /// A list item.
    List(RlpList<'a>),
}

impl<'a> RlpItem<'a> {
    /// Returns the total encoded item length in bytes.
    #[must_use]
    pub const fn encoded_len(self) -> usize {
        match self {
            Self::Scalar(scalar) => scalar.encoded_len(),
            Self::List(list) => list.encoded_len(),
        }
    }

    /// Returns the RLP header length in bytes.
    #[must_use]
    pub const fn header_len(self) -> usize {
        match self {
            Self::Scalar(scalar) => scalar.header_len(),
            Self::List(list) => list.header_len(),
        }
    }

    /// Returns true when this item is a scalar byte string.
    #[must_use]
    pub const fn is_scalar(self) -> bool {
        matches!(self, Self::Scalar(_))
    }

    /// Returns true when this item is a list.
    #[must_use]
    pub const fn is_list(self) -> bool {
        matches!(self, Self::List(_))
    }

    /// Returns the scalar item, if this item is a scalar byte string.
    #[must_use]
    pub const fn as_scalar(&self) -> Option<RlpScalar<'a>> {
        match self {
            Self::Scalar(scalar) => Some(*scalar),
            Self::List(_) => None,
        }
    }

    /// Returns the list item, if this item is a list.
    #[must_use]
    pub const fn as_list(&self) -> Option<RlpList<'a>> {
        match self {
            Self::Scalar(_) => None,
            Self::List(list) => Some(*list),
        }
    }
}

/// Iterator over immediate child items in a decoded RLP list.
#[derive(Clone, Debug)]
pub struct RlpListItems<'a> {
    input: &'a [u8],
    cursor: usize,
    remaining: usize,
    limits: DecodeLimits,
    depth_remaining: usize,
}

impl<'a> RlpListItems<'a> {
    /// Returns the number of child items not yielded yet.
    #[must_use]
    pub const fn remaining(&self) -> usize {
        self.remaining
    }

    /// Builds a list-item iterator for tests that need malformed internals.
    #[cfg(test)]
    pub(super) const fn for_test(input: &'a [u8], remaining: usize, limits: DecodeLimits) -> Self {
        Self {
            input,
            cursor: 0,
            remaining,
            limits,
            depth_remaining: 0,
        }
    }
}

impl<'a> Iterator for RlpListItems<'a> {
    type Item = Result<RlpItem<'a>, DecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        let result = parse_item(self.input, self.cursor, self.input.len())
            .and_then(|item| {
                item.into_rlp_item(self.input, self.cursor, self.limits, self.depth_remaining)
            })
            .map(|(item, next_cursor)| {
                self.cursor = next_cursor;
                item
            });
        self.remaining = self.remaining.saturating_sub(1);

        match result {
            Ok(item) => Some(Ok(item)),
            Err(error) => {
                self.remaining = 0;
                Some(Err(error))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.remaining))
    }
}

impl core::iter::FusedIterator for RlpListItems<'_> {}

/// Decodes exactly one canonical RLP list.
pub fn decode_rlp_list<'a>(
    input: &'a [u8],
    limits: DecodeLimits,
) -> Result<RlpList<'a>, DecodeError> {
    let mut accumulator = limits.accumulator();
    let list = decode_rlp_list_partial(input, &mut accumulator)?;
    require_exact_consumption(list.encoded_len, input.len())?;
    Ok(list)
}

/// Decodes one canonical RLP list from the start of `input`.
///
/// Warning: this intentionally accepts trailing bytes. Use [`decode_rlp_list`]
/// when the full input must be consumed.
///
/// The input-length budget check applies to the full `input` slice, not only
/// the consumed list bytes. Callers that decode from a larger outer buffer must
/// pre-slice before calling this helper.
pub fn decode_rlp_list_partial<'a>(
    input: &'a [u8],
    accumulator: &mut DecodeAccumulator,
) -> Result<RlpList<'a>, DecodeError> {
    accumulator.check_input_len(input.len())?;
    accumulator.account_items(1)?;
    accumulator.check_nesting_depth(1)?;

    let prefix = *input.first().ok_or(DecodeError::Malformed)?;
    let list = match prefix {
        SHORT_LIST_OFFSET..=LONG_LIST_OFFSET => {
            decode_short_list(input, prefix, accumulator.limits())?
        }
        0xf8..=0xff => decode_long_list(input, prefix, accumulator.limits())?,
        0x00..=0xbf => return Err(DecodeError::UnexpectedScalar),
    };
    let item_count = validate_list_payload(list.payload, accumulator)?;
    accumulator.check_list_count(item_count)?;
    Ok(RlpList {
        item_count,
        iteration_depth_remaining: MAX_RLP_LIST_TRAVERSAL_DEPTH.saturating_sub(1),
        ..list
    })
}

fn decode_short_list(
    input: &[u8],
    prefix: u8,
    limits: DecodeLimits,
) -> Result<RlpList<'_>, DecodeError> {
    let payload_len = usize::from(prefix.saturating_sub(SHORT_LIST_OFFSET));
    let payload = payload(input, 1, payload_len)?;
    Ok(RlpList {
        payload,
        encoded_len: checked_len_add(1, payload_len)?,
        header_len: 1,
        item_count: 0,
        form: RlpListForm::ShortList,
        limits,
        iteration_depth_remaining: 0,
    })
}

fn decode_long_list(
    input: &[u8],
    prefix: u8,
    limits: DecodeLimits,
) -> Result<RlpList<'_>, DecodeError> {
    let len_of_len = usize::from(prefix.saturating_sub(LONG_LIST_OFFSET));
    let payload_len = parse_payload_len(input, 1, len_of_len)?;
    if payload_len <= SHORT_STRING_LIMIT {
        return Err(DecodeError::Malformed);
    }
    let header_len = checked_len_add(1, len_of_len)?;
    let payload = payload(input, header_len, payload_len)?;
    Ok(RlpList {
        payload,
        encoded_len: checked_len_add(header_len, payload_len)?,
        header_len,
        item_count: 0,
        form: RlpListForm::LongList,
        limits,
        iteration_depth_remaining: 0,
    })
}

#[derive(Clone, Copy)]
struct ListFrame {
    end: usize,
    count: usize,
}

pub(super) fn validate_list_payload(
    input: &[u8],
    accumulator: &mut DecodeAccumulator,
) -> Result<usize, DecodeError> {
    let mut stack = [ListFrame { end: 0, count: 0 }; MAX_RLP_LIST_TRAVERSAL_DEPTH];
    let root = stack.get_mut(0).ok_or(DecodeError::NestingTooDeep)?;
    *root = ListFrame {
        end: input.len(),
        count: 0,
    };
    let mut depth = 1usize;
    let mut cursor = 0usize;

    loop {
        let frame_index = depth.checked_sub(1).ok_or(DecodeError::LengthOverflow)?;
        let frame = stack
            .get_mut(frame_index)
            .ok_or(DecodeError::NestingTooDeep)?;

        if cursor == frame.end {
            let finished_count = frame.count;
            accumulator.check_list_count(finished_count)?;
            depth = depth.checked_sub(1).ok_or(DecodeError::LengthOverflow)?;
            if depth == 0 {
                return Ok(finished_count);
            }
            continue;
        }
        if cursor > frame.end {
            return Err(DecodeError::OffsetOutOfBounds);
        }

        frame.count = checked_len_add(frame.count, 1)?;
        accumulator.check_list_count(frame.count)?;

        let item = parse_item(input, cursor, frame.end)?;
        accumulator.account_items(1)?;
        if matches!(item.kind, ParsedItemKind::List(_)) {
            let next_depth = checked_len_add(depth, 1)?;
            accumulator.check_nesting_depth(next_depth)?;
            if next_depth > MAX_RLP_LIST_TRAVERSAL_DEPTH {
                return Err(DecodeError::NestingTooDeep);
            }
            let child_index = next_depth
                .checked_sub(1)
                .ok_or(DecodeError::LengthOverflow)?;
            let child = stack
                .get_mut(child_index)
                .ok_or(DecodeError::NestingTooDeep)?;
            *child = ListFrame {
                end: item.payload_end,
                count: 0,
            };
            depth = next_depth;
            cursor = item.payload_start;
        } else {
            cursor = item.item_end;
        }
    }
}

fn count_immediate_items(input: &[u8], limits: DecodeLimits) -> Result<usize, DecodeError> {
    // Iteration-phase recounting is bounded independently from the
    // decode-phase accumulator used by partial streaming callers.
    let mut accumulator = limits.accumulator();
    accumulator.check_input_len(input.len())?;
    let mut count = 0usize;
    let mut cursor = 0usize;
    while cursor < input.len() {
        let item = parse_item(input, cursor, input.len())?;
        count = checked_len_add(count, 1)?;
        accumulator.account_items(1)?;
        accumulator.check_list_count(count)?;
        cursor = item.item_end;
    }
    Ok(count)
}
