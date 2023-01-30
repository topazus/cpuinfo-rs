#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::CpuidResult;

#[macro_use]

pub mod intel_hybrid_core_info;
mod intel_ext_topo_0bh_1fh;
pub use intel_ext_topo_0bh_1fh::*;

pub mod parse_cpuid;
pub mod topo_info;
pub mod utils;
pub mod vendor;
