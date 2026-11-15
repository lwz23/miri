// Queue two distinct vectors while masked, then verify highest-vector-first dispatch.
// Explicit raises make the ordering deterministic; no automatic timer is configured.
// LAPIC arbitration must deliver 41 before 40 through the one registered entry.
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
    {
        let _guard = irq::IrqGuard::new();
        irq::raise(TIMER_VECTOR);
        irq::raise(TIMER_VECTOR + 1);
    }
    assert_eq!(unsafe { *core::ptr::addr_of_mut!(COUNT) }, 2);
    assert_eq!(unsafe { *core::ptr::addr_of_mut!(RECEIVED) }, [41, 40]);
}
