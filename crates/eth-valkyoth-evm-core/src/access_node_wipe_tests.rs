use core::sync::atomic::{AtomicUsize, Ordering};

use super::{AccessKey, AvlSet};
use crate::EvmCoreError;

static WIPE_CALLS: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct WipeProbe(u8);

impl AccessKey for WipeProbe {
    fn wipe(&mut self) {
        self.0 = 0;
        let _ = WIPE_CALLS.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn rollback_clear_and_drop_wipe_each_removed_live_key() -> Result<(), EvmCoreError> {
    WIPE_CALLS.store(0, Ordering::SeqCst);
    {
        let mut set = AvlSet::try_new(2, 2)?;
        set.insert_known_absent(WipeProbe(1))?;
        set.insert_known_absent(WipeProbe(2))?;
        set.restore_len(1)?;
        assert_eq!(WIPE_CALLS.load(Ordering::SeqCst), 1);

        set.clear();
        assert_eq!(WIPE_CALLS.load(Ordering::SeqCst), 2);
        set.insert_known_absent(WipeProbe(3))?;
    }
    assert_eq!(WIPE_CALLS.load(Ordering::SeqCst), 3);
    Ok(())
}
