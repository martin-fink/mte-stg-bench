use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mte_measurement::{
    memset, set_mte_mode, st2g, st2g_zero, stg, stg_zero, stgp, stz2g, stzg, MTEMode,
};
use rand::random;

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

pub fn criterion_benchmark_stg(c: &mut Criterion) {
    unsafe {
        set_mte_mode(MTEMode::Sync);
    }

    c.bench_function("memset", |b| {
        b.iter_custom(|iters| measure_custom(iters, |mem| unsafe { memset(black_box(mem)) }))
    });
    c.bench_function("stg", |b| {
        b.iter_custom(|iters| {
            measure_custom(iters, |mem| unsafe {
                stg(black_box(mem), black_box(random()))
            })
        })
    });
    c.bench_function("stgp", |b| {
        b.iter_custom(|iters| {
            measure_custom(iters, |mem| unsafe {
                stgp(black_box(mem), black_box(random()))
            })
        })
    });
    c.bench_function("st2g", |b| {
        b.iter_custom(|iters| {
            measure_custom(iters, |mem| unsafe {
                st2g(black_box(mem), black_box(random()))
            })
        })
    });
    c.bench_function("stzg", |b| {
        b.iter_custom(|iters| {
            measure_custom(iters, |mem| unsafe {
                stzg(black_box(mem), black_box(random()))
            })
        })
    });
    c.bench_function("stz2g", |b| {
        b.iter_custom(|iters| {
            measure_custom(iters, |mem| unsafe {
                stz2g(black_box(mem), black_box(random()))
            })
        })
    });
    c.bench_function("stg+memset", |b| {
        b.iter_custom(|iters| {
            measure_custom(iters, |mem| unsafe {
                stg_zero(black_box(mem), black_box(random()))
            })
        })
    });
    c.bench_function("st2g+memset", |b| {
        b.iter_custom(|iters| {
            measure_custom(iters, |mem| unsafe {
                st2g_zero(black_box(mem), black_box(random()))
            })
        })
    });
}

criterion_group!(benches, criterion_benchmark_stg);
criterion_main!(benches);
