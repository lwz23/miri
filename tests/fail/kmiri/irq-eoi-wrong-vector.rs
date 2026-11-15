// An entry may only acknowledge the vector currently in service.
// Deliver 40 but acknowledge 41; expect the not-in-service Abort.
//@error-in-other-file: kern_miri_irq_eoi: vector 41 is not in service on cpu 0
//@compile-flags: -Zkmiri-toml=dev/test_utils/irq.toml

#[path = "../../../dev/test_utils/irq.rs"]
mod irq;

const TIMER_VECTOR: u8 = 40;

extern "Rust" fn entry(vector: u8) {
    assert_eq!(vector, TIMER_VECTOR);
    irq::eoi(TIMER_VECTOR + 1);
}

fn main() {
    irq::register_entry(entry);
    irq::raise(TIMER_VECTOR);
}
