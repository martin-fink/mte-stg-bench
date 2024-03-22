use std::arch::asm;

#[inline]
fn set_tag(addr: *mut u8, tag: u64) -> *mut u8 {
    let tag = tag & 0x0f00_0000_0000_0000;
    (((addr as u64) & 0x0000_ffff_ffff_ffff) | tag) as *mut u8
}

pub unsafe fn stg(mem: &mut [u8], tag: u64) {
    assert_eq!(mem.len() % 32, 0);

    let mut index = mem.as_mut_ptr();
    let end = index.add(mem.len());

    while index != end {
        asm!("stg {tag}, [{index}], #16", tag = in(reg) tag, index = inout(reg) index);
    }
}

pub unsafe fn stg_prefetch(mem: &mut [u8], tag: u64) {
    assert_eq!(mem.len() % 32, 0);

    let index = mem.as_mut_ptr();
    let end = index.add(mem.len());

    let line_size = 0u64;
    let tmp = 0u64;
    let next = 0u64;

    // The following code is a modified version of the code from the Android Scudo project
    // https://android.googlesource.com/platform/external/scudo/+/refs/tags/android-14.0.0_r1/standalone/memtag.h#167
    asm! {
        "DCZID .req {tmp}",
        "mrs DCZID, dczid_el0",
        "tbnz DCZID, #4, 4f",
        "and DCZID, DCZID, #15",
        "mov {line_size}, #4",
        "lsl {line_size}, {line_size}, DCZID",
        ".unreq DCZID",

        "Size .req {tmp}",
        "sub Size, {end}, {index}",
        "cmp Size, {line_size}, lsl #1",
        "b.lt 4f",
        ".unreq Size",

        "LineMask .req {tmp}",
        "sub LineMask, {line_size}, #1",

        "orr {next}, {index}, LineMask",

        "2:",
        "stg {tag}, [{index}], #16",
        "cmp {index}, {next}",
        "b.lt 2b",

        "bic {next}, {end}, LineMask",
        ".unreq LineMask",

        "3:",
        "dc gzva, {index}",
        "add {index}, {index}, {line_size}",
        "cmp {index}, {next}",
        "b.lt 3b",

        "4:",
        "cmp {index}, {end}",
        "b.ge 5f",
        "stg {tag}, [{index}], #16",
        "b 4b",

        "5:",

        tmp = in(reg) tmp,
        line_size = in(reg) line_size,
        next = in(reg) next,
        index = in(reg) index,
        end = in(reg) end,
        tag = in(reg) tag,
    };
}

pub unsafe fn stg_zero(mem: &mut [u8], tag: u64) {
    assert_eq!(mem.len() % 32, 0);

    let mut index = mem.as_mut_ptr();
    let end = index.add(mem.len());

    while index != end {
        asm!("stg {tag}, [{index}], #16", tag = in(reg) tag, index = inout(reg) index);
    }

    let ptr = set_tag(mem.as_mut_ptr(), tag);
    std::slice::from_raw_parts_mut(ptr, mem.len()).fill(0);
}

pub unsafe fn stgp(mem: &mut [u8], tag: u64) {
    assert_eq!(mem.len() % 32, 0);

    let mut index = set_tag(mem.as_mut_ptr(), tag);
    let end = index.add(mem.len());

    let zero = 0u64;

    while index != end {
        asm!("stgp {zero}, {zero}, [{index}], #16", zero = in(reg) zero, index = inout(reg) index);
    }
}

pub unsafe fn st2g(mem: &mut [u8], tag: u64) {
    assert_eq!(mem.len() % 32, 0);

    let mut index = mem.as_mut_ptr();
    let end = index.add(mem.len());

    while index != end {
        asm!("st2g {tag}, [{index}], #32", tag = in(reg) tag, index = inout(reg) index);
    }
}

pub unsafe fn st2g_zero(mem: &mut [u8], tag: u64) {
    assert_eq!(mem.len() % 32, 0);

    let mut index = mem.as_mut_ptr();
    let end = index.add(mem.len());

    while index != end {
        asm!("st2g {tag}, [{index}], #32", tag = in(reg) tag, index = inout(reg) index);
    }

    let ptr = set_tag(mem.as_mut_ptr(), tag);
    std::slice::from_raw_parts_mut(ptr, mem.len()).fill(0);
}

pub unsafe fn stzg(mem: &mut [u8], tag: u64) {
    assert_eq!(mem.len() % 32, 0);

    let mut index = mem.as_mut_ptr();
    let end = index.add(mem.len());

    while index != end {
        asm!("stzg {tag}, [{index}], #16", tag = in(reg) tag, index = inout(reg) index);
    }
}

pub unsafe fn memset(mem: &mut [u8]) {
    assert_eq!(mem.len() % 32, 0);

    mem.fill(0);
}
