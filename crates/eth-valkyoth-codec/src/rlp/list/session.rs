use crate::{DecodeError, DecodeSession, require_exact_consumption};

use super::*;

impl<'a> RlpList<'a> {
    /// Iterates immediate children while charging every compatibility reparse.
    pub fn items_in_session<'s>(
        self,
        session: &'s mut DecodeSession,
    ) -> RlpListSessionItems<'a, 's> {
        RlpListSessionItems {
            inner: self.items(),
            session,
        }
    }
}

impl<'a> RlpListItems<'a> {
    /// Advances this cursor while charging the shared decode session.
    ///
    /// Unlike [`RlpList::items_in_session`], this method borrows the session
    /// only for one step. Nested consumers can therefore use the same session
    /// before requesting the next sibling item.
    pub fn next_in_session(
        &mut self,
        session: &mut DecodeSession,
    ) -> Option<Result<RlpItem<'a>, DecodeError>> {
        if self.remaining == 0 {
            return None;
        }

        let result = parse_item(self.input, self.cursor, self.input.len()).and_then(|item| {
            let encoded_len = item
                .item_end
                .checked_sub(self.cursor)
                .ok_or(DecodeError::LengthOverflow)?;
            let headers = usize::from(item.header_len() != 0);
            session.account_rlp_reparse(encoded_len, headers, 1)?;
            item.into_rlp_item_in_session(
                self.input,
                self.cursor,
                self.limits,
                self.depth_remaining,
                session,
            )
        });
        self.remaining = self.remaining.saturating_sub(1);

        match result {
            Ok((item, next_cursor)) => {
                self.cursor = next_cursor;
                Some(Ok(item))
            }
            Err(error) => {
                self.remaining = 0;
                Some(Err(error))
            }
        }
    }
}

/// Accounted compatibility iterator over immediate RLP child items.
pub struct RlpListSessionItems<'a, 's> {
    inner: RlpListItems<'a>,
    session: &'s mut DecodeSession,
}

impl<'a> Iterator for RlpListSessionItems<'a, '_> {
    type Item = Result<RlpItem<'a>, DecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next_in_session(self.session)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl core::iter::FusedIterator for RlpListSessionItems<'_, '_> {}

/// Decodes exactly one canonical RLP list under a shared work session.
pub fn decode_rlp_list_in_session<'a>(
    input: &'a [u8],
    session: &mut DecodeSession,
) -> Result<RlpList<'a>, DecodeError> {
    let list = decode_rlp_list_partial_in_session(input, session)?;
    require_exact_consumption(list.encoded_len, input.len())?;
    Ok(list)
}

/// Decodes one canonical RLP list while charging one shared work session.
///
/// Structural validation visits the nested payload once. Later iteration must
/// use [`RlpList::items_in_session`] when it belongs to the same untrusted
/// operation so compatibility reparses remain charged.
pub fn decode_rlp_list_partial_in_session<'a>(
    input: &'a [u8],
    session: &mut DecodeSession,
) -> Result<RlpList<'a>, DecodeError> {
    session.check_input_len(input.len())?;
    session.account_items(1)?;
    session.check_nesting_depth(1)?;

    let prefix = *input.first().ok_or(DecodeError::Malformed)?;
    let list = match prefix {
        SHORT_LIST_OFFSET..=LONG_LIST_OFFSET => decode_short_list(input, prefix, session.limits())?,
        0xf8..=0xff => decode_long_list(input, prefix, session.limits())?,
        0x00..=0xbf => return Err(DecodeError::UnexpectedScalar),
    };
    session.account_encoded_bytes(list.header_len())?;
    session.account_rlp_headers(1)?;
    let item_count = validate_list_payload_in_session(list.payload, session)?;
    session.check_list_count(item_count)?;
    Ok(RlpList {
        item_count,
        iteration_depth_remaining: MAX_RLP_LIST_TRAVERSAL_DEPTH.saturating_sub(1),
        ..list
    })
}

fn validate_list_payload_in_session(
    input: &[u8],
    session: &mut DecodeSession,
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
            session.check_list_count(finished_count)?;
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
        session.check_list_count(frame.count)?;
        let item = parse_item(input, cursor, frame.end)?;
        session.account_items(1)?;
        session.account_encoded_bytes(item.structural_scan_bytes(cursor)?)?;
        if item.header_len() != 0 {
            session.account_rlp_headers(1)?;
        }
        if matches!(item.kind, ParsedItemKind::List(_)) {
            let next_depth = checked_len_add(depth, 1)?;
            session.check_nesting_depth(next_depth)?;
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
