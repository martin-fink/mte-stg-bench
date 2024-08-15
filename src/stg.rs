use std::hint::black_box;
use rand::random;
use mte_measurement::{memset, MTEMode, set_mte_mode, st2g, st2g_zero, stg, stg_zero, stgp, stz2g, stzg};

// 128 MiB
const SIZE: usize = 128 * 1024 * 1024;

fn measure_custom(iters: u64, f: impl Fn(&mut [u8]) -> ()) -> std::time::Duration {
    let mut result = std::time::Duration::from_secs(0);

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
    unsafe {
        set_mte_mode(MTEMode::Sync);
    }
    let fns: [(&'static str, Box<dyn Fn(&mut [u8]) -> ()>); 8] = [
        ("memset", Box::new(|mem| unsafe { memset(black_box(mem)) })),
        ("stg", Box::new(|mem| unsafe { stg(black_box(mem), black_box(random())) })),
        ("stgp", Box::new(|mem| unsafe { stgp(black_box(mem), black_box(random())) })),
        ("st2g", Box::new(|mem| unsafe { st2g(black_box(mem), black_box(random())) })),
        ("stzg", Box::new(|mem| unsafe { stzg(black_box(mem), black_box(random())) })),
        ("stz2g", Box::new(|mem| unsafe { stz2g(black_box(mem), black_box(random())) })),
        ("stg+memset", Box::new(|mem| unsafe { stg_zero(black_box(mem), black_box(random())) })),
        ("st2g+memset", Box::new(|mem| unsafe { st2g_zero(black_box(mem), black_box(random())) })),
    ];

    let mut results = Vec::new();

    for (_, f) in fns.iter() {
        let result = measure_custom(50, f);
        results.push(result);
        std::thread::sleep(std::time::Duration::from_secs(15));
    }

    let result = results.iter().map(|result| result.as_millis())
        .map(|m| m.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    println!("[{}]", result);
}
