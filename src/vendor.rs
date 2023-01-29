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
    pub fn as_str(&self) -> &str {
        let brand_string_start = self as *const VendorInfo as *const u8;
        let slice = unsafe {
            // Safety: VendorInfo is laid out with repr(C) and exactly
            // 12 byte long without any padding.
            std::slice::from_raw_parts(brand_string_start, std::mem::size_of::<VendorInfo>())
        };

        std::str::from_utf8(slice).unwrap_or("InvalidVendorString")
    }
    pub fn as_string(&self) -> String {
        self.as_str().to_string()
    }
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

        match vi.as_str() {
            "GenuineIntel" => Vendor::Intel,
            "AuthenticAMD" => Vendor::Amd,
            _ => Vendor::Unknown(res.ebx, res.ecx, res.edx),
        }
    }
}
