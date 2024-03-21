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
