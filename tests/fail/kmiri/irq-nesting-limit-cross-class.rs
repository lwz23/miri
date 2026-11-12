// An acceptable cross-class IRQ at the configured nesting depth must Abort.
// 0x48 has a higher class than both in-service 0x28 and 0x38, so silently
// holding it pending would hide the unbounded nesting seen in issue #1648.
//@error-in-other-file: IRQ nesting limit reached
//@compile-flags: -Zkmiri-toml=dev/test_utils/irq.toml

#[path = "../../../dev/test_utils/irq.rs"]
mod irq;

const LOW_VECTOR: u8 = 0x28;
const MIDDLE_VECTOR: u8 = 0x38;
const HIGH_VECTOR: u8 = 0x48;

fn cross_block_boundary() {
    let mut value = 0;
    value += 1;
    assert_eq!(value, 1);
}

extern "Rust" fn entry(vector: u8) {
    if vector == LOW_VECTOR {
        irq::enable();
        irq::raise(MIDDLE_VECTOR);
        cross_block_boundary();
        irq::disable();
    } else if vector == MIDDLE_VECTOR {
        irq::enable();
        irq::raise(HIGH_VECTOR);
        cross_block_boundary();
        irq::disable();
    }
}

fn main() {
    irq::register_entry(entry);
    irq::raise(LOW_VECTOR);
}
