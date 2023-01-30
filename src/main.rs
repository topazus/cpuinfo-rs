use cpuinfo::intel_hybrid_core_info::get_cpu_purpose;
use cpuinfo::parse_cpuid::CpuId;
use std::arch::x86_64::__cpuid_count;

fn main() {
    let a = get_cpu_purpose();
    println!("{:?}", a);
    let cpuid_data = unsafe { __cpuid_count(0x7, 0x0) };
    println!("{:x?}", cpuid_data);
    // get CPUID data in different CPU
    let cpuid_data = unsafe { __cpuid_count(0x7, 0x1) };
    println!("{:x?}", cpuid_data);
    let cpuid = CpuId::new();
    println!(
        "stepping id: {:#x}, model id: {:#x}, family id: {:#x}, processor type: {:#x}, extended model id: {}, extended family id: {:#x}",
        cpuid.get_stepping_id().unwrap(), cpuid.get_model().unwrap(), cpuid.get_family().unwrap(), cpuid.get_processor_type().unwrap(), cpuid.get_extended_model().unwrap(), cpuid.get_extended_family().unwrap())
}
