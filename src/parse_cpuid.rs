use crate::utils::{cpuid_count, extract_bit, extract_bits, num_to_string};
use crate::vendor::*;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::CpuidResult;

//
// Normal leafs:
//
const EAX_VENDOR_INFO: u32 = 0x0;
const EAX_FEATURE_INFO: u32 = 0x1;
const EAX_CACHE_INFO: u32 = 0x2;
const EAX_PROCESSOR_SERIAL: u32 = 0x3;
const EAX_CACHE_PARAMETERS: u32 = 0x4;
const EAX_MONITOR_MWAIT_INFO: u32 = 0x5;
const EAX_THERMAL_POWER_INFO: u32 = 0x6;
const EAX_STRUCTURED_EXTENDED_FEATURE_INFO: u32 = 0x7;
const EAX_DIRECT_CACHE_ACCESS_INFO: u32 = 0x9;
const EAX_PERFORMANCE_MONITOR_INFO: u32 = 0xA;
const EAX_EXTENDED_TOPOLOGY_INFO: u32 = 0xB;
const EAX_EXTENDED_STATE_INFO: u32 = 0xD;
const EAX_RDT_MONITORING: u32 = 0xF;
const EAX_RDT_ALLOCATION: u32 = 0x10;
const EAX_SGX: u32 = 0x12;
const EAX_TRACE_INFO: u32 = 0x14;
const EAX_TIME_STAMP_COUNTER_INFO: u32 = 0x15;
const EAX_FREQUENCY_INFO: u32 = 0x16;
const EAX_SOC_VENDOR_INFO: u32 = 0x17;
const EAX_DETERMINISTIC_ADDRESS_TRANSLATION_INFO: u32 = 0x18;
const EAX_EXTENDED_TOPOLOGY_INFO_V2: u32 = 0x1F;

/// Hypervisor leaf
const EAX_HYPERVISOR_INFO: u32 = 0x4000_0000;

//
// Extended leafs:
//
const EAX_EXTENDED_FUNCTION_INFO: u32 = 0x8000_0000;
const EAX_EXTENDED_PROCESSOR_AND_FEATURE_IDENTIFIERS: u32 = 0x8000_0001;
const EAX_EXTENDED_BRAND_STRING: u32 = 0x8000_0002;
const EAX_L1_CACHE_INFO: u32 = 0x8000_0005;
const EAX_L2_L3_CACHE_INFO: u32 = 0x8000_0006;
const EAX_ADVANCED_POWER_MGMT_INFO: u32 = 0x8000_0007;
const EAX_PROCESSOR_CAPACITY_INFO: u32 = 0x8000_0008;
const EAX_TLB_1GB_PAGE_INFO: u32 = 0x8000_0019;
const EAX_PERFORMANCE_OPTIMIZATION_INFO: u32 = 0x8000_001A;
const EAX_CACHE_PARAMETERS_AMD: u32 = 0x8000_001D;
const EAX_PROCESSOR_TOPOLOGY_INFO: u32 = 0x8000_001E;
const EAX_MEMORY_ENCRYPTION_INFO: u32 = 0x8000_001F;
const EAX_SVM_FEATURES: u32 = 0x8000_000A;

/// Implements function to read/write cpuid.
/// This allows to conveniently swap out the underlying cpuid implementation
/// with one that returns data that is deterministic (for unit-testing).
#[derive(Debug, Clone, Copy)]
struct CpuIdReader {
    cpuid_fn: fn(u32, u32) -> CpuidResult,
}

impl CpuIdReader {
    fn new(cpuid_fn: fn(u32, u32) -> CpuidResult) -> Self {
        Self { cpuid_fn }
    }

    fn cpuid1(&self, eax: u32) -> CpuidResult {
        (self.cpuid_fn)(eax, 0)
    }

    fn cpuid2(&self, eax: u32, ecx: u32) -> CpuidResult {
        (self.cpuid_fn)(eax, ecx)
    }
}

#[cfg(any(
    all(target_arch = "x86", not(target_env = "sgx"), target_feature = "sse"),
    all(target_arch = "x86_64", not(target_env = "sgx"))
))]
impl Default for CpuIdReader {
    fn default() -> Self {
        Self {
            cpuid_fn: cpuid_count,
        }
    }
}

/// The main type used to query information about the CPU we're running on.
///
/// Other structs can be accessed by going through this type.
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Clone, Copy)]
pub struct CpuId {
    #[cfg_attr(feature = "serialize", serde(skip))]
    read: CpuIdReader,
    /// CPU vendor to differntiate cases where logic needs to differ in code .
    vendor: Vendor,
    /// How many basic leafs are supported (EAX < EAX_HYPERVISOR_INFO)
    supported_leafs: u32,
    /// How many extended leafs are supported (e.g., leafs with EAX > EAX_EXTENDED_FUNCTION_INFO)
    supported_extended_leafs: u32,
}

#[cfg(any(
    all(target_arch = "x86", not(target_env = "sgx"), target_feature = "sse"),
    all(target_arch = "x86_64", not(target_env = "sgx"))
))]
impl Default for CpuId {
    fn default() -> CpuId {
        CpuId::with_cpuid_fn(cpuid_count)
    }
}
impl CpuId {
    /// Return new CpuId struct.
    #[cfg(any(
        all(target_arch = "x86", not(target_env = "sgx"), target_feature = "sse"),
        all(target_arch = "x86_64", not(target_env = "sgx"))
    ))]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return new CpuId struct with custom reader function.
    ///
    /// This is useful for example when testing code or if we want to interpose
    /// on the CPUID calls this library makes.
    pub fn with_cpuid_fn(cpuid_fn: fn(u32, u32) -> CpuidResult) -> Self {
        let read = CpuIdReader::new(cpuid_fn);
        let vendor_leaf = read.cpuid1(EAX_VENDOR_INFO);
        let extended_leaf = read.cpuid1(EAX_EXTENDED_FUNCTION_INFO);
        CpuId {
            supported_leafs: vendor_leaf.eax,
            supported_extended_leafs: extended_leaf.eax,
            vendor: Vendor::from_vendor_leaf(vendor_leaf),
            read,
        }
    }

    /// Check if a non extended leaf  (`val`) is supported.
    fn leaf_is_supported(&self, val: u32) -> bool {
        // Exclude reserved functions/leafs on AMD
        if self.vendor == Vendor::Amd && ((0x2..=0x4).contains(&val) || (0x8..=0xa).contains(&val))
        {
            return false;
        }

        if val < EAX_EXTENDED_FUNCTION_INFO {
            val <= self.supported_leafs
        } else {
            val <= self.supported_extended_leafs
        }
    }

    /// Return information about the vendor (LEAF=0x00).
    ///
    /// This leaf will contain a ASCII readable string such as "GenuineIntel"
    /// for Intel CPUs or "AuthenticAMD" for AMD CPUs.
    ///
    /// # Platforms
    /// ✅ AMD ✅ Intel
    pub fn get_vendor_info(&self) -> Option<VendorInfo> {
        if self.leaf_is_supported(EAX_VENDOR_INFO) {
            let res = self.read.cpuid1(EAX_VENDOR_INFO);
            Some(VendorInfo {
                ebx: res.ebx,
                ecx: res.ecx,
                edx: res.edx,
            })
        } else {
            None
        }
    }
    // EAX[3:0] := Stepping ID;
    // EAX[7:4] := Model;
    // EAX[11:8] := Family
    // EAX[13:12] := Processor type;
    // EAX[15:14] := Reserved;
    // EAX[19:16] := Extended Model;
    // EAX[27:20] := Extended Family;
    // EAX[31:28] := Reserved;
    // EBX[7:0] := Brand Index; (* Reserved if the value is zero. *)
    // EBX[15:8] := CLFLUSH Line Size;
    // EBX[16:23] := Reserved; (* Number of threads enabled = 2 if MT enable fuse set. *)
    // EBX[24:31] := Initial APIC ID;
    // ECX := Feature flags; (* See Figure 3-7. *)
    // EDX := Feature flags; (* See Figure 3-8. *)

    /// EAX[3:0] := Stepping ID;
    pub fn get_stepping_id(&self) -> Option<u32> {
        if self.leaf_is_supported(EAX_FEATURE_INFO) {
            let res = self.read.cpuid1(EAX_FEATURE_INFO);
            Some(extract_bits(res.eax, 3, 0))
        } else {
            None
        }
    }
    /// EAX[7:4] := Model;
    pub fn get_model(&self) -> Option<u32> {
        if self.leaf_is_supported(EAX_FEATURE_INFO) {
            let res = self.read.cpuid1(EAX_FEATURE_INFO);
            Some(extract_bits(res.eax, 7, 4))
        } else {
            None
        }
    }
    /// EAX[11:8] := Family
    pub fn get_family(&self) -> Option<u32> {
        if self.leaf_is_supported(EAX_FEATURE_INFO) {
            let res = self.read.cpuid1(EAX_FEATURE_INFO);
            Some(extract_bits(res.eax, 11, 8))
        } else {
            None
        }
    }
    /// EAX[13:12] := Processor type;
    pub fn get_processor_type(&self) -> Option<u32> {
        if self.leaf_is_supported(EAX_FEATURE_INFO) {
            let res = self.read.cpuid1(EAX_FEATURE_INFO);
            Some(extract_bits(res.eax, 13, 12))
        } else {
            None
        }
    }
    // EAX[19:16] := Extended Model;
    pub fn get_extended_model(&self) -> Option<u32> {
        if self.leaf_is_supported(EAX_FEATURE_INFO) {
            let res = self.read.cpuid1(EAX_FEATURE_INFO);
            Some(extract_bits(res.eax, 19, 16))
        } else {
            None
        }
    }
    // EAX[27:20] := Extended Family;
    pub fn get_extended_family(&self) -> Option<u32> {
        if self.leaf_is_supported(EAX_FEATURE_INFO) {
            let res = self.read.cpuid1(EAX_FEATURE_INFO);
            Some(extract_bits(res.eax, 27, 20))
        } else {
            None
        }
    }
    /// EBX[16:23] := Reserved; (* Number of threads enabled = 2 if MT enable fuse set. *)
    pub fn get_logical_cores(&self) -> Option<u32> {
        if self.leaf_is_supported(EAX_FEATURE_INFO) {
            let res = self.read.cpuid1(EAX_FEATURE_INFO);
            Some(extract_bits(res.ebx, 23, 16))
        } else {
            None
        }
    }
}

// get CPUID dump raw data for every cpu
pub fn cpuid_get_all_raw_data() {}
