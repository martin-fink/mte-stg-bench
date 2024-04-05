use std::hint::black_box;
use mte_measurement::{memset, MTEMode, set_mte_mode};

// 64 KiB -- should fit into L1 cache of Cortex X3 (128 KiB L1 cache)
const SIZE: usize = 64 * 1024;

fn measure_custom(iters: u64, mode: MTEMode, f: impl Fn(&mut [u8]) -> ()) {
    unsafe { set_mte_mode(mode) };

    let mem = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            SIZE,
            libc::PROT_READ | libc::PROT_WRITE | 0x20,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        )
    };

    for _ in 0..iters {
        assert_ne!(mem, libc::MAP_FAILED);
        let mut mem_slice = unsafe { std::slice::from_raw_parts_mut(mem as *mut u8, SIZE) };

        f(&mut mem_slice);
    }

    unsafe { libc::munmap(mem, SIZE) };
}

fn main() {
    measure_custom(black_box(500000), MTEMode::None, |mem| unsafe {
        memset(black_box(mem))
    });
}
