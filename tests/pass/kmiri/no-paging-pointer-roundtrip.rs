//@compile-flags: -Cpanic=abort -Zkmiri-toml=tests/pass/kmiri/no-paging-pointer-roundtrip.toml
#![no_std]
#![no_main]
unsafe extern "Rust" { fn miri_write_to_stdout(bytes: &[u8]); }
unsafe extern "C" { fn abort() -> !; }
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! { unsafe { abort() } }
static mut BYTES: [u8;8] = [0;8];
#[unsafe(no_mangle)]
fn miri_start(_: isize, _: *const *const u8) -> isize {
    unsafe { miri_write_to_stdout(b"ROUNDTRIP_ENTER\n"); }
    let mut bytes=[0u8;8];
    let pointer=core::ptr::addr_of_mut!(bytes).cast::<u8>();
    let address=pointer as usize;
    unsafe {
        (address as *mut u8).add(3).write(42);
        assert_eq!(pointer.add(3).read(),42);
        miri_write_to_stdout(b"ROUNDTRIP_STACK_OK\n");
        let pointer=core::ptr::addr_of_mut!(BYTES).cast::<u8>();
        let address=pointer as usize;
        (address as *mut u8).add(3).write(23);
        assert_eq!(pointer.add(3).read(),23);
        miri_write_to_stdout(b"ROUNDTRIP_STATIC_OK\n");
    }
    0
}
