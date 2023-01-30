use cpuinfo::intel_hybrid_core_info::{self, get_cpu_purpose, HybridInfo, IntelNativeModelId};
use cpuinfo::parse_cpuid::CpuId;
use cpuinfo::utils::cpuid_count;
use std::arch::x86_64::__cpuid_count;

fn main() {
    let cpuid = CpuId::new();
    // use HybridInfo
    let hybrid = IntelNativeModelId::from(&cpuid_count(0x7, 0x0));
    println!("{:?}", hybrid);
    let core_type = HybridInfo::get_core_type2(&cpuid_count(0x1A, 0x0)).unwrap();
    println!("{:?}", core_type);
    let core_type = HybridInfo::get_core_type(&cpuid_count(0x1A, 0x0)).unwrap();
    println!("{:?}", core_type);

    println!("{:?}", intel_hybrid_core_info::is_hetero_core_supported());

    let a = get_cpu_purpose();
    println!("{:?}", a);
    let cpuid_data = unsafe { __cpuid_count(0x7, 0x0) };
    println!("{:x?}", cpuid_data);
    // get CPUID data in different CPU
    let cpuid_data = unsafe { __cpuid_count(0x7, 0x1) };
    println!("{:x?}", cpuid_data);
    let cpuid = CpuId::new();
    println!("{}", cpuid.get_logical_cores().unwrap());
    println!(
        "stepping id: {:#x}, model id: {:#x}, family id: {:#x}, processor type: {:#x}, extended model id: {}, extended family id: {:#x}",
        cpuid.get_stepping_id().unwrap(), cpuid.get_model().unwrap(), cpuid.get_family().unwrap(), cpuid.get_processor_type().unwrap(), cpuid.get_extended_model().unwrap(), cpuid.get_extended_family().unwrap())
}
