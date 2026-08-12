use core::sync::atomic::{AtomicUsize, Ordering};

use super::{AccessKey, RadixSet};
use crate::EvmCoreError;

static WIPE_CALLS: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct CountingWipeProbe(u8);

impl AccessKey for CountingWipeProbe {
    const BITS: usize = u8::BITS as usize;

    fn bit(&self, index: usize) -> bool {
        probe_bit(self.0, index)
    }

    fn wipe(&mut self) {
        self.0 = 0;
        let _ = WIPE_CALLS.fetch_add(1, Ordering::SeqCst);
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct StructuralProbe(u8);

impl AccessKey for StructuralProbe {
    const BITS: usize = u8::BITS as usize;

    fn bit(&self, index: usize) -> bool {
        probe_bit(self.0, index)
    }

    fn wipe(&mut self) {
        self.0 = 0;
    }
}

#[test]
fn rollback_clear_and_drop_wipe_each_removed_live_key() -> Result<(), EvmCoreError> {
    WIPE_CALLS.store(0, Ordering::SeqCst);
    {
        let mut set = RadixSet::try_new(2, 2)?;
        assert!(set.can_insert(CountingWipeProbe(1))?);
        set.insert_known_absent(CountingWipeProbe(1))?;
        let checkpoint = set.checkpoint();
        assert!(set.can_insert(CountingWipeProbe(2))?);
        set.insert_known_absent(CountingWipeProbe(2))?;
        set.restore(checkpoint)?;
        assert_eq!(WIPE_CALLS.load(Ordering::SeqCst), 1);

        set.clear();
        assert_eq!(WIPE_CALLS.load(Ordering::SeqCst), 2);
        assert!(set.can_insert(CountingWipeProbe(3))?);
        set.insert_known_absent(CountingWipeProbe(3))?;
    }
    assert_eq!(WIPE_CALLS.load(Ordering::SeqCst), 3);
    Ok(())
}

#[test]
fn rollback_retains_nodes_without_rebuilding_them() -> Result<(), EvmCoreError> {
    let mut set = RadixSet::try_new(4, 4)?;
    for value in [1, 2, 3] {
        let key = StructuralProbe(value);
        assert!(set.can_insert(key)?);
        set.insert_known_absent(key)?;
    }
    let checkpoint = set.checkpoint();
    let retained_nodes = set.nodes.len();
    let retained_undo = set.undo.len();

    set.restore(checkpoint)?;
    assert_eq!(set.nodes.len(), retained_nodes);
    assert_eq!(set.undo.len(), retained_undo);

    let key = StructuralProbe(4);
    assert!(set.can_insert(key)?);
    set.insert_known_absent(key)?;
    set.restore(checkpoint)?;
    assert_eq!(set.nodes.len(), retained_nodes);
    assert_eq!(set.undo.len(), retained_undo);
    for value in [1, 2, 3] {
        assert!(set.contains(StructuralProbe(value))?);
    }
    Ok(())
}

fn probe_bit(value: u8, index: usize) -> bool {
    const MASKS: [u8; 8] = [0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01];
    let mask = MASKS.get(index).copied().unwrap_or_default();
    value & mask != 0
}
