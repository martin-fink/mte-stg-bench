use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mte_measurement::{memset, set_mte_mode, MTEMode};

// 128 MiB
const SIZE: usize = 128 * 1024 * 1024;

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

pub fn criterion_benchmark_stg(c: &mut Criterion) {
    c.bench_function("none", |b| {
        b.iter_custom(|iters| {
            measure_custom(iters, MTEMode::None, |mem| unsafe {
                memset(black_box(mem))
            })
        })
    });
    c.bench_function("sync", |b| {
        b.iter_custom(|iters| {
            measure_custom(iters, MTEMode::Sync, |mem| unsafe {
                memset(black_box(mem))
            })
        })
    });
    c.bench_function("async", |b| {
        b.iter_custom(|iters| {
            measure_custom(iters, MTEMode::Async, |mem| unsafe {
                memset(black_box(mem))
            })
        })
    });
}

criterion_group!(benches, criterion_benchmark_stg);
criterion_main!(benches);
