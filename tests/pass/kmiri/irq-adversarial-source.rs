// The adversarial source raises configured device vectors at random basic-block
// boundaries, whether or not the kernel has a handler for them. The config uses
// a fixed seed, so the run is deterministic. Expect no delivery while IRQs are
// masked (a raise there stays pending), at least one delivery during the loop
// with IRQs enabled, and only vector 64.
//@compile-flags: -Zkmiri-toml=dev/test_utils/irq-adversarial.toml

#[path = "../../../dev/test_utils/irq.rs"]
mod irq;

const DEVICE_VECTOR: u8 = 64;

static mut COUNT: usize = 0;

extern "Rust" fn entry(vector: u8) {
    assert_eq!(vector, DEVICE_VECTOR);
    unsafe { *core::ptr::addr_of_mut!(COUNT) += 1 };
    irq::eoi(vector);
}

fn count() -> usize {
    unsafe { *core::ptr::addr_of_mut!(COUNT) }
}

fn spin(iterations: usize) {
    for _ in 0..iterations {
        std::hint::black_box(());
    }
}

fn main() {
    irq::register_entry(entry);
    {
        let _guard = irq::IrqGuard::new();
        spin(200);
        assert_eq!(count(), 0);
    }
    let before = count();
    spin(1000);
    assert!(count() > before);
}
