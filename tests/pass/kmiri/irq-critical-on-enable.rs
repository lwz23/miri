// Critical-point injection raises a configured device vector exactly when IRQs
// go from disabled to enabled (the moment of `sti`). The config sets the rate to
// 1.0 and has no timer, so every `irq::enable()` queues vector 64, and it is
// delivered right after the call. Expect no IRQ while masked and exactly one
// delivery of 64 per enable.
//@compile-flags: -Zkmiri-toml=dev/test_utils/irq-critical.toml

#[path = "../../../dev/test_utils/irq.rs"]
mod irq;

const DEVICE_VECTOR: u8 = 64;

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

fn count() -> usize {
    unsafe { *core::ptr::addr_of_mut!(COUNT) }
}

fn main() {
    irq::register_entry(entry);
    for round in 1..=2 {
        irq::disable();
        assert_eq!(count(), round - 1);
        irq::enable();
        assert_eq!(count(), round);
    }
    assert_eq!(unsafe { *core::ptr::addr_of_mut!(RECEIVED) }, [DEVICE_VECTOR; 2]);
}
