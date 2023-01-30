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

// Detect if Hetero Core is supported.
//   Check for CPU whether is Hetrogenous or not.
// Check Hetero feature is supported
// with CPUID.(EAX=7,ECX=0):EDX[15]=1
pub fn is_hetero_core_supported() -> bool {
    let cpuid = cpuid_count(0x7, 0x0);
    extract_bit(cpuid.edx, 15) == 0x1
}

/* https://github.com/slimbootloader/slimbootloader/blob/master/Platform/AlderlakeBoardPkg/Library/Stage2BoardInitLib/CpuInfoLib.c */

// CPUID Native Model ID Information
//   @param   EAX  CPUID_NATIVE_MODEL_ID_INFO (0x1A)
//   @retval  EAX  Value of bits [23:0] gives Native Model ID
//                 Value of bits [31:24] gives Core Type. 0x40 - Core, 0x20 - Atom
//   @retval  EBX  Reserved.
//   @retval  ECX  Reserved.
//   @retval  EDX  Reserved.
// Detect the type of core.

//   get the core type which is running
//   10h - Quark
//   20h - Atom
//   30H - Knights
//   40H - Core
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
#[repr(u8)]
pub enum HybridCoreType {
    Quark = 0x10, // Quark?
    Atom = 0x20,
    Knights = 0x30, // Knights?
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
            0x10 => HybridCoreType::Quark,
            0x20 => HybridCoreType::Atom,
            0x30 => HybridCoreType::Knights,
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

    // Check which is the running core by reading CPUID.(EAX=1AH, ECX=00H):EAX[31:24]
    //  10h - Quark
    //  20h - Atom
    //  30h - Knights
    //  40h - Core
    pub fn get_core_type(cpuid: &CpuidResult) -> Option<HybridCoreType> {
        let core_type = HybridCoreType::from(cpuid);

        if core_type == HybridCoreType::Invalid {
            return None;
        }
        Some(core_type)
    }

    // Check which is the running core by reading CPUID.(EAX=1AH, ECX=00H):EAX[31:24]
    pub fn get_core_type2(cpuid: &CpuidResult) -> Option<HybridCoreType> {
        let cpuid_data = extract_bits(cpuid.eax, 31, 24);
        match cpuid_data {
            0x10 => Some(HybridCoreType::Quark),
            0x20 => Some(HybridCoreType::Atom),
            0x30 => Some(HybridCoreType::Knights),
            0x40 => Some(HybridCoreType::Core),
            _ => None,
        }
    }

    pub fn get_native_model_id(cpuid: &CpuidResult) -> u32 {
        cpuid.eax & 0x00FFFFFF
    }
}

#[derive(Debug)]
#[repr(u32)]
pub enum IntelNativeModelId {
    /* Atom */
    Tremont = Self::TNT,
    /// Gracemont microarchitecture is used in the efficiency cores (E-core)
    /// of the 12th-generation Intel Core processors (codenamed "Alder Lake")
    Gracemont = Self::GRT,
    Crestmont = Self::CMT,
    /* Core */
    SunnyCove = Self::SNC,
    /// GoldenCove microarchitecture is used in the high-performance cores (P-core)
    /// of the 12th-generation Intel Core processors (codenamed "Alder Lake")
    GoldenCove = Self::GLC,
    RedwoodCove = Self::RWC,
    /* */
    Unknown(u32),
}

/* https://github.com/intel/perfmon/blob/main/mapfile.csv */
impl IntelNativeModelId {
    pub const fn gen_eax(core_type: HybridCoreType, nid: u32) -> u32 {
        ((core_type as u32) << 24) | (nid & 0x00FFFFFF)
    }

    const TNT: u32 = Self::gen_eax(HybridCoreType::Atom, 0x0);
    const GRT: u32 = Self::gen_eax(HybridCoreType::Atom, 0x1);
    const CMT: u32 = Self::gen_eax(HybridCoreType::Atom, 0x2);

    const SNC: u32 = Self::gen_eax(HybridCoreType::Core, 0x0);
    const GLC: u32 = Self::gen_eax(HybridCoreType::Core, 0x1);
    const RWC: u32 = Self::gen_eax(HybridCoreType::Core, 0x2);
}

impl From<&CpuidResult> for IntelNativeModelId {
    fn from(cpuid: &CpuidResult) -> Self {
        match cpuid.eax & 0x00FFFFFF {
            Self::TNT => Self::Tremont,
            Self::GRT => Self::Gracemont,
            Self::CMT => Self::Crestmont,
            Self::SNC => Self::SunnyCove,
            Self::GLC => Self::GoldenCove,
            Self::RWC => Self::RedwoodCove,
            _ => Self::Unknown(cpuid.eax),
        }
    }
}
