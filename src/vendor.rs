use crate::utils::num_to_string;

use std::arch::x86_64::CpuidResult;

#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[repr(C)]
pub struct VendorInfo {
    pub ebx: u32,
    pub edx: u32,
    pub ecx: u32,
}

impl VendorInfo {
    /// Return vendor identification as human readable string.
    pub fn as_string(&self) -> String {
        format!(
            "{}{}{}",
            num_to_string(self.ebx),
            num_to_string(self.edx),
            num_to_string(self.ecx)
        )
    }
    pub fn as_str2(&self) -> &str {
        let brand_string_start = self as *const VendorInfo as *const u8;
        let slice = unsafe {
            // Safety: VendorInfo is laid out with repr(C) and exactly
            // 12 byte long without any padding.
            std::slice::from_raw_parts(brand_string_start, std::mem::size_of::<VendorInfo>())
        };

        std::str::from_utf8(slice).unwrap_or("InvalidVendorString")
    }

    /// AMD vendor strings: "AuthenticAMD"
    const AMD: Self = Self {
        ebx: 0x68747541,
        edx: 0x69746E65,
        ecx: 0x444D4163,
    };
    /// Intel vendor string: "GenuineIntel"
    const INTEL: Self = Self {
        ebx: 0x756E6547,
        edx: 0x49656E69,
        ecx: 0x6C65746E,
    };
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum Vendor {
    Intel,
    Amd,
    Unknown(u32, u32, u32),
}

impl Vendor {
    pub fn from_vendor_leaf(res: CpuidResult) -> Self {
        let vi = VendorInfo {
            ebx: res.ebx,
            ecx: res.ecx,
            edx: res.edx,
        };
        match vi {
            VendorInfo::AMD => Vendor::Amd,
            VendorInfo::INTEL => Vendor::Intel,
            _ => Vendor::Unknown(res.ebx, res.ecx, res.edx),
        }
    }
    pub fn from_vendor_leaf_by_str(res: CpuidResult) -> Self {
        let vi = VendorInfo {
            ebx: res.ebx,
            ecx: res.ecx,
            edx: res.edx,
        };

        match vi.as_string().as_str() {
            "GenuineIntel" => Vendor::Intel,
            "AuthenticAMD" => Vendor::Amd,
            _ => Vendor::Unknown(res.ebx, res.ecx, res.edx),
        }
    }
}
