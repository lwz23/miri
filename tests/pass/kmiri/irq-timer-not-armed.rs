// The timer must remain silent before the guest arms it with set_timer_vector.
// This loop crosses many 64-block tick boundaries, but no entry may run and no
// timer event may be generated while the timer vector is unset.
//@compile-flags: -Zkmiri-toml=dev/test_utils/irq-timer.toml

#[path = "../../../dev/test_utils/irq.rs"]
mod irq;

static mut RUNS: usize = 0;

#[inline(never)]
fn work(value: usize) -> usize {
    value.wrapping_mul(3).wrapping_add(1)
}

extern "Rust" fn entry(_vector: u8) {
    unsafe { *core::ptr::addr_of_mut!(RUNS) += 1 };
}

fn main() {
    irq::register_entry(entry);
    let mut value = 0;
    for _ in 0..256 {
        value = work(value);
    }
    assert_ne!(value, 0);
    assert_eq!(unsafe { *core::ptr::addr_of_mut!(RUNS) }, 0);
}
