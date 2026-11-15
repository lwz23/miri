// EOI permits a different pending vector, but interrupts remain masked in the entry.
//@compile-flags: -Zkmiri-toml=dev/test_utils/irq.toml
#[path = "../../../dev/test_utils/irq.rs"]
mod irq;
const TIMER_VECTOR: u8 = 40;
const OTHER_VECTOR: u8 = 41;
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Mark {
    Enter,
    Exit,
}
static mut SEQ: [(u8, Mark); 4] = [(0, Mark::Enter); 4];
static mut POS: usize = 0;
fn record(v: u8, m: Mark) {
    unsafe {
        let p = core::ptr::addr_of_mut!(POS);
        let i = *p;
        (*core::ptr::addr_of_mut!(SEQ))[i] = (v, m);
        *p = i + 1;
    }
}
extern "Rust" fn entry(v: u8) {
    record(v, Mark::Enter);
    irq::eoi(v);
    if v == TIMER_VECTOR {
        irq::raise(OTHER_VECTOR);
    }
    record(v, Mark::Exit);
}
fn main() {
    irq::register_entry(entry);
    irq::raise(TIMER_VECTOR);
    assert_eq!(unsafe { *core::ptr::addr_of_mut!(POS) }, 4);
    assert_eq!(
        unsafe { *core::ptr::addr_of_mut!(SEQ) },
        [(40, Mark::Enter), (40, Mark::Exit), (41, Mark::Enter), (41, Mark::Exit)]
    );
}
