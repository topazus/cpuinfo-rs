use cpuinfo::utils::cpuid_count;
use cpuinfo::vendor::*;

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

    /// Query a set of features that are available on this CPU (LEAF=0x01).

    pub fn get_codename(&self) -> Option<String> {
        let res = self.read.cpuid1(0x01);
        let codename = match res.eax & 0xf00 {
            0x000 => "Intel Core i3",
            0x100 => "Intel Core i5",
            0x200 => "Intel Core i7",
            0x300 => "Intel Core i9",
            0x400 => "Intel Core i3",
            0x500 => "Intel Core i5",
            0x600 => "Intel Core i7",
            0x700 => "Intel Core i9",
            0x800 => "Intel Core i3",
            0x900 => "Intel Core i5",
            0xA00 => "Intel Core i7",
            0xB00 => "Intel Core i9",
            0xC00 => "Intel Core i3",
            0xD00 => "Intel Core i5",
            0xE00 => "Intel Core i7",
            0xF00 => "Intel Core i9",
            _ => "Unknown",
        };
        Some(codename.to_string())
    }
}

fn main() {
    let cpuid = CpuId::new();
    let codename = cpuid.get_codename().unwrap();
    println!("Codename: {}", codename);
    let vi = VendorInfo {
        ebx: 0x756e6547,
        edx: 0x49656e69,
        ecx: 0x6c65746e,
    };
    println!("Vendor: {}", vi.as_str());
}
