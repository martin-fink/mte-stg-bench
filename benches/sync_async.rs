use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mte_measurement::memset;

// 128 MiB
const SIZE: usize = 256 * 1024 * 1024;

/// In which mode MTE should be enabled
#[derive(Copy, Clone)]
pub enum MTEMode {
    /// Ignore tag check faults
    None,
    /// Synchronous tag check fault mode
    Sync,
    /// Asynchronous tag check fault mode
    Async,
}

impl MTEMode {
    fn mask(&self) -> u64 {
        const PR_MTE_TCF_SHIFT: i32 = 1;
        match self {
            MTEMode::None => 0,
            MTEMode::Sync => 1u64 << PR_MTE_TCF_SHIFT,
            MTEMode::Async => 2u64 << PR_MTE_TCF_SHIFT,
        }
    }
}

fn measure_custom(iters: u64, mode: MTEMode, f: impl Fn(&mut [u8]) -> ()) -> std::time::Duration {
    let mut result = std::time::Duration::from_secs(0);

    #[cfg(any(target_os = "linux", target_os = "android"))]
    unsafe {
        const PR_SET_TAGGED_ADDR_CTRL: i32 = 55;
        const PR_TAGGED_ADDR_ENABLE: u64 = 1 << 0;
        const PR_MTE_TAG_SHIFT: i32 = 3;

        assert_eq!(
            libc::prctl(
                PR_SET_TAGGED_ADDR_CTRL,
                PR_TAGGED_ADDR_ENABLE | mode.mask() | (0x0 << PR_MTE_TAG_SHIFT), // no excluded tags for irg
                0,
                0,
                0,
            ),
            0,
            "could not enable mte"
        );
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
