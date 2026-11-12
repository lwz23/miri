// EOI with no delivered IRQ must abort rather than silently do nothing.
// No entry is registered or raised, isolating the controller service check.
//@error-in-other-file: kern_miri_irq_eoi: vector 40 is not in service on cpu 0
//@compile-flags: -Zkmiri-toml=dev/test_utils/irq.toml

#[path = "../../../dev/test_utils/irq.rs"]
mod irq;

const TIMER_VECTOR: u8 = 40;

fn main() {
    irq::eoi(TIMER_VECTOR);
}
