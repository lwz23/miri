//@compile-flags: -Zkmiri-toml=dev/test_utils/physical-copy.toml -Zmiri-permissive-provenance

//! `kern_miri_copy` between two Rust objects behaves like `ptr::copy`: both
//! sides are interpreter memory and are accessed through their addresses.
//!
//! Expected: `dst` equals `src`.

#[path = "../../../dev/test_utils/physical_copy.rs"]
mod utils;

use utils::*;

fn main() {
    let src: [u64; 4] = [1, 2, 3, 4];
    let mut dst = [0u64; 4];
    unsafe {
        kern_miri_copy(
            dst.as_mut_ptr().expose_provenance(),
            src.as_ptr().expose_provenance(),
            size_of_val(&src),
        );
    }
    assert_eq!(dst, src);
}
