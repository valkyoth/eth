//! Adversarial sorted and reverse-order node access-tracker benchmark.

use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use eth_valkyoth_evm_core::{EvmAddress, EvmNodeAccessTracker, EvmWord};

const DISTINCT_ACCESSES: usize = 32_768;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sorted = run_pattern(false)?;
    let reversed = run_pattern(true)?;

    println!(
        "accesses={DISTINCT_ACCESSES} sorted_ms={} reverse_ms={}",
        sorted.as_millis(),
        reversed.as_millis()
    );
    Ok(())
}

fn run_pattern(reverse: bool) -> Result<Duration, Box<dyn std::error::Error>> {
    let mut tracker = EvmNodeAccessTracker::try_new(DISTINCT_ACCESSES, DISTINCT_ACCESSES)?;
    let started = Instant::now();
    for offset in 0..DISTINCT_ACCESSES {
        let value = if reverse {
            DISTINCT_ACCESSES.saturating_sub(offset)
        } else {
            offset
        };
        let address = EvmAddress::from_bytes(address_bytes(value));
        let slot = EvmWord::from_be_bytes(slot_bytes(value));
        black_box(tracker.warm_storage(black_box(address), black_box(slot))?);
    }
    let elapsed = started.elapsed();
    let (address_height, storage_height) = tracker.tree_heights()?;
    let height_limit = logarithmic_height_limit(DISTINCT_ACCESSES);
    if address_height > height_limit || storage_height > height_limit {
        return Err("AVL height exceeded deterministic logarithmic bound".into());
    }
    if tracker.address_len() != DISTINCT_ACCESSES || tracker.storage_len() != DISTINCT_ACCESSES {
        return Err("access benchmark did not retain every distinct access".into());
    }
    Ok(elapsed)
}

fn address_bytes(value: usize) -> [u8; 20] {
    let mut bytes = [0; 20];
    for (destination, source) in bytes.iter_mut().rev().zip(value.to_be_bytes().iter().rev()) {
        *destination = *source;
    }
    bytes
}

fn slot_bytes(value: usize) -> [u8; 32] {
    let mut bytes = [0; 32];
    for (destination, source) in bytes.iter_mut().rev().zip(value.to_be_bytes().iter().rev()) {
        *destination = *source;
    }
    bytes
}

fn logarithmic_height_limit(items: usize) -> usize {
    let bits = usize::BITS.saturating_sub(items.leading_zeros()) as usize;
    bits.saturating_mul(2).saturating_add(1)
}
