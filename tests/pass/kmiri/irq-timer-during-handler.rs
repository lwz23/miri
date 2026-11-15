// A timer event raised during an IRQ handler stays pending until the handler returns.
// Only the first entry runs the fixed loop, which exceeds the timer interval without
// invoking an IRQ API; repeating it would let the pending timer at return starve the
// interrupted main loop. The stderr trace must show timer-queued (or merged) before
// returned, then the next timer event delivered afterward.
//@compile-flags: -Zkmiri-toml=dev/test_utils/irq-timer-during-handler.toml

#[path = "../../../dev/test_utils/irq.rs"]
mod irq;

const TIMER_VECTOR: u8 = 40;
static mut RUNS: usize = 0;

#[inline(never)]
fn work(value: usize) -> usize {
    value.wrapping_mul(3).wrapping_add(1)
}

extern "Rust" fn entry(vector: u8) {
    assert_eq!(vector, TIMER_VECTOR);
    unsafe { *core::ptr::addr_of_mut!(RUNS) += 1 };
    let runs = unsafe { *core::ptr::addr_of_mut!(RUNS) };
    if runs == 1 {
        let mut value = 0;
        for _ in 0..256 {
            value = work(value);
        }
        assert_ne!(value, 0);
    }
    irq::eoi(vector);
}

fn main() {
    irq::register_entry(entry);
    irq::set_timer_vector(TIMER_VECTOR);
    while unsafe { *core::ptr::addr_of_mut!(RUNS) } < 2 {
        core::hint::spin_loop();
    }
}
