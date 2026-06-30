use crate::{DecodeError, DecodeLimits, checked_len_add, require_range_in_bounds};

use super::{
    LONG_LIST_OFFSET, RlpItem, RlpList, RlpListForm, RlpScalar, SHORT_LIST_OFFSET,
    SHORT_STRING_LIMIT, count_immediate_items,
};
use crate::rlp::{
    RlpScalarForm, decode_long_string, decode_short_string, parse_payload_len, payload,
};

pub(super) struct ParsedItem {
    pub(super) kind: ParsedItemKind,
    header_len: usize,
    pub(super) payload_start: usize,
    pub(super) payload_end: usize,
    pub(super) item_end: usize,
}

impl ParsedItem {
    pub(super) fn into_rlp_item<'a>(
        self,
        input: &'a [u8],
        offset: usize,
        limits: DecodeLimits,
        depth_remaining: usize,
    ) -> Result<(RlpItem<'a>, usize), DecodeError> {
        let payload = input
            .get(self.payload_start..self.payload_end)
            .ok_or(DecodeError::OffsetOutOfBounds)?;
        let encoded_len = self
            .item_end
            .checked_sub(offset)
            .ok_or(DecodeError::LengthOverflow)?;
        let item = match self.kind {
            ParsedItemKind::Scalar(form) => RlpItem::Scalar(RlpScalar {
                payload,
                encoded_len,
                header_len: self.header_len,
                form,
            }),
            ParsedItemKind::List(form) => {
                if depth_remaining == 0 {
                    return Err(DecodeError::NestingTooDeep);
                }
                RlpItem::List(RlpList {
                    payload,
                    encoded_len,
                    header_len: self.header_len,
                    item_count: count_immediate_items(payload, limits)?,
                    form,
                    limits,
                    iteration_depth_remaining: depth_remaining.saturating_sub(1),
                })
            }
        };
        Ok((item, self.item_end))
    }
}

#[derive(Clone, Copy)]
pub(super) enum ParsedItemKind {
    Scalar(RlpScalarForm),
    List(RlpListForm),
}

pub(super) fn parse_item(
    input: &[u8],
    offset: usize,
    container_end: usize,
) -> Result<ParsedItem, DecodeError> {
    let local = input
        .get(offset..container_end)
        .ok_or(DecodeError::OffsetOutOfBounds)?;
    let prefix = *local.first().ok_or(DecodeError::Malformed)?;
    let (kind, header_len, payload_len) = match prefix {
        0x00..=0x7f => (ParsedItemKind::Scalar(RlpScalarForm::SingleByte), 0, 1),
        crate::rlp::SHORT_STRING_OFFSET..=crate::rlp::LONG_STRING_OFFSET => {
            let scalar = decode_short_string(local, prefix)?;
            (
                ParsedItemKind::Scalar(scalar.form()),
                scalar.header_len(),
                scalar.payload().len(),
            )
        }
        0xb8..=0xbf => {
            let scalar = decode_long_string(local, prefix)?;
            (
                ParsedItemKind::Scalar(scalar.form()),
                scalar.header_len(),
                scalar.payload().len(),
            )
        }
        SHORT_LIST_OFFSET..=LONG_LIST_OFFSET => {
            let payload_len = usize::from(prefix.saturating_sub(SHORT_LIST_OFFSET));
            payload(local, 1, payload_len)?;
            (ParsedItemKind::List(RlpListForm::ShortList), 1, payload_len)
        }
        0xf8..=0xff => {
            let len_of_len = usize::from(prefix.saturating_sub(LONG_LIST_OFFSET));
            let payload_len = parse_payload_len(local, 1, len_of_len)?;
            if payload_len <= SHORT_STRING_LIMIT {
                return Err(DecodeError::Malformed);
            }
            let header_len = checked_len_add(1, len_of_len)?;
            payload(local, header_len, payload_len)?;
            (
                ParsedItemKind::List(RlpListForm::LongList),
                header_len,
                payload_len,
            )
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
        kind,
        header_len,
        payload_start,
        payload_end,
        item_end,
    })
}
