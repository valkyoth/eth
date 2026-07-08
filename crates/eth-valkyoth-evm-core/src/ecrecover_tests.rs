use crate::{
    EVM_ECRECOVER_INPUT_BYTES, EVM_ECRECOVER_PUBLIC_KEY_BYTES, EvmAddress, EvmCoreError,
    EvmEcRecoverBackend, EvmEcRecoverSignature, EvmFork, EvmGas, EvmPrecompileKeccak256,
    EvmPrecompileKind, EvmPrecompilePlan, EvmPrecompileRegistry, execute_ecrecover,
};

const DIGEST_OFFSET: usize = 0;
const V_OFFSET: usize = 32;
const R_OFFSET: usize = 64;
const S_OFFSET: usize = 96;
const WORD_BYTES: usize = 32;

fn registry(fork: EvmFork) -> Result<EvmPrecompileRegistry, EvmCoreError> {
    EvmPrecompileRegistry::try_new(fork)
}

#[test]
fn ecrecover_executes_with_caller_provided_backends() -> Result<(), EvmCoreError> {
    let digest = filled_word(1);
    let r = filled_word(2);
    let s = filled_word(3);
    let frame = ecrecover_frame(digest, 27, r, s);
    let mut frame_with_extra = [0u8; 131];
    if let Some(target) = frame_with_extra.get_mut(..EVM_ECRECOVER_INPUT_BYTES) {
        target.copy_from_slice(&frame);
    }
    if let Some(extra) = frame_with_extra.get_mut(EVM_ECRECOVER_INPUT_BYTES..) {
        extra.copy_from_slice(&[9, 9, 9]);
    }

    let public_key = [4u8; EVM_ECRECOVER_PUBLIC_KEY_BYTES];
    let address = [7u8; EvmAddress::LEN];
    let mut backend = RecordingEcRecoverBackend::new(digest, r, s, 0, public_key);
    let mut hasher = RecordingKeccak::new(public_key, address);
    let mut output = [0u8; 32];

    let descriptor = registry(EvmFork::FRONTIER)?.descriptor(EvmPrecompileKind::EcRecover)?;
    let plan = EvmPrecompilePlan::try_new(descriptor, &frame_with_extra)?;
    assert_eq!(plan.gas_cost(), Some(EvmGas::new(3_000)));
    assert_eq!(
        plan.execute_ecrecover(&frame_with_extra, &mut output, &mut backend, &mut hasher)?,
        32
    );
    assert_eq!(output, address_word(address));
    assert_eq!(backend.calls, 1);
    assert_eq!(hasher.calls, 1);
    Ok(())
}

#[test]
fn ecrecover_accepts_high_s_values_like_the_precompile_spec() -> Result<(), EvmCoreError> {
    let digest = filled_word(8);
    let r = filled_word(9);
    let s = secp256k1_order_minus_one();
    let frame = ecrecover_frame(digest, 28, r, s);
    let public_key = [10u8; EVM_ECRECOVER_PUBLIC_KEY_BYTES];
    let address = [11u8; EvmAddress::LEN];
    let mut output = [0u8; 32];
    assert_eq!(
        execute_ecrecover(
            &frame,
            &mut output,
            RecordingEcRecoverBackend::new(digest, r, s, 1, public_key),
            RecordingKeccak::new(public_key, address),
        )?,
        32
    );
    assert_eq!(output, address_word(address));
    Ok(())
}

#[test]
fn ecrecover_invalid_inputs_return_zero_length_without_backend_call() -> Result<(), EvmCoreError> {
    let digest = filled_word(1);
    let r = filled_word(2);
    let s = filled_word(3);
    let mut frame = ecrecover_frame(digest, 29, r, s);
    let mut output = [6u8; 32];
    let mut backend = RejectingEcRecoverBackend { calls: 0 };
    let mut hasher = RejectingKeccak { calls: 0 };
    assert_eq!(
        execute_ecrecover(&frame, &mut output, &mut backend, &mut hasher)?,
        0
    );
    assert_eq!(output, [6u8; 32]);
    assert_eq!(backend.calls, 0);
    assert_eq!(hasher.calls, 0);

    if let Some(byte) = frame.get_mut(V_OFFSET) {
        *byte = 1;
    }
    assert_eq!(
        execute_ecrecover(&frame, &mut output, &mut backend, &mut hasher)?,
        0
    );
    assert_eq!(backend.calls, 0);
    assert_eq!(hasher.calls, 0);
    Ok(())
}

#[test]
fn ecrecover_rejects_bad_scalars_and_short_output() -> Result<(), EvmCoreError> {
    let digest = filled_word(1);
    let r = [0u8; 32];
    let s = filled_word(3);
    let frame = ecrecover_frame(digest, 27, r, s);
    let mut output = [5u8; 32];
    let mut backend = RejectingEcRecoverBackend { calls: 0 };
    let mut hasher = RejectingKeccak { calls: 0 };
    assert_eq!(
        execute_ecrecover(&frame, &mut output, &mut backend, &mut hasher)?,
        0
    );
    assert_eq!(backend.calls, 0);

    let mut short_output = [5u8; 31];
    assert_eq!(
        execute_ecrecover(&frame, &mut short_output, &mut backend, &mut hasher),
        Err(EvmCoreError::PrecompileOutputTooSmall)
    );
    assert_eq!(short_output, [5u8; 31]);
    Ok(())
}

#[test]
fn ecrecover_plan_rejects_wrong_input_len_or_kind() -> Result<(), EvmCoreError> {
    let descriptor = registry(EvmFork::FRONTIER)?.descriptor(EvmPrecompileKind::EcRecover)?;
    let frame = ecrecover_frame(filled_word(1), 27, filled_word(2), filled_word(3));
    let plan = EvmPrecompilePlan::try_new(descriptor, &frame)?;
    let mut output = [0u8; 32];
    assert_eq!(
        plan.execute_ecrecover(
            frame.get(..127).unwrap_or(&[]),
            &mut output,
            RejectingEcRecoverBackend { calls: 0 },
            RejectingKeccak { calls: 0 },
        ),
        Err(EvmCoreError::PrecompileInvalidInputLength)
    );

    let identity = registry(EvmFork::FRONTIER)?.descriptor(EvmPrecompileKind::Identity)?;
    let wrong_plan = EvmPrecompilePlan::try_new(identity, &frame)?;
    assert_eq!(
        wrong_plan.execute_ecrecover(
            &frame,
            &mut output,
            RejectingEcRecoverBackend { calls: 0 },
            RejectingKeccak { calls: 0 },
        ),
        Err(EvmCoreError::PrecompileBackendUnavailable)
    );
    Ok(())
}

#[derive(Clone, Copy)]
struct RecordingEcRecoverBackend {
    expected_digest: [u8; 32],
    expected_r: [u8; 32],
    expected_s: [u8; 32],
    expected_y_parity: u8,
    public_key: [u8; EVM_ECRECOVER_PUBLIC_KEY_BYTES],
    calls: u8,
}

impl RecordingEcRecoverBackend {
    fn new(
        expected_digest: [u8; 32],
        expected_r: [u8; 32],
        expected_s: [u8; 32],
        expected_y_parity: u8,
        public_key: [u8; EVM_ECRECOVER_PUBLIC_KEY_BYTES],
    ) -> Self {
        Self {
            expected_digest,
            expected_r,
            expected_s,
            expected_y_parity,
            public_key,
            calls: 0,
        }
    }
}

impl EvmEcRecoverBackend for RecordingEcRecoverBackend {
    fn recover_uncompressed_public_key(
        &mut self,
        digest: [u8; 32],
        signature: EvmEcRecoverSignature,
    ) -> Option<[u8; EVM_ECRECOVER_PUBLIC_KEY_BYTES]> {
        self.calls = self.calls.saturating_add(1);
        assert_eq!(digest, self.expected_digest);
        assert_eq!(signature.r(), self.expected_r);
        assert_eq!(signature.s(), self.expected_s);
        assert_eq!(signature.y_parity(), self.expected_y_parity);
        Some(self.public_key)
    }
}

struct RejectingEcRecoverBackend {
    calls: u8,
}

impl EvmEcRecoverBackend for RejectingEcRecoverBackend {
    fn recover_uncompressed_public_key(
        &mut self,
        _digest: [u8; 32],
        _signature: EvmEcRecoverSignature,
    ) -> Option<[u8; EVM_ECRECOVER_PUBLIC_KEY_BYTES]> {
        self.calls = self.calls.saturating_add(1);
        None
    }
}

#[derive(Clone, Copy)]
struct RecordingKeccak {
    expected_input: [u8; EVM_ECRECOVER_PUBLIC_KEY_BYTES],
    address: [u8; EvmAddress::LEN],
    calls: u8,
}

impl RecordingKeccak {
    fn new(expected_input: [u8; EVM_ECRECOVER_PUBLIC_KEY_BYTES], address: [u8; 20]) -> Self {
        Self {
            expected_input,
            address,
            calls: 0,
        }
    }
}

impl EvmPrecompileKeccak256 for RecordingKeccak {
    fn keccak256(&mut self, input: &[u8]) -> [u8; 32] {
        self.calls = self.calls.saturating_add(1);
        assert_eq!(input, self.expected_input);
        address_word(self.address)
    }
}

struct RejectingKeccak {
    calls: u8,
}

impl EvmPrecompileKeccak256 for RejectingKeccak {
    fn keccak256(&mut self, _input: &[u8]) -> [u8; 32] {
        self.calls = self.calls.saturating_add(1);
        [0u8; 32]
    }
}

fn ecrecover_frame(
    digest: [u8; 32],
    v: u8,
    r: [u8; 32],
    s: [u8; 32],
) -> [u8; EVM_ECRECOVER_INPUT_BYTES] {
    let mut frame = [0u8; EVM_ECRECOVER_INPUT_BYTES];
    copy_word(&mut frame, DIGEST_OFFSET, digest);
    if let Some(slot) = frame.get_mut(V_OFFSET..V_OFFSET.saturating_add(WORD_BYTES)) {
        slot.fill(0);
        if let Some(byte) = slot.last_mut() {
            *byte = v;
        }
    }
    copy_word(&mut frame, R_OFFSET, r);
    copy_word(&mut frame, S_OFFSET, s);
    frame
}

fn copy_word(output: &mut [u8], offset: usize, word: [u8; 32]) {
    if let Some(target) = output.get_mut(offset..offset.saturating_add(WORD_BYTES)) {
        target.copy_from_slice(&word);
    }
}

fn filled_word(byte: u8) -> [u8; 32] {
    [byte; 32]
}

fn address_word(address: [u8; 20]) -> [u8; 32] {
    let mut output = [0u8; 32];
    if let Some(target) = output.get_mut(12..) {
        target.copy_from_slice(&address);
    }
    output
}

fn secp256k1_order_minus_one() -> [u8; 32] {
    hex_word("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364140")
}

fn hex_word(hex: &str) -> [u8; 32] {
    let mut output = [0u8; 32];
    for (target, pair) in output.iter_mut().zip(hex.as_bytes().chunks_exact(2)) {
        let high = pair.first().copied().map(hex_nibble).unwrap_or(0);
        let low = pair.get(1).copied().map(hex_nibble).unwrap_or(0);
        *target = (high << 4) | low;
    }
    output
}

fn hex_nibble(byte: u8) -> u8 {
    match byte {
        b'0' => 0,
        b'1' => 1,
        b'2' => 2,
        b'3' => 3,
        b'4' => 4,
        b'5' => 5,
        b'6' => 6,
        b'7' => 7,
        b'8' => 8,
        b'9' => 9,
        b'a' | b'A' => 10,
        b'b' | b'B' => 11,
        b'c' | b'C' => 12,
        b'd' | b'D' => 13,
        b'e' | b'E' => 14,
        b'f' | b'F' => 15,
        _ => 0,
    }
}
