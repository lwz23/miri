// A pending vector whose priority class is higher than every in-service vector can nest.
// While 0x28 remains in service, 0x38 is one class higher and must enter at depth 2.
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
    if vector == LOW_VECTOR {
        irq::enable();
        irq::raise(HIGH_VECTOR);
        cross_block_boundary();
        assert_eq!(unsafe { *core::ptr::addr_of_mut!(POSITION) }, 3);
        irq::disable();
    }
    irq::eoi(vector);
    record(vector, Mark::Exit);
}

fn main() {
    irq::register_entry(entry);
    irq::raise(LOW_VECTOR);
    assert_eq!(
        unsafe { *core::ptr::addr_of_mut!(SEQUENCE) },
        [
            (LOW_VECTOR, Mark::Enter),
            (HIGH_VECTOR, Mark::Enter),
            (HIGH_VECTOR, Mark::Exit),
            (LOW_VECTOR, Mark::Exit),
        ]
    );
}
