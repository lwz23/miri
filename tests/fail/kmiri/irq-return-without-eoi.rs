// Returning from the entry must not implicitly acknowledge the controller.
// Deliberately omit EOI; the entry-frame return hook must abort.
//@error-in-other-file: IRQ entry returned without EOI (event=0, vector=40)
//@compile-flags: -Zkmiri-toml=dev/test_utils/irq.toml

#[path = "../../../dev/test_utils/irq.rs"]
mod irq;

const TIMER_VECTOR: u8 = 40;

extern "Rust" fn entry(vector: u8) {
    assert_eq!(vector, TIMER_VECTOR);
}

fn main() {
    irq::register_entry(entry);
    irq::raise(TIMER_VECTOR);
}
