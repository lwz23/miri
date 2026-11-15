// A panic must not unwind out of an IRQ entry into the interrupted task.
// Use the normal Rust panic path and expect the entry-frame Abort diagnostic.
//@compile-flags: -Zkmiri-toml=dev/test_utils/irq.toml
//@error-in-other-file: panic unwound out of IRQ entry

#[path = "../../../dev/test_utils/irq.rs"]
mod irq;

const TIMER_VECTOR: u8 = 40;

extern "Rust" fn entry(vector: u8) {
    assert_eq!(vector, TIMER_VECTOR);
    panic!("IRQ entry panic");
}

fn main() {
    irq::register_entry(entry);
    irq::raise(TIMER_VECTOR);
}
