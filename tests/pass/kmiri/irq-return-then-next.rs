// Two distinct masked vectors remain two pending events under vector latching.
// After unmasking, each entry must return before the next is delivered; the
// higher vector (41) is delivered first.
// The saved stderr checks delivered(1), returned(1), delivered(0), returned(0).
//@compile-flags: -Zkmiri-toml=dev/test_utils/irq.toml

#[path = "../../../dev/test_utils/irq.rs"]
mod irq;

const TIMER_VECTOR: u8 = 40;
static mut RUNS: usize = 0;

extern "Rust" fn entry(vector: u8) {
    assert!(vector == TIMER_VECTOR || vector == TIMER_VECTOR + 1);
    unsafe { *core::ptr::addr_of_mut!(RUNS) += 1 };
    irq::eoi(vector);
}

fn main() {
    irq::register_entry(entry);
    let guard = irq::IrqGuard::new();
    irq::raise(TIMER_VECTOR);
    irq::raise(TIMER_VECTOR + 1);
    drop(guard);
    assert_eq!(unsafe { *core::ptr::addr_of_mut!(RUNS) }, 2);
}
