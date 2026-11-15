// IRQ entry runs with IF cleared; deep helper returns must not restore it early.
// Each recursive frame checks its own locals after its child returns. Main's
// scalars and array must survive the interrupt, which runs exactly once.
//@compile-flags: -Zkmiri-toml=dev/test_utils/irq.toml

#[path = "../../../dev/test_utils/irq.rs"]
mod irq;

const TIMER_VECTOR: u8 = 40;
static mut RUNS: usize = 0;

fn recurse(depth: u32) {
    let mut local = [depth, 0, depth + 2];
    local[1] = depth + 1;
    if depth != 0 {
        recurse(depth - 1);
    }
    assert_eq!(local, [depth, depth + 1, depth + 2]);
    assert!(!irq::is_enabled());
}

extern "Rust" fn entry(vector: u8) {
    assert_eq!(vector, TIMER_VECTOR);
    assert!(!irq::is_enabled());
    recurse(50);
    unsafe { *core::ptr::addr_of_mut!(RUNS) += 1 };
}

fn main() {
    irq::register_entry(entry);
    let number = 123u64;
    let signed = -42i32;
    let array = [7u32, 11, 13, 17];
    irq::raise(TIMER_VECTOR);
    assert!(irq::is_enabled());
    assert_eq!(number, 123);
    assert_eq!(signed, -42);
    assert_eq!(array, [7, 11, 13, 17]);
    assert_eq!(unsafe { *core::ptr::addr_of_mut!(RUNS) }, 1);
}
