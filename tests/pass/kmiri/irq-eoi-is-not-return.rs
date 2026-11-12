// EOI ends controller service, not the entry frame or the masked CPU context.
// Raise the same vector after EOI: single-active gating must still prevent reentry.
// Expect two entries, each with delivered < eoi < returned, and the second queued first.
//@compile-flags: -Zkmiri-toml=dev/test_utils/irq.toml

#[path = "../../../dev/test_utils/irq.rs"]
mod irq;

const TIMER_VECTOR: u8 = 40;

static mut RUNS: usize = 0;

extern "Rust" fn entry(vector: u8) {
    assert_eq!(vector, TIMER_VECTOR);
    let runs = unsafe {
        *core::ptr::addr_of_mut!(RUNS) += 1;
        *core::ptr::addr_of_mut!(RUNS)
    };
    irq::eoi(vector);
    assert!(!irq::is_enabled());
    if runs == 1 {
        irq::raise(TIMER_VECTOR);
        assert_eq!(unsafe { *core::ptr::addr_of_mut!(RUNS) }, 1);
    }
}

fn main() {
    irq::register_entry(entry);
    irq::raise(TIMER_VECTOR);
    assert_eq!(unsafe { *core::ptr::addr_of_mut!(RUNS) }, 2);
}
