// EOI, re-enable interrupts, and raise the same vector to exercise nesting.
//@compile-flags: -Zkmiri-toml=dev/test_utils/irq.toml
#[path = "../../../dev/test_utils/irq.rs"]
mod irq;
const TIMER_VECTOR: u8 = 40;
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Mark {
    Enter,
    Exit,
}
static mut SEQ: [(u8, Mark); 4] = [(0, Mark::Enter); 4];
static mut POS: usize = 0;
fn record(vector: u8, mark: Mark) {
    unsafe {
        let p = core::ptr::addr_of_mut!(POS);
        let i = *p;
        (*core::ptr::addr_of_mut!(SEQ))[i] = (vector, mark);
        *p = i + 1;
    }
}
extern "Rust" fn entry(vector: u8) {
    record(vector, Mark::Enter);
    irq::eoi(vector);
    unsafe {
        if *core::ptr::addr_of_mut!(POS) == 1 {
            irq::enable();
            irq::raise(TIMER_VECTOR);
            helper();
            irq::disable();
        }
    }
    record(vector, Mark::Exit);
}
fn helper() {
    let mut x = 0;
    x += 1;
    assert_eq!(x, 1);
}
fn main() {
    irq::register_entry(entry);
    irq::raise(TIMER_VECTOR);
    assert_eq!(unsafe { *core::ptr::addr_of_mut!(POS) }, 4);
    assert_eq!(
        unsafe { *core::ptr::addr_of_mut!(SEQ) },
        [(40, Mark::Enter), (40, Mark::Enter), (40, Mark::Exit), (40, Mark::Exit)]
    );
}
