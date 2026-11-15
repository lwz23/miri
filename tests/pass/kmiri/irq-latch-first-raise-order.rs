// Latching a repeated vector must not move it behind a later distinct vector.
// Masked raises 40, 41, 40 must deliver [40, 41], with the last raise merged
// into event 0. No automatic timer is configured.
//@compile-flags: -Zkmiri-toml=dev/test_utils/irq.toml

#[path = "../../../dev/test_utils/irq.rs"]
mod irq;

const TIMER_VECTOR: u8 = 40;
static mut RECEIVED: [u8; 2] = [0; 2];
static mut COUNT: usize = 0;

extern "Rust" fn entry(vector: u8) {
    unsafe {
        let count = *core::ptr::addr_of_mut!(COUNT);
        (*core::ptr::addr_of_mut!(RECEIVED))[count] = vector;
        *core::ptr::addr_of_mut!(COUNT) = count + 1;
    }
    irq::eoi(vector);
}

fn main() {
    irq::register_entry(entry);
    let guard = irq::IrqGuard::new();
    irq::raise(TIMER_VECTOR);
    irq::raise(TIMER_VECTOR + 1);
    irq::raise(TIMER_VECTOR);
    drop(guard);
    assert_eq!(unsafe { *core::ptr::addr_of_mut!(COUNT) }, 2);
    assert_eq!(unsafe { *core::ptr::addr_of_mut!(RECEIVED) }, [TIMER_VECTOR, TIMER_VECTOR + 1]);
}
