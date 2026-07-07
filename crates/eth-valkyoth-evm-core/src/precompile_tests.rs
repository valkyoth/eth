use crate::{
    EVM_PRECOMPILE_INPUT_LIMIT, EvmAddress, EvmCoreError, EvmFork, EvmGas,
    EvmPrecompileImplementation, EvmPrecompileKind, EvmPrecompilePlan, EvmPrecompileRegistry,
    execute_identity, execute_ripemd160, execute_sha256,
};

fn registry(fork: EvmFork) -> Result<EvmPrecompileRegistry, EvmCoreError> {
    EvmPrecompileRegistry::try_new(fork)
}

#[test]
fn precompile_addresses_are_low_canonical_accounts() {
    assert_eq!(
        EvmPrecompileKind::Identity.address().to_bytes(),
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4]
    );
    assert_eq!(
        EvmPrecompileKind::Bls12MapFp2ToG2.address().to_bytes(),
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 17]
    );
}

#[test]
fn registry_is_fork_aware() -> Result<(), EvmCoreError> {
    assert_eq!(
        registry(EvmFork::FRONTIER)?.descriptor(EvmPrecompileKind::Bn254Add),
        Err(EvmCoreError::PrecompileNotAvailableInFork)
    );
    assert_eq!(
        registry(EvmFork::BYZANTIUM)?
            .descriptor(EvmPrecompileKind::Bn254Add)?
            .implementation,
        EvmPrecompileImplementation::RequiresCryptoBackend
    );
    assert_eq!(
        registry(EvmFork::FRONTIER)?
            .descriptor(EvmPrecompileKind::Sha256)?
            .implementation,
        EvmPrecompileImplementation::NativeSha256
    );
    assert_eq!(
        registry(EvmFork::FRONTIER)?
            .descriptor(EvmPrecompileKind::Ripemd160)?
            .implementation,
        EvmPrecompileImplementation::NativeRipemd160
    );
    assert_eq!(
        registry(EvmFork::ISTANBUL)?
            .descriptor(EvmPrecompileKind::Blake2F)?
            .address,
        EvmPrecompileKind::Blake2F.address()
    );
    assert_eq!(
        registry(EvmFork::CANCUN)?
            .descriptor(EvmPrecompileKind::KzgPointEvaluation)?
            .address,
        EvmPrecompileKind::KzgPointEvaluation.address()
    );
    Ok(())
}

#[test]
fn lookup_rejects_unknown_and_future_precompile_addresses() -> Result<(), EvmCoreError> {
    let unknown =
        EvmAddress::from_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 99]);
    let non_reserved =
        EvmAddress::from_bytes([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4]);
    assert_eq!(registry(EvmFork::FRONTIER)?.lookup(unknown)?, None);
    assert_eq!(registry(EvmFork::FRONTIER)?.lookup(non_reserved)?, None);
    assert_eq!(
        registry(EvmFork::BERLIN)?.lookup(EvmPrecompileKind::KzgPointEvaluation.address()),
        Err(EvmCoreError::PrecompileNotAvailableInFork)
    );
    Ok(())
}

#[test]
fn lookup_returns_descriptors_for_known_addresses() -> Result<(), EvmCoreError> {
    let identity = registry(EvmFork::FRONTIER)?
        .lookup(EvmPrecompileKind::Identity.address())?
        .ok_or(EvmCoreError::PrecompileNotAvailableInFork)?;
    assert_eq!(identity.kind, EvmPrecompileKind::Identity);
    assert_eq!(identity.fork, EvmFork::FRONTIER);

    let kzg = registry(EvmFork::CANCUN)?
        .lookup(EvmPrecompileKind::KzgPointEvaluation.address())?
        .ok_or(EvmCoreError::PrecompileNotAvailableInFork)?;
    assert_eq!(kzg.kind, EvmPrecompileKind::KzgPointEvaluation);
    assert_eq!(kzg.fork, EvmFork::CANCUN);
    Ok(())
}

#[test]
fn identity_plan_computes_word_gas_and_executes() -> Result<(), EvmCoreError> {
    let descriptor = registry(EvmFork::FRONTIER)?.descriptor(EvmPrecompileKind::Identity)?;
    let plan = EvmPrecompilePlan::try_new(descriptor, &[1u8; 33])?;
    let mut output = [0u8; 33];
    assert_eq!(plan.gas_cost(), Some(EvmGas::new(21)));
    assert_eq!(plan.execute_identity(&[1u8; 33], &mut output)?, 33);
    assert_eq!(output, [1u8; 33]);
    Ok(())
}

#[test]
fn identity_output_buffer_is_checked_before_copy() -> Result<(), EvmCoreError> {
    let mut output = [0u8; 2];
    assert_eq!(
        execute_identity(&[1, 2, 3], &mut output),
        Err(EvmCoreError::PrecompileOutputTooSmall)
    );
    assert_eq!(output, [0, 0]);
    Ok(())
}

#[test]
fn input_policy_rejects_unbounded_or_bad_lengths() -> Result<(), EvmCoreError> {
    let blake = registry(EvmFork::ISTANBUL)?.descriptor(EvmPrecompileKind::Blake2F)?;
    assert_eq!(
        EvmPrecompilePlan::try_new(blake, &[0u8; 212]),
        Err(EvmCoreError::PrecompileInvalidInputLength)
    );
    let pairing = registry(EvmFork::BYZANTIUM)?.descriptor(EvmPrecompileKind::Bn254Pairing)?;
    assert_eq!(
        EvmPrecompilePlan::try_new(pairing, &[0u8; 191]),
        Err(EvmCoreError::PrecompileInvalidInputLength)
    );
    static OVERSIZED: [u8; EVM_PRECOMPILE_INPUT_LIMIT + 1] = [0u8; EVM_PRECOMPILE_INPUT_LIMIT + 1];
    let identity = registry(EvmFork::FRONTIER)?.descriptor(EvmPrecompileKind::Identity)?;
    assert_eq!(
        EvmPrecompilePlan::try_new(identity, &OVERSIZED),
        Err(EvmCoreError::PrecompileInputTooLarge)
    );
    Ok(())
}

#[test]
fn unsupported_crypto_precompile_plans_do_not_execute_without_backend() -> Result<(), EvmCoreError>
{
    let descriptor = registry(EvmFork::FRONTIER)?.descriptor(EvmPrecompileKind::EcRecover)?;
    let plan = EvmPrecompilePlan::try_new(descriptor, &[0u8; 1])?;
    let mut output = [0u8; 32];
    assert_eq!(
        plan.execute_identity(&[0u8; 1], &mut output),
        Err(EvmCoreError::PrecompileBackendUnavailable)
    );
    Ok(())
}

#[test]
fn sha256_precompile_matches_known_vectors() -> Result<(), EvmCoreError> {
    let descriptor = registry(EvmFork::FRONTIER)?.descriptor(EvmPrecompileKind::Sha256)?;
    let plan = EvmPrecompilePlan::try_new(descriptor, b"")?;
    let mut output = [0u8; 32];
    assert_eq!(plan.gas_cost(), Some(EvmGas::new(60)));
    assert_eq!(plan.execute_sha256(b"", &mut output)?, 32);
    assert_eq!(
        output,
        hex_word("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
    );

    assert_eq!(execute_sha256(b"abc", &mut output)?, 32);
    assert_eq!(
        output,
        hex_word("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
    );
    Ok(())
}

#[test]
fn ripemd160_precompile_matches_known_vectors() -> Result<(), EvmCoreError> {
    let descriptor = registry(EvmFork::FRONTIER)?.descriptor(EvmPrecompileKind::Ripemd160)?;
    let plan = EvmPrecompilePlan::try_new(descriptor, b"")?;
    let mut output = [0u8; 32];
    assert_eq!(plan.gas_cost(), Some(EvmGas::new(600)));
    assert_eq!(plan.execute_ripemd160(b"", &mut output)?, 32);
    assert_eq!(
        output,
        hex_word("0000000000000000000000009c1185a5c5e9fc54612808977ee8f548b2258d31")
    );

    assert_eq!(execute_ripemd160(b"abc", &mut output)?, 32);
    assert_eq!(
        output,
        hex_word("0000000000000000000000008eb208f7e05d987a9b044a8e98c6b087f15a0bfc")
    );
    Ok(())
}

#[test]
fn hash_precompile_matches_padding_boundary_vectors() -> Result<(), EvmCoreError> {
    for case in HASH_BOUNDARY_CASES {
        let input = repeated_a_input(case.len);
        let mut output = [0u8; 32];
        assert_eq!(execute_sha256(input, &mut output)?, 32);
        assert_eq!(output, hex_word(case.sha256));

        assert_eq!(execute_ripemd160(input, &mut output)?, 32);
        assert_eq!(output, ripemd160_word(case.ripemd160));
    }
    Ok(())
}

#[test]
fn hash_precompile_output_buffer_is_checked_before_write() {
    let mut output = [7u8; 31];
    assert_eq!(
        execute_sha256(b"abc", &mut output),
        Err(EvmCoreError::PrecompileOutputTooSmall)
    );
    assert_eq!(output, [7u8; 31]);

    assert_eq!(
        execute_ripemd160(b"abc", &mut output),
        Err(EvmCoreError::PrecompileOutputTooSmall)
    );
    assert_eq!(output, [7u8; 31]);
}

#[test]
fn hash_plan_rejects_wrong_input_len_or_kind() -> Result<(), EvmCoreError> {
    let sha = registry(EvmFork::FRONTIER)?.descriptor(EvmPrecompileKind::Sha256)?;
    let plan = EvmPrecompilePlan::try_new(sha, b"abc")?;
    let mut output = [0u8; 32];
    assert_eq!(
        plan.execute_sha256(b"abcd", &mut output),
        Err(EvmCoreError::PrecompileInvalidInputLength)
    );

    let ripemd = registry(EvmFork::FRONTIER)?.descriptor(EvmPrecompileKind::Ripemd160)?;
    let wrong_plan = EvmPrecompilePlan::try_new(ripemd, b"abc")?;
    assert_eq!(
        wrong_plan.execute_sha256(b"abc", &mut output),
        Err(EvmCoreError::PrecompileBackendUnavailable)
    );
    Ok(())
}

#[test]
fn known_precompile_gas_policies_are_bounded() -> Result<(), EvmCoreError> {
    let sha = registry(EvmFork::FRONTIER)?.descriptor(EvmPrecompileKind::Sha256)?;
    assert_eq!(sha.output_len, Some(32));
    assert_eq!(
        EvmPrecompilePlan::try_new(sha, &[0u8; 33])?.gas_cost(),
        Some(EvmGas::new(84))
    );

    let byzantium_pairing =
        registry(EvmFork::BYZANTIUM)?.descriptor(EvmPrecompileKind::Bn254Pairing)?;
    assert_eq!(
        EvmPrecompilePlan::try_new(byzantium_pairing, &[0u8; 192])?.gas_cost(),
        Some(EvmGas::new(180_000))
    );

    let bn_add = registry(EvmFork::ISTANBUL)?.descriptor(EvmPrecompileKind::Bn254Add)?;
    assert_eq!(bn_add.output_len, Some(64));

    let pairing = registry(EvmFork::ISTANBUL)?.descriptor(EvmPrecompileKind::Bn254Pairing)?;
    assert_eq!(
        EvmPrecompilePlan::try_new(pairing, &[0u8; 384])?.gas_cost(),
        Some(EvmGas::new(113_000))
    );

    let mut blake_input = [0u8; 213];
    blake_input[3] = 12;
    let blake = registry(EvmFork::ISTANBUL)?.descriptor(EvmPrecompileKind::Blake2F)?;
    assert_eq!(blake.output_len, Some(64));
    assert_eq!(
        EvmPrecompilePlan::try_new(blake, &blake_input)?.gas_cost(),
        Some(EvmGas::new(12))
    );
    Ok(())
}

struct HashBoundaryCase {
    len: usize,
    sha256: &'static str,
    ripemd160: &'static str,
}

const HASH_BOUNDARY_CASES: &[HashBoundaryCase] = &[
    HashBoundaryCase {
        len: 55,
        sha256: "9f4390f8d30c2dd92ec9f095b65e2b9ae9b0a925a5258e241c9f1e910f734318",
        ripemd160: "0d8a8c9063a48576a7c97e9f95253a6e53ff6765",
    },
    HashBoundaryCase {
        len: 56,
        sha256: "b35439a4ac6f0948b6d6f9e3c6af0f5f590ce20f1bde7090ef7970686ec6738a",
        ripemd160: "e72334b46c83cc70bef979e15453706c95b888be",
    },
    HashBoundaryCase {
        len: 64,
        sha256: "ffe054fe7ae0cb6dc65c3af9b61d5209f439851db43d0ba5997337df154668eb",
        ripemd160: "9dfb7d374ad924f3f88de96291c33e9abed53e32",
    },
    HashBoundaryCase {
        len: 119,
        sha256: "31eba51c313a5c08226adf18d4a359cfdfd8d2e816b13f4af952f7ea6584dcfb",
        ripemd160: "23e398ff2bac815aa1bbb57ca2a669c841872919",
    },
    HashBoundaryCase {
        len: 120,
        sha256: "2f3d335432c70b580af0e8e1b3674a7c020d683aa5f73aaaedfdc55af904c21c",
        ripemd160: "c476770a6dae31fcee8d25efe6559a05c8024595",
    },
];

static REPEATED_A_120: [u8; 120] = [b'a'; 120];

fn repeated_a_input(len: usize) -> &'static [u8] {
    match REPEATED_A_120.get(..len) {
        Some(input) => input,
        None => &[],
    }
}

fn hex_word(hex: &str) -> [u8; 32] {
    assert_eq!(hex.len(), 64);
    let mut output = [0u8; 32];
    for (target, pair) in output.iter_mut().zip(hex.as_bytes().chunks_exact(2)) {
        let high = pair.first().copied().map(hex_nibble).unwrap_or(0);
        let low = pair.get(1).copied().map(hex_nibble).unwrap_or(0);
        *target = (high << 4) | low;
    }
    output
}

fn ripemd160_word(hex: &str) -> [u8; 32] {
    assert_eq!(hex.len(), 40);
    let mut output = [0u8; 32];
    let digest = hex20(hex);
    if let Some(target) = output.get_mut(12..) {
        target.copy_from_slice(&digest);
    }
    output
}

fn hex20(hex: &str) -> [u8; 20] {
    let mut output = [0u8; 20];
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

#[test]
fn deferred_dynamic_precompile_gas_is_not_zero_cost() -> Result<(), EvmCoreError> {
    let modexp = registry(EvmFork::BYZANTIUM)?.descriptor(EvmPrecompileKind::Modexp)?;
    assert_eq!(
        EvmPrecompilePlan::try_new(modexp, &[0u8; 96])?.gas_cost(),
        None
    );

    let bls = registry(EvmFork::PRAGUE)?.descriptor(EvmPrecompileKind::Bls12PairingCheck)?;
    assert_eq!(
        EvmPrecompilePlan::try_new(bls, &[0u8; 384])?.gas_cost(),
        None
    );
    Ok(())
}
