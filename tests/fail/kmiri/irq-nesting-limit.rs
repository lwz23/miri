// Reaching the configured nesting depth with an acceptable pending IRQ is a diagnostic.
// Each layer EOIs before raising the next vector, so the third entry is acceptable
// and must Abort instead of being silently held pending.
//@error-in-other-file: IRQ nesting limit reached
//@compile-flags: -Zkmiri-toml=dev/test_utils/irq.toml
#[path = "../../../dev/test_utils/irq.rs"]
mod irq;
const TIMER_VECTOR: u8 = 40;
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Mark {
    Enter,
    Exit,
}
static mut SEQ: [(u8, Mark); 6] = [(0, Mark::Enter); 6];
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
    irq::eoi(v);
    if v < 42 {
        irq::enable();
        irq::raise(v + 1);
        work();
        irq::disable();
    }
    record(v, Mark::Exit);
}
fn main() {
    irq::register_entry(entry);
    irq::raise(TIMER_VECTOR);
    assert_eq!(unsafe { *core::ptr::addr_of_mut!(POS) }, 6);
    assert_eq!(
        unsafe { *core::ptr::addr_of_mut!(SEQ) },
        [
            (40, Mark::Enter),
            (41, Mark::Enter),
            (41, Mark::Exit),
            (42, Mark::Enter),
            (42, Mark::Exit),
            (40, Mark::Exit)
        ]
    );
}
