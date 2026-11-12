// A lower-priority-class vector cannot interrupt an in-service higher-class vector.
// Pending 0x28 therefore waits until the 0x38 entry returns and restores IF.
//@compile-flags: -Zkmiri-toml=dev/test_utils/irq.toml

#[path = "../../../dev/test_utils/irq.rs"]
mod irq;

const LOW_VECTOR: u8 = 0x28;
const HIGH_VECTOR: u8 = 0x38;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Mark {
    Enter,
    Exit,
}

static mut SEQUENCE: [(u8, Mark); 4] = [(0, Mark::Enter); 4];
static mut POSITION: usize = 0;

fn record(vector: u8, mark: Mark) {
    unsafe {
        let position = *core::ptr::addr_of_mut!(POSITION);
        (*core::ptr::addr_of_mut!(SEQUENCE))[position] = (vector, mark);
        *core::ptr::addr_of_mut!(POSITION) = position + 1;
    }
}

fn cross_block_boundary() {
    let mut value = 0;
    value += 1;
    assert_eq!(value, 1);
}

extern "Rust" fn entry(vector: u8) {
    record(vector, Mark::Enter);
    if vector == HIGH_VECTOR {
        irq::enable();
        irq::raise(LOW_VECTOR);
        cross_block_boundary();
        assert_eq!(unsafe { *core::ptr::addr_of_mut!(POSITION) }, 1);
        irq::disable();
    }
    irq::eoi(vector);
    record(vector, Mark::Exit);
}

fn main() {
    irq::register_entry(entry);
    irq::raise(HIGH_VECTOR);
    assert_eq!(
        unsafe { *core::ptr::addr_of_mut!(SEQUENCE) },
        [
            (HIGH_VECTOR, Mark::Enter),
            (HIGH_VECTOR, Mark::Exit),
            (LOW_VECTOR, Mark::Enter),
            (LOW_VECTOR, Mark::Exit),
        ]
    );
}
