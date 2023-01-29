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
