use criterion::{criterion_group, criterion_main, Criterion};
use mte_measurement::{migrate_mte_off, migrate_tags, set_mte_mode, set_tags_random, MTEMode, set_mte_mode_tags};
use rand::random;

// 128 MiB
const SIZE: usize = 128 * 1024 * 1024;

fn measure_custom(
    iters: u64,
    f: impl Fn(&[u8], &mut [u8]) -> (),
    mte: bool,
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
        if mte {
            set_tags_random(std::slice::from_raw_parts_mut(mem as *mut u8, SIZE));
        }
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

pub fn criterion_benchmark_migrate(c: &mut Criterion) {
    unsafe {
        set_mte_mode(MTEMode::Sync);
    }

    // c.bench_function("migrate", |b| {
    //     unsafe {
    //         set_mte_mode(MTEMode::None);
    //     }
    //     b.iter_custom(|iters| {
    //         let result = measure_custom(iters, |from, to| to.copy_from_slice(from), true);
    //         result
    //     });
    //
    //     unsafe {
    //         set_mte_mode(MTEMode::Sync);
    //     }
    // });
    c.bench_function("migrate", |b| {
        unsafe {
            set_mte_mode_tags(MTEMode::Sync, 0x0000);
        }
        b.iter_custom(|iters| {
            let result = measure_custom(iters, |from, to| to.copy_from_slice(from), false);
            result
        });
        unsafe {
            set_mte_mode(MTEMode::Sync);
        }
    });
    c.bench_function("migrate_disable_enable", |b| {
        b.iter_custom(|iters| {
            measure_custom(iters, |from, to| unsafe { migrate_mte_off(from, to) }, true)
        })
    });
    c.bench_function("migrate_tags", |b| {
        b.iter_custom(|iters| {
            measure_custom(iters, |from, to| unsafe { migrate_tags(from, to) }, true)
        })
    });
}

criterion_group!(benches, criterion_benchmark_migrate);
criterion_main!(benches);
