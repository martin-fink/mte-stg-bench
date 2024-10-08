use std::hint::black_box;
use mte_measurement::{memset, MTEMode, set_mte_mode};

// 128 MiB
const SIZE: usize = 128 * 1024 * 1024;

const ITERS: u32 = 50;

fn measure_custom(iters: u64, mode: MTEMode, f: impl Fn(&mut [u8]) -> ()) -> std::time::Duration {
    let mut result = std::time::Duration::from_secs(0);

    unsafe {
        set_mte_mode(mode);
    }

    for _ in 0..iters {
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

        assert_ne!(mem, libc::MAP_FAILED);
        let mut mem_slice = unsafe { std::slice::from_raw_parts_mut(mem as *mut u8, SIZE) };

        let start = std::time::Instant::now();
        f(&mut mem_slice);
        result += start.elapsed();

        unsafe { libc::munmap(mem, SIZE) };
    }

    result
}

fn main() {
    let modes = [
        MTEMode::None,
        MTEMode::Sync,
        MTEMode::Async,
    ];

    let mut results = Vec::new();

    for mode in modes {
        let result = measure_custom(ITERS.into(), mode, |mem| unsafe {
            memset(black_box(mem));
        });
        results.push(result / ITERS);
        std::thread::sleep(std::time::Duration::from_secs(15));
    }

    let result = results.iter().map(|result| result.as_millis())
        .map(|m| m.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    println!("[{}]", result);
}
