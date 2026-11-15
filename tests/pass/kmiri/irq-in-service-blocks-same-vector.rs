// An in-service vector blocks reentry until EOI, then the pending vector is accepted.
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
fn record(v: u8, m: Mark) {
    unsafe {
        let p = core::ptr::addr_of_mut!(POS);
        let i = *p;
        (*core::ptr::addr_of_mut!(SEQ))[i] = (v, m);
        *p = i + 1;
    }
}
fn work() {
    let mut x = 0;
    x += 1;
    assert_eq!(x, 1);
}
extern "Rust" fn entry(v: u8) {
    record(v, Mark::Enter);
    if unsafe { *core::ptr::addr_of_mut!(POS) } == 1 {
        irq::enable();
        irq::raise(TIMER_VECTOR);
        work();
        assert_eq!(unsafe { *core::ptr::addr_of_mut!(POS) }, 1);
        irq::eoi(v);
        work();
        assert_eq!(unsafe { *core::ptr::addr_of_mut!(POS) }, 3);
        irq::disable();
    } else {
        irq::eoi(v);
    }
    record(v, Mark::Exit);
}
fn main() {
    irq::register_entry(entry);
    irq::raise(TIMER_VECTOR);
    assert_eq!(
        unsafe { *core::ptr::addr_of_mut!(SEQ) },
        [(40, Mark::Enter), (40, Mark::Enter), (40, Mark::Exit), (40, Mark::Exit)]
    );
}
