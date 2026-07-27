//! Adversarial sorted and reverse-order node access-tracker benchmark.

use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use eth_valkyoth_evm_core::{EvmAccessTracker, EvmAddress, EvmNodeAccessTracker, EvmWord};

const DISTINCT_ACCESSES: usize = 32_768;
const ROLLBACKS: usize = 4_096;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sorted = run_pattern(false)?;
    let reversed = run_pattern(true)?;
    let empty_reverts = run_rollback_pattern(false)?;
    let one_insert_reverts = run_rollback_pattern(true)?;

    println!(
        "accesses={DISTINCT_ACCESSES} sorted_ms={} reverse_ms={} \
         reverts={ROLLBACKS} empty_revert_us={} one_insert_revert_us={}",
        sorted.as_millis(),
        reversed.as_millis(),
        empty_reverts.as_micros(),
        one_insert_reverts.as_micros()
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
    let (address_depth, storage_depth) = tracker.max_lookup_depths()?;
    if address_depth > 161 || storage_depth > 417 {
        return Err("radix lookup depth exceeded the fixed key-width bound".into());
    }
    if tracker.address_len() != DISTINCT_ACCESSES || tracker.storage_len() != DISTINCT_ACCESSES {
        return Err("access benchmark did not retain every distinct access".into());
    }
    Ok(elapsed)
}

fn run_rollback_pattern(insert: bool) -> Result<Duration, Box<dyn std::error::Error>> {
    let capacity = DISTINCT_ACCESSES.saturating_add(1);
    let mut tracker = EvmNodeAccessTracker::try_new(capacity, 1)?;
    for value in 0..DISTINCT_ACCESSES {
        let address = EvmAddress::from_bytes(address_bytes(value));
        black_box(tracker.warm_address(black_box(address))?);
    }

    let reverted = EvmAddress::from_bytes(address_bytes(DISTINCT_ACCESSES));
    let started = Instant::now();
    for _ in 0..ROLLBACKS {
        let checkpoint = tracker.checkpoint()?;
        if insert {
            black_box(tracker.warm_address(black_box(reverted))?);
        }
        tracker.revert(checkpoint)?;
    }
    if tracker.address_len() != DISTINCT_ACCESSES {
        return Err("rollback benchmark changed retained outer state".into());
    }
    Ok(started.elapsed())
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
