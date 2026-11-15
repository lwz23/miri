// An active vector is not pending: raising it inside its entry creates a new event.
// The single-active gate prevents reentry until the first entry returns.
// Expect delivered(0), queued(1), returned(0), delivered(1), returned(1), no merge.
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
    if runs == 1 {
        irq::raise(TIMER_VECTOR);
        assert_eq!(unsafe { *core::ptr::addr_of_mut!(RUNS) }, 1);
    }
    irq::eoi(vector);
}

fn main() {
    irq::register_entry(entry);
    irq::raise(TIMER_VECTOR);
    assert_eq!(unsafe { *core::ptr::addr_of_mut!(RUNS) }, 2);
}
