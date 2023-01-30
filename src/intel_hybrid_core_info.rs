use crate::utils::{cpuid_count, extract_bit, extract_bits};

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::CpuidResult;

#[derive(Debug)]
pub enum CpuPurpose {
    /// general purpose CPU core
    GeneralPurpose,
    /// performance CPU core
    PerformancePurpose,
    /// efficiency CPU core
    EfficiencyPurpose,
}

/// Check for hybrid architecture
/// 	From Intel® 64 and IA-32 Architectures Software Developer’s Manual Combined Volumes: 1, 2A, 2B, 2C, 2D, 3A, 3B, 3C, 3D, and 4
/// 	Available at https://cdrdv2.intel.com/v1/dl/getContent/671200

/// 	- CPUID[7h] is Structured Extended Feature Flags Enumeration Leaf (Output depends on ECX input value)
/// 	  EDX, bit 15: Hybrid. If 1, the processor is identified as a hybrid part.

/// 	- CPUID[1Ah] is Hybrid Information Enumeration Leaf (EAX = 1AH, ECX = 0)
/// 	  EAX, bits 31-24: Core type
pub fn get_cpu_purpose() -> CpuPurpose {
    let cpuid = cpuid_count(0x7, 0x0);

    if extract_bit(cpuid.edx, 15) == 0x1 {
        let cpuid = cpuid_count(0x1A, 0x0);

        match extract_bits(cpuid.eax, 31, 24) {
            // Atom
            0x20 => CpuPurpose::EfficiencyPurpose,
            // Core
            0x40 => CpuPurpose::PerformancePurpose,
            _ => CpuPurpose::GeneralPurpose,
        }
    } else {
        CpuPurpose::GeneralPurpose
    }
}


/* https://github.com/slimbootloader/slimbootloader/blob/master/Platform/AlderlakeBoardPkg/Library/Stage2BoardInitLib/CpuInfoLib.c */

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
#[repr(u8)]
pub enum HybridCoreType {
    _Reserved1 = 0x10, // Quark?
    Atom = 0x20,
    _Reserved2 = 0x30, // Knights?
    Core = 0x40,
    Invalid,
}

impl std::fmt::Display for HybridCoreType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<&CpuidResult> for HybridCoreType {
    fn from(cpuid: &CpuidResult) -> Self {
        match cpuid.eax >> 24 {
            0x10 => HybridCoreType::_Reserved1,
            0x20 => HybridCoreType::Atom,
            0x30 => HybridCoreType::_Reserved2,
            0x40 => HybridCoreType::Core,
            _ => Self::Invalid,
        }
    }
}

pub struct HybridInfo;

impl HybridInfo {
    pub fn get_hybrid_info_from_cpuid(cpuid: &CpuidResult) -> (Option<HybridCoreType>, u32) {
        (Self::get_core_type(cpuid), Self::get_native_model_id(cpuid))
    }

    pub fn get_hybrid_info() -> (Option<HybridCoreType>, u32) {
        let cpuid = cpuid_count(0x1A, 0x0);

        Self::get_hybrid_info_from_cpuid(&cpuid)
    }

    pub fn get_core_type(cpuid: &CpuidResult) -> Option<HybridCoreType> {
        let core_type = HybridCoreType::from(cpuid);

        if core_type == HybridCoreType::Invalid {
            return None;
        }

        Some(core_type)
    }

    pub fn get_native_model_id(cpuid: &CpuidResult) -> u32 {
        cpuid.eax & 0x00FFFFFF
    }
}

/* https://github.com/intel/perfmon/blob/main/mapfile.csv */
impl IntelNativeModelId {
    const fn gen_eax(core_type: HybridCoreType, nid: u32) -> u32 {
        ((core_type as u32) << 24) | (nid & 0x00FFFFFF)
    }

    const TNT: u32 = Self::gen_eax(HybridCoreType::Atom, 0x0);
    const GRT: u32 = Self::gen_eax(HybridCoreType::Atom, 0x1);
    const CMT: u32 = Self::gen_eax(HybridCoreType::Atom, 0x2);

    const SNC: u32 = Self::gen_eax(HybridCoreType::Core, 0x0);
    const GLC: u32 = Self::gen_eax(HybridCoreType::Core, 0x1);
    const RWC: u32 = Self::gen_eax(HybridCoreType::Core, 0x2);
}

#[derive(Debug)]
#[repr(u32)]
enum IntelNativeModelId {
    /* Atom */
    Tremont = Self::TNT,
    Gracemont = Self::GRT,
    Crestmont = Self::CMT,
    /* Core */
    SunnyCove = Self::SNC,
    GoldenCove = Self::GLC,
    RedwoodCove = Self::RWC,
    /* */
    _Reserved,
}

impl From<u32> for IntelNativeModelId {
    fn from(eax: u32) -> Self {
        match eax {
            Self::TNT => Self::Tremont,
            Self::GRT => Self::Gracemont,
            Self::CMT => Self::Crestmont,
            Self::SNC => Self::SunnyCove,
            Self::GLC => Self::GoldenCove,
            Self::RWC => Self::RedwoodCove,
            _ => Self::_Reserved,
        }
    }
}
