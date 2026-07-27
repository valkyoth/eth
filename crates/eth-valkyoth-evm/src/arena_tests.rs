use eth_valkyoth_primitives::Address;

use crate::{BorrowedTransactionArena, HostCapabilityError, IterativeCallFrame, TransactionArena};

#[test]
fn iterative_frames_and_memory_remain_bounded() {
    let mut memory = [0xff; 8];
    let Some(mut arena) = BorrowedTransactionArena::<2>::try_new(&mut memory).ok() else {
        return;
    };
    let child = IterativeCallFrame {
        address: zero_address(),
        is_static: false,
    };
    assert_eq!(arena.enter_frame(child), Ok(1));
    assert_eq!(arena.enter_frame(child), Ok(2));
    assert_eq!(
        arena.enter_frame(child),
        Err(HostCapabilityError::CallDepthExceeded)
    );
    assert_eq!(arena.reset_transaction(), Ok(()));
    assert_eq!(arena.reserve_memory(8), Ok(()));
    assert_eq!(arena.memory(), &[0; 8]);
    assert_eq!(
        arena.reserve_memory(9),
        Err(HostCapabilityError::MemoryCapacityExceeded)
    );
}

#[test]
fn invalid_frame_profiles_fail_closed() {
    let mut memory = [0u8; 1];
    assert_eq!(
        BorrowedTransactionArena::<0>::try_new(&mut memory),
        Err(HostCapabilityError::InvalidFrameCapacity)
    );
}

fn zero_address() -> Address {
    Address::from_bytes(core::array::from_fn(|_| u8::default()))
}
