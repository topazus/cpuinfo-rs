#[cfg(all(target_arch = "x86", not(target_env = "sgx"), target_feature = "sse"))]
use core::arch::x86 as arch;
#[cfg(all(target_arch = "x86_64", not(target_env = "sgx")))]
use core::arch::x86_64 as arch;
use std::arch::x86_64::CpuidResult;

pub fn cpuid_count(a: u32, c: u32) -> CpuidResult {
    // Safety: CPUID is supported on all x86_64 CPUs and all x86 CPUs with
    // SSE, but not by SGX.
    let result = unsafe { self::arch::__cpuid_count(a, c) };

    CpuidResult {
        eax: result.eax,
        ebx: result.ebx,
        ecx: result.ecx,
        edx: result.edx,
    }
}

/// convert number to array of u8 and parse it to string
pub fn num_to_string(x: u32) -> String {
    let b = x.to_le_bytes();
    let res = std::str::from_utf8(&b).unwrap();
    res.to_string()
}

// in C,
// #define EXTRACTS_BIT(reg, bit)              ((reg >> bit)    & 0x1)
pub fn extract_bit(reg: u32, bit: u32) -> u32 {
    (reg >> bit) & 0x1
}

// in C,
// #define EXTRACTS_BITS(reg, highbit, lowbit) ((reg >> lowbit) & ((1ULL << (highbit - lowbit + 1)) - 1))
pub fn extract_bits(reg: u32, highbit: u32, lowbit: u32) -> u32 {
    (reg >> lowbit) & ((1 << (highbit - lowbit + 1)) - 1)
}
