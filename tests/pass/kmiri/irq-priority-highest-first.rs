// LAPIC arbitration must deliver the highest pending vector first.
// Raising 0x28, 0x48, and 0x38 while masked creates three distinct pending
// vectors; after unmasking, the expected order is therefore 0x48, 0x38, 0x28.
//@compile-flags: -Zkmiri-toml=dev/test_utils/irq.toml

#[path = "../../../dev/test_utils/irq.rs"]
mod irq;

const LOW_VECTOR: u8 = 0x28;
const MIDDLE_VECTOR: u8 = 0x38;
const HIGH_VECTOR: u8 = 0x48;

static mut RECEIVED: [u8; 3] = [0; 3];
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
    {
        let _guard = irq::IrqGuard::new();
        irq::raise(LOW_VECTOR);
        irq::raise(HIGH_VECTOR);
        irq::raise(MIDDLE_VECTOR);
    }
    assert_eq!(unsafe { *core::ptr::addr_of_mut!(COUNT) }, 3);
    assert_eq!(
        unsafe { *core::ptr::addr_of_mut!(RECEIVED) },
        [HIGH_VECTOR, MIDDLE_VECTOR, LOW_VECTOR]
    );
}
