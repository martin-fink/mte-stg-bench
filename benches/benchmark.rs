use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mte_measurement::{memset, st2g, st2g_zero, stg, stgp, stg_zero, stzg};
use rand::random;

// 128 MiB
const SIZE: usize = 256 * 1024 * 1024;

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
    // first we enable MTE for the current process
    #[cfg(any(target_os = "linux", target_os = "android"))]
    unsafe {
        const PR_SET_TAGGED_ADDR_CTRL: i32 = 55;
        const PR_TAGGED_ADDR_ENABLE: u64 = 1 << 0;
        const PR_MTE_TCF_SHIFT: i32 = 1;
        const PR_MTE_TAG_SHIFT: i32 = 3;

        assert_eq!(
            libc::prctl(
                PR_SET_TAGGED_ADDR_CTRL,
                PR_TAGGED_ADDR_ENABLE
                | (0x1 << PR_MTE_TCF_SHIFT) // sync mte
                | (0x0 << PR_MTE_TAG_SHIFT), // no excluded tags for irg
                0,
                0,
                0,
            ),
            0,
            "could not enable mte"
        );
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
