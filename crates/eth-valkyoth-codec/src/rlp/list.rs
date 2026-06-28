use crate::{
    DecodeAccumulator, DecodeError, DecodeLimits, checked_len_add, require_exact_consumption,
    require_range_in_bounds,
};

use super::{
    LONG_LIST_OFFSET, SHORT_LIST_OFFSET, SHORT_STRING_LIMIT, decode_long_string,
    decode_short_string, parse_payload_len, payload,
};

const MAX_TRAVERSAL_STACK: usize = 128;

/// Canonical RLP list form used by the decoder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RlpListForm {
    /// A list with a one-byte RLP header.
    ShortList,
    /// A list with a length-of-length RLP header.
    LongList,
}

/// Borrowed RLP list payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RlpList<'a> {
    payload: &'a [u8],
    encoded_len: usize,
    header_len: usize,
    item_count: usize,
    form: RlpListForm,
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
}

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
pub fn decode_rlp_list_partial<'a>(
    input: &'a [u8],
    accumulator: &mut DecodeAccumulator,
) -> Result<RlpList<'a>, DecodeError> {
    accumulator.check_input_len(input.len())?;
    accumulator.account_items(1)?;
    accumulator.check_nesting_depth(1)?;

    let prefix = *input.first().ok_or(DecodeError::Malformed)?;
    let list = match prefix {
        SHORT_LIST_OFFSET..=LONG_LIST_OFFSET => decode_short_list(input, prefix)?,
        0xf8..=0xff => decode_long_list(input, prefix)?,
        0x00..=0xbf => return Err(DecodeError::UnexpectedScalar),
    };
    let item_count = validate_list_payload(list.payload, accumulator)?;
    accumulator.check_list_count(item_count)?;
    Ok(RlpList { item_count, ..list })
}

fn decode_short_list(input: &[u8], prefix: u8) -> Result<RlpList<'_>, DecodeError> {
    let payload_len = usize::from(prefix.saturating_sub(SHORT_LIST_OFFSET));
    let payload = payload(input, 1, payload_len)?;
    Ok(RlpList {
        payload,
        encoded_len: checked_len_add(1, payload_len)?,
        header_len: 1,
        item_count: 0,
        form: RlpListForm::ShortList,
    })
}

fn decode_long_list(input: &[u8], prefix: u8) -> Result<RlpList<'_>, DecodeError> {
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
    })
}

#[derive(Clone, Copy)]
struct ListFrame {
    end: usize,
    count: usize,
}

fn validate_list_payload(
    input: &[u8],
    accumulator: &mut DecodeAccumulator,
) -> Result<usize, DecodeError> {
    let mut stack = [ListFrame { end: 0, count: 0 }; MAX_TRAVERSAL_STACK];
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
        if item.is_list {
            let next_depth = checked_len_add(depth, 1)?;
            accumulator.check_nesting_depth(next_depth)?;
            if next_depth > MAX_TRAVERSAL_STACK {
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

struct ParsedItem {
    is_list: bool,
    payload_start: usize,
    payload_end: usize,
    item_end: usize,
}

fn parse_item(
    input: &[u8],
    offset: usize,
    container_end: usize,
) -> Result<ParsedItem, DecodeError> {
    let local = input
        .get(offset..container_end)
        .ok_or(DecodeError::OffsetOutOfBounds)?;
    let prefix = *local.first().ok_or(DecodeError::Malformed)?;
    let (is_list, header_len, payload_len) = match prefix {
        0x00..=0x7f => (false, 0, 1),
        super::SHORT_STRING_OFFSET..=super::LONG_STRING_OFFSET => {
            let scalar = decode_short_string(local, prefix)?;
            (false, scalar.header_len(), scalar.payload().len())
        }
        0xb8..=0xbf => {
            let scalar = decode_long_string(local, prefix)?;
            (false, scalar.header_len(), scalar.payload().len())
        }
        SHORT_LIST_OFFSET..=LONG_LIST_OFFSET => {
            let list = decode_short_list(local, prefix)?;
            (true, list.header_len(), list.payload().len())
        }
        0xf8..=0xff => {
            let list = decode_long_list(local, prefix)?;
            (true, list.header_len(), list.payload().len())
        }
    };
    let payload_start = checked_len_add(offset, header_len)?;
    let payload_end = checked_len_add(payload_start, payload_len)?;
    let item_end = if header_len == 0 {
        checked_len_add(offset, 1)?
    } else {
        payload_end
    };
    require_range_in_bounds(offset, item_end.saturating_sub(offset), container_end)?;
    Ok(ParsedItem {
        is_list,
        payload_start,
        payload_end,
        item_end,
    })
}
