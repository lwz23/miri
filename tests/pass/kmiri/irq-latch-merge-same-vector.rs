// Three raises of one pending vector latch into the first event, without new IDs.
// Masking keeps all raises pending; unmasking must run the entry exactly once.
// The saved trace contains one queued, two merged into event 0, and one return.
//@compile-flags: -Zkmiri-toml=dev/test_utils/irq.toml

#[path = "../../../dev/test_utils/irq.rs"]
mod irq;

const TIMER_VECTOR: u8 = 40;
static mut RUNS: usize = 0;

extern "Rust" fn entry(vector: u8) {
    assert_eq!(vector, TIMER_VECTOR);
    unsafe { *core::ptr::addr_of_mut!(RUNS) += 1 };
}

fn main() {
    irq::register_entry(entry);
    let guard = irq::IrqGuard::new();
    irq::raise(TIMER_VECTOR);
    irq::raise(TIMER_VECTOR);
    irq::raise(TIMER_VECTOR);
    drop(guard);
    assert_eq!(unsafe { *core::ptr::addr_of_mut!(RUNS) }, 1);
}
