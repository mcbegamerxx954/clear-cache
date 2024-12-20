#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) unsafe fn os_arch_clear_cache<T>(_start: *const T, _end: *const T) -> bool {
    // Intel processors have a unified instruction and data cache so there is nothing to do
    true
}

#[cfg(target_arch = "loongarch64")]
pub(crate) unsafe fn os_arch_clear_cache<T>(start: *const T, end: *const T) -> bool {
    unsafe {
        core::arch::asm!("ibar 0");
    }
    true
}

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn os_arch_clear_cache<T>(start: *const T, end: *const T) -> bool {
    let start = start as u64;
    let end = end as u64;

    let mut ctr_el0: u64;
    // Get Cache Type Info.
    unsafe {
        core::arch::asm!("mrs {0}, ctr_el0", out(reg) ctr_el0);
    }

    // If CTR_EL0.IDC is set, data cache cleaning to the point of unification
    // is not required for instruction to data coherence.
    if ((ctr_el0 >> 28) & 0x1) == 0x0 {
        let dcache_line_size = 4 << ((ctr_el0 >> 16) & 15);
        let mut addr = start & !(dcache_line_size - 1);
        while addr < end {
            unsafe {
                core::arch::asm!(
                    "dc cvau, {0}",
                    in(reg) addr,
                );
            }
            addr += dcache_line_size;
        }
    }
    unsafe {
        core::arch::asm!("dsb ish");
    }
    // If CTR_EL0.DIC is set, instruction cache invalidation to the point of
    // unification is not required for instruction to data coherence.
    if ((ctr_el0 >> 29) & 0x1) == 0x0 {
        let icache_line_size = 4 << ((ctr_el0 >> 0) & 15);
        let mut addr = start & !(icache_line_size - 1);
        while addr < end {
            unsafe {
                core::arch::asm!(
                    "ic ivau, {0}",
                    in(reg) addr,
                );
            }
            addr += icache_line_size;
        }
        unsafe {
            core::arch::asm!("dsb ish");
        }
    }
    unsafe {
        core::arch::asm!("isb sy");
    }

    true
}
#[cfg(target_arch = "arm")]
pub(crate) unsafe fn os_arch_clear_cache<T>(start: *const T, end: *const T) -> bool {
    const __ARM_NR_CACHEFLUSH: i32 = 0x0f0002;
    let res = libc::syscall(
        __ARM_NR_CACHEFLUSH,
        start as usize as u64,
        end as usize as u64,
        0,
    );
    res == 0
}

#[cfg(target_arch = "riscv64")]
pub(crate) unsafe fn os_arch_clear_cache<T>(start: *const T, end: *const T) -> bool {
    const __NR_RISCV_FLUSH_ICACHE: i64 = 259;
    let res = unsafe {
        libc::syscall(
            __NR_RISCV_FLUSH_ICACHE,
            start as usize as u64,
            end as usize as u64,
            // "0" means that we clear cache for all threads (SYS_RISCV_FLUSH_ICACHE_ALL)
            0,
        )
    };
    res == 0
}
