use std::arch::asm;

#[inline]
fn set_tag(addr: *mut u8, tag: u64) -> *mut u8 {
    let tag = tag & 0x0f00_0000_0000_0000;
    (((addr as u64) & 0x0000_ffff_ffff_ffff) | tag) as *mut u8
}

pub unsafe fn stg(mem: &mut [u8], tag: u64) {
    debug_assert_eq!(mem.len() % 32, 0);

    let mut index = mem.as_mut_ptr();
    let end = index.add(mem.len());

    while index != end {
        asm!("stg {tag}, [{index}], #16", tag = in(reg) tag, index = inout(reg) index);
    }
}

pub unsafe fn stg_prefetch(mem: &mut [u8], tag: u64) {
    debug_assert_eq!(mem.len() % 32, 0);

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
    debug_assert_eq!(mem.len() % 32, 0);

    let mut index = mem.as_mut_ptr();
    let end = index.add(mem.len());

    while index != end {
        asm!("stg {tag}, [{index}], #16", tag = in(reg) tag, index = inout(reg) index);
    }

    let ptr = set_tag(mem.as_mut_ptr(), tag);
    std::slice::from_raw_parts_mut(ptr, mem.len()).fill(0);
}

pub unsafe fn stgp(mem: &mut [u8], tag: u64) {
    debug_assert_eq!(mem.len() % 32, 0);

    let mut index = set_tag(mem.as_mut_ptr(), tag);
    let end = index.add(mem.len());

    let zero = 0u64;

    while index != end {
        asm!("stgp {zero}, {zero}, [{index}], #16", zero = in(reg) zero, index = inout(reg) index);
    }
}

pub unsafe fn st2g(mem: &mut [u8], tag: u64) {
    debug_assert_eq!(mem.len() % 32, 0);

    let mut index = mem.as_mut_ptr();
    let end = index.add(mem.len());

    while index != end {
        asm!("st2g {tag}, [{index}], #32", tag = in(reg) tag, index = inout(reg) index);
    }
}

pub unsafe fn st2g_zero(mem: &mut [u8], tag: u64) {
    debug_assert_eq!(mem.len() % 32, 0);

    let mut index = mem.as_mut_ptr();
    let end = index.add(mem.len());

    while index != end {
        asm!("st2g {tag}, [{index}], #32", tag = in(reg) tag, index = inout(reg) index);
    }

    let ptr = set_tag(mem.as_mut_ptr(), tag);
    std::slice::from_raw_parts_mut(ptr, mem.len()).fill(0);
}

pub unsafe fn stzg(mem: &mut [u8], tag: u64) {
    debug_assert_eq!(mem.len() % 32, 0);

    let mut index = mem.as_mut_ptr();
    let end = index.add(mem.len());

    while index != end {
        asm!("stzg {tag}, [{index}], #16", tag = in(reg) tag, index = inout(reg) index);
    }
}

pub unsafe fn stz2g(mem: &mut [u8], tag: u64) {
    debug_assert_eq!(mem.len() % 32, 0);

    let mut index = mem.as_mut_ptr();
    let end = index.add(mem.len());

    while index != end {
        asm!("stz2g {tag}, [{index}], #32", tag = in(reg) tag, index = inout(reg) index);
    }
}

pub unsafe fn memset(mem: &mut [u8]) {
    debug_assert_eq!(mem.len() % 32, 0);

    mem.fill(0);
}

pub unsafe fn set_tags_random(mem: &mut [u8]) {
    debug_assert_eq!(mem.len() % 16, 0);

    let mut index = mem.as_mut_ptr();
    let end = index.add(mem.len());
    let mut tag = 0u64;

    while index != end {
        asm!(
            "stg {tag}, [{index}], #16",
            "addg {tag}, {tag}, #0, #1",
            tag = inout(reg) tag,
            index = inout(reg) index,
        );
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub unsafe fn migrate_mte_off(from: &[u8], to: &mut [u8]) {
    debug_assert_eq!(from.len() % 16, 0);
    debug_assert!(to.len() >= from.len());

    set_mte_mode(MTEMode::None);

    to.copy_from_slice(from);

    let mut index = from.as_ptr();
    let mut index_to = to.as_mut_ptr();
    let end = index.add(from.len());

    while index != end {
        asm!(
            "ldg {tag}, [{index}]",
            "stg {tag}, [{index_to}], #16",
            tag = out(reg) _,
            index = in(reg) index,
            index_to = inout(reg) index_to,
        );
        index = index.add(16);
    }

    set_mte_mode(MTEMode::Sync);
}

pub unsafe fn migrate_tags(from: &[u8], to: &mut [u8]) {
    debug_assert!(to.len() >= from.len());

    let mut index = from.as_ptr();
    let mut index_to = to.as_mut_ptr();
    let end = index.add(from.len());

    while index != end {
        asm!(
            "ldg {index}, [{index}]",
            "ldg {index_to}, [{index}]",
            "ldp {val1}, {val2}, [{index}]",
            "stgp {val1}, {val2}, [{index_to}], #16",
            "add {index}, {index}, #16",
            index = inout(reg) index,
            index_to = inout(reg) index_to,
            val1 = out(reg) _,
            val2 = out(reg) _,
        );
    }
}

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

#[cfg(any(target_os = "linux", target_os = "android"))]
pub unsafe fn set_mte_mode(mode: MTEMode) {
    const PR_SET_TAGGED_ADDR_CTRL: i32 = 55;
    const PR_TAGGED_ADDR_ENABLE: u64 = 1 << 0;
    const PR_MTE_TAG_SHIFT: i32 = 3;

    assert_eq!(
        libc::prctl(
            PR_SET_TAGGED_ADDR_CTRL,
            PR_TAGGED_ADDR_ENABLE | mode.mask() | (0xffff << PR_MTE_TAG_SHIFT), // no excluded tags for irg
            0,
            0,
            0,
        ),
        0,
        "could not enable mte"
    );
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub unsafe fn set_mte_mode_tags(mode: MTEMode, included_tags: u64) {
    const PR_SET_TAGGED_ADDR_CTRL: i32 = 55;
    const PR_TAGGED_ADDR_ENABLE: u64 = 1 << 0;
    const PR_MTE_TAG_SHIFT: i32 = 3;

    assert_eq!(
        libc::prctl(
            PR_SET_TAGGED_ADDR_CTRL,
            PR_TAGGED_ADDR_ENABLE | mode.mask() | (included_tags << PR_MTE_TAG_SHIFT),
            0,
            0,
            0,
        ),
        0,
        "could not enable mte"
    );
}
