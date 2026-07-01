use super::*;

#[test]
fn list_envelope_defers_header_until_writer_finish_succeeds() {
    let mut output = [0xaa_u8; 4];

    assert_eq!(
        encode_list_envelope(3, &mut output, |_fields| Ok(())),
        Err(DecodeError::DecoderOverread)
    );
    assert_eq!(output, [0xaa; 4]);
}

#[test]
fn typed_envelope_defers_type_byte_until_payload_succeeds() {
    let mut output = [0xaa_u8; 5];

    assert_eq!(
        encode_typed_envelope(ACCESS_LIST_TRANSACTION_TYPE, 3, &mut output, |_fields| Ok(
            ()
        )),
        Err(DecodeError::DecoderOverread)
    );
    assert_eq!(output, [0xaa; 5]);
}
