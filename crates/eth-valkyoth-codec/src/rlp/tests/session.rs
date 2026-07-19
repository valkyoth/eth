use super::super::*;
use crate::{DecodeSession, DecodeSessionPolicy};

const NESTED_LIST: &[u8] = &[0xc4, 0xc2, 0x01, 0x02, 0x03];

fn test_session() -> Result<DecodeSession, &'static str> {
    DecodeSession::new(DecodeSessionPolicy::TEST_FIXTURE).map_err(|_| "invalid test policy")
}

#[test]
fn structural_decode_counts_each_encoded_byte_once() -> Result<(), &'static str> {
    let mut session = test_session()?;
    let list = decode_rlp_list_in_session(NESTED_LIST, &mut session);

    assert!(list.is_ok());
    assert_eq!(session.encoded_bytes(), NESTED_LIST.len());
    assert_eq!(session.rlp_headers(), 2);
    assert_eq!(session.items(), 5);
    assert_eq!(session.max_nesting_depth(), 2);
    Ok(())
}

#[test]
fn nested_cursor_steps_share_one_non_resetting_ledger() -> Result<(), &'static str> {
    let mut session = test_session()?;
    let list = decode_rlp_list_in_session(NESTED_LIST, &mut session)
        .map_err(|_| "nested list must decode")?;
    let before = session.total_work();
    let mut root = list.items();
    let child = root
        .next_in_session(&mut session)
        .ok_or("nested item must exist")?
        .map_err(|_| "nested item must decode")?
        .as_list()
        .ok_or("nested item must be a list")?;
    let mut children = child.items();
    assert!(matches!(
        children.next_in_session(&mut session),
        Some(Ok(RlpItem::Scalar(_)))
    ));
    assert!(matches!(
        children.next_in_session(&mut session),
        Some(Ok(RlpItem::Scalar(_)))
    ));
    assert!(matches!(
        root.next_in_session(&mut session),
        Some(Ok(RlpItem::Scalar(_)))
    ));

    assert!(session.total_work() > before);
    assert_eq!(session.encoded_bytes(), 11);
    assert_eq!(session.rlp_headers(), 3);
    assert_eq!(session.items(), 11);
    Ok(())
}

#[test]
fn nested_list_recount_charges_every_immediate_visit() -> Result<(), &'static str> {
    const MANY_CHILDREN: &[u8] = &[0xc6, 0xc5, 0x01, 0x02, 0x03, 0x04, 0x05];
    let mut session = test_session()?;
    let list = decode_rlp_list_in_session(MANY_CHILDREN, &mut session)
        .map_err(|_| "nested list must decode")?;
    let before_items = session.items();
    let before_headers = session.rlp_headers();
    let before_bytes = session.encoded_bytes();

    let mut items = list.items();
    assert!(matches!(
        items.next_in_session(&mut session),
        Some(Ok(RlpItem::List(_)))
    ));
    assert_eq!(session.items() - before_items, 6);
    assert_eq!(session.rlp_headers() - before_headers, 1);
    assert_eq!(session.encoded_bytes() - before_bytes, 6);
    Ok(())
}

#[test]
fn nested_list_recount_cannot_exceed_item_ceiling() -> Result<(), &'static str> {
    const MANY_CHILDREN: &[u8] = &[0xc6, 0xc5, 0x01, 0x02, 0x03, 0x04, 0x05];
    let limits = DecodeLimits {
        max_input_bytes: 7,
        max_list_items: 8,
        max_nesting_depth: 2,
        max_total_allocation: 7,
        max_proof_nodes: 1,
        max_total_items: 11,
    };
    let policy = DecodeSessionPolicy::reviewed_policy(limits, 16, 8, 1, 1, 1, 1, 64)
        .map_err(|_| "policy must be valid")?;
    let mut session = DecodeSession::new(policy).map_err(|_| "session must initialize")?;
    let list = decode_rlp_list_in_session(MANY_CHILDREN, &mut session)
        .map_err(|_| "nested list must decode")?;
    let mut items = list.items();

    assert_eq!(
        items.next_in_session(&mut session),
        Some(Err(DecodeError::ItemCountExceeded))
    );
    assert_eq!(session.items(), limits.max_total_items);
    Ok(())
}

#[test]
fn cumulative_encoded_budget_cannot_reset_between_values() -> Result<(), &'static str> {
    let limits = DecodeLimits {
        max_input_bytes: 4,
        max_list_items: 4,
        max_nesting_depth: 2,
        max_total_allocation: 4,
        max_proof_nodes: 1,
        max_total_items: 8,
    };
    let policy = DecodeSessionPolicy::reviewed_policy(limits, 4, 4, 1, 4, 4, 4, 16)
        .map_err(|_| "invalid test policy")?;
    let mut session = DecodeSession::new(policy).map_err(|_| "invalid test session")?;

    assert!(decode_rlp_scalar_in_session(&[0x82, 0xaa, 0xbb], &mut session).is_ok());
    assert_eq!(
        decode_rlp_scalar_in_session(&[0x81, 0xcc], &mut session),
        Err(DecodeError::EncodedBytesExceeded)
    );
    assert_eq!(session.encoded_bytes(), 3);
    Ok(())
}
