//! Linear-work evidence runner for shared RLP decode sessions.

use std::time::Instant;

use eth_valkyoth_codec::{
    DecodeLimits, DecodeSession, DecodeSessionPolicy, decode_rlp_list_in_session,
};

const ITERATIONS: usize = 20_000;
const PAYLOAD_SIZES: [usize; 4] = [16, 64, 128, 255];

fn main() -> Result<(), &'static str> {
    println!("payload_bytes,iterations,total_work,elapsed_ns,ns_per_byte");
    for payload_size in PAYLOAD_SIZES {
        let input = encoded_list(payload_size)?;
        let limits = DecodeLimits {
            max_input_bytes: input.len(),
            max_list_items: payload_size,
            max_nesting_depth: 1,
            max_total_allocation: input.len(),
            max_proof_nodes: 1,
            max_total_items: payload_size
                .checked_add(1)
                .ok_or("benchmark item limit overflow")?,
        };
        let policy = DecodeSessionPolicy::reviewed_policy(
            limits,
            input.len(),
            input.len(),
            1,
            input.len(),
            input.len(),
            input.len(),
            input
                .len()
                .checked_mul(6)
                .ok_or("benchmark work limit overflow")?,
        )
        .map_err(|_| "benchmark policy must be valid")?;
        let start = Instant::now();
        let mut observed_work = 0usize;
        for _ in 0..ITERATIONS {
            let mut session =
                DecodeSession::new(policy).map_err(|_| "benchmark session must initialize")?;
            decode_rlp_list_in_session(&input, &mut session)
                .map_err(|_| "benchmark list must decode")?;
            if session.encoded_bytes() != input.len() {
                return Err("structural scan must charge each byte exactly once");
            }
            observed_work = session.total_work();
        }
        let elapsed = start.elapsed().as_nanos();
        let total_bytes = input.len().saturating_mul(ITERATIONS);
        let ns_per_byte = elapsed.checked_div(total_bytes as u128).unwrap_or(0);
        println!("{payload_size},{ITERATIONS},{observed_work},{elapsed},{ns_per_byte}");
    }
    Ok(())
}

fn encoded_list(payload_size: usize) -> Result<Vec<u8>, &'static str> {
    let capacity = payload_size
        .checked_add(2)
        .ok_or("benchmark capacity overflow")?;
    let mut output = Vec::with_capacity(capacity);
    if payload_size <= 55 {
        let len = u8::try_from(payload_size).map_err(|_| "short length must fit")?;
        output.push(0xc0_u8.saturating_add(len));
    } else {
        let len = u8::try_from(payload_size).map_err(|_| "benchmark length must fit")?;
        output.extend_from_slice(&[0xf8, len]);
    }
    let encoded_len = output
        .len()
        .checked_add(payload_size)
        .ok_or("benchmark encoded length overflow")?;
    output.resize(encoded_len, 1);
    Ok(output)
}
