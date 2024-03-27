use mte_measurement::{migrate_tags, MTEMode, set_mte_mode, set_tags_random};
use rand::random;
use std::hint::black_box;

const SIZE: usize = 512;

fn measure_custom(
    iters: u64,
    f: impl Fn(&[u8], &mut [u8]) -> (),
) -> std::time::Duration {
    let mut result = std::time::Duration::from_secs(0);

    let mut mem = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            SIZE,
            libc::PROT_READ | libc::PROT_WRITE | 0x20,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        )
    };
    assert_ne!(mem, libc::MAP_FAILED);
    unsafe {
        // fill with random values
        for i in 0..SIZE {
            std::ptr::write(mem.add(i) as *mut u8, random());
        }
        set_tags_random(std::slice::from_raw_parts_mut(mem as *mut u8, SIZE));
    }

    for _ in 0..iters {
        let new_mem = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                SIZE,
                libc::PROT_READ | libc::PROT_WRITE | 0x20,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };
        assert_ne!(new_mem, libc::MAP_FAILED);

        let mem_slice = unsafe { std::slice::from_raw_parts_mut(mem as *mut u8, SIZE) };
        let mut new_mem_slice = unsafe { std::slice::from_raw_parts_mut(new_mem as *mut u8, SIZE) };

        let start = std::time::Instant::now();
        f(&mem_slice, &mut new_mem_slice);
        result += start.elapsed();

        unsafe { libc::munmap(mem, SIZE) };
        mem = new_mem;
    }

    unsafe { libc::munmap(mem, SIZE) };

    result
}

fn main() {
    unsafe { set_mte_mode(MTEMode::Sync); }

    measure_custom(black_box(1), |from, to| unsafe {
        migrate_tags(from, to);
    });
    println!("Done!");
}
