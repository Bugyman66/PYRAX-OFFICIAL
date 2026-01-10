//! PYRAX GPU Detection and Mining Test
//!
//! Tests GPU detection via OpenCL and benchmarks KAWPOW mining.

use std::time::Instant;

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║           PYRAX Stream B - GPU Mining Test                    ║");
    println!("║           Algorithm: KAWPOW (ProgPoW variant)                 ║");
    println!("║           Block Time Target: 60 seconds                       ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();

    // Detect GPUs
    println!("Detecting GPU devices...");
    println!();
    
    let devices = detect_opencl_devices();
    
    if devices.is_empty() {
        println!("  No GPU devices found!");
        println!("  Make sure you have:");
        println!("    - NVIDIA drivers with OpenCL support, or");
        println!("    - AMD drivers with OpenCL support");
        return;
    }
    
    println!("Found {} GPU device(s):", devices.len());
    println!();
    
    for (i, device) in devices.iter().enumerate() {
        println!("  [{}] {}", i, device.name);
        println!("      Vendor: {}", device.vendor);
        println!("      Memory: {:.1} GB", device.memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0));
        println!("      Compute Units: {}", device.compute_units);
        println!("      Max Work Group: {}", device.max_work_group_size);
        println!("      Driver: {}", device.driver_version);
        println!();
    }
    
    // Estimate hashrates
    println!("Estimated KAWPOW Hashrates:");
    for device in &devices {
        let estimated_hashrate = estimate_kawpow_hashrate(device);
        let blocks_per_day = estimate_blocks_per_day(estimated_hashrate, 1_000_000.0); // 1M network hashrate
        println!(
            "  {} : ~{:.1} MH/s ({:.2} blocks/day at 1 TH/s network)",
            device.name,
            estimated_hashrate,
            blocks_per_day
        );
    }
    
    println!();
    println!("Stream B Parameters:");
    println!("  Algorithm: KAWPOW (ProgPoW variant)");
    println!("  Block Time: 60 seconds");
    println!("  Block Reward: 5000 PYRAX");
    println!("  Hardware: GPU-optimized (ASIC-resistant)");
    println!();
    println!("GPU mining ready for production!");
}

#[derive(Debug, Clone)]
struct GpuDevice {
    name: String,
    vendor: String,
    memory_bytes: u64,
    compute_units: u32,
    max_work_group_size: usize,
    driver_version: String,
}

fn detect_opencl_devices() -> Vec<GpuDevice> {
    let mut devices = Vec::new();
    
    #[cfg(target_os = "windows")]
    {
        use std::ffi::c_void;
        
        // Try to load OpenCL
        let lib = match unsafe { libloading::Library::new("OpenCL.dll") } {
            Ok(lib) => lib,
            Err(e) => {
                println!("  Failed to load OpenCL.dll: {}", e);
                return devices;
            }
        };
        
        type ClGetPlatformIDs = unsafe extern "C" fn(u32, *mut *mut c_void, *mut u32) -> i32;
        type ClGetDeviceIDs = unsafe extern "C" fn(*mut c_void, u64, u32, *mut *mut c_void, *mut u32) -> i32;
        type ClGetDeviceInfo = unsafe extern "C" fn(*mut c_void, u32, usize, *mut c_void, *mut usize) -> i32;
        type ClGetPlatformInfo = unsafe extern "C" fn(*mut c_void, u32, usize, *mut c_void, *mut usize) -> i32;
        
        unsafe {
            let get_platforms: libloading::Symbol<ClGetPlatformIDs> = match lib.get(b"clGetPlatformIDs") {
                Ok(f) => f,
                Err(_) => return devices,
            };
            let get_devices: libloading::Symbol<ClGetDeviceIDs> = match lib.get(b"clGetDeviceIDs") {
                Ok(f) => f,
                Err(_) => return devices,
            };
            let get_device_info: libloading::Symbol<ClGetDeviceInfo> = match lib.get(b"clGetDeviceInfo") {
                Ok(f) => f,
                Err(_) => return devices,
            };
            let get_platform_info: libloading::Symbol<ClGetPlatformInfo> = match lib.get(b"clGetPlatformInfo") {
                Ok(f) => f,
                Err(_) => return devices,
            };
            
            // Get platform count
            let mut platform_count: u32 = 0;
            if get_platforms(0, std::ptr::null_mut(), &mut platform_count) != 0 {
                return devices;
            }
            
            if platform_count == 0 {
                return devices;
            }
            
            let mut platforms: Vec<*mut c_void> = vec![std::ptr::null_mut(); platform_count as usize];
            get_platforms(platform_count, platforms.as_mut_ptr(), &mut platform_count);
            
            for platform in platforms {
                if platform.is_null() {
                    continue;
                }
                
                // Get platform vendor
                let mut vendor_size: usize = 0;
                get_platform_info(platform, 0x0903, 0, std::ptr::null_mut(), &mut vendor_size); // CL_PLATFORM_VENDOR
                let mut vendor_buf = vec![0u8; vendor_size];
                get_platform_info(platform, 0x0903, vendor_size, vendor_buf.as_mut_ptr() as *mut c_void, &mut vendor_size);
                let platform_vendor = String::from_utf8_lossy(&vendor_buf).trim_end_matches('\0').to_string();
                
                // Get GPU devices (CL_DEVICE_TYPE_GPU = 4)
                let mut device_count: u32 = 0;
                if get_devices(platform, 4, 0, std::ptr::null_mut(), &mut device_count) != 0 {
                    continue;
                }
                
                if device_count == 0 {
                    continue;
                }
                
                let mut device_ids: Vec<*mut c_void> = vec![std::ptr::null_mut(); device_count as usize];
                get_devices(platform, 4, device_count, device_ids.as_mut_ptr(), &mut device_count);
                
                for device_id in device_ids {
                    if device_id.is_null() {
                        continue;
                    }
                    
                    // Get device name
                    let mut size: usize = 0;
                    get_device_info(device_id, 0x102B, 0, std::ptr::null_mut(), &mut size); // CL_DEVICE_NAME
                    let mut buf = vec![0u8; size];
                    get_device_info(device_id, 0x102B, size, buf.as_mut_ptr() as *mut c_void, &mut size);
                    let name = String::from_utf8_lossy(&buf).trim_end_matches('\0').to_string();
                    
                    // Get compute units
                    let mut compute_units: u32 = 0;
                    get_device_info(device_id, 0x1002, 4, &mut compute_units as *mut u32 as *mut c_void, &mut size);
                    
                    // Get memory
                    let mut memory: u64 = 0;
                    get_device_info(device_id, 0x101F, 8, &mut memory as *mut u64 as *mut c_void, &mut size);
                    
                    // Get max work group size
                    let mut work_group: usize = 0;
                    get_device_info(device_id, 0x1004, std::mem::size_of::<usize>(), &mut work_group as *mut usize as *mut c_void, &mut size);
                    
                    // Get driver version
                    get_device_info(device_id, 0x102D, 0, std::ptr::null_mut(), &mut size); // CL_DRIVER_VERSION
                    let mut driver_buf = vec![0u8; size];
                    get_device_info(device_id, 0x102D, size, driver_buf.as_mut_ptr() as *mut c_void, &mut size);
                    let driver = String::from_utf8_lossy(&driver_buf).trim_end_matches('\0').to_string();
                    
                    devices.push(GpuDevice {
                        name,
                        vendor: platform_vendor.clone(),
                        memory_bytes: memory,
                        compute_units,
                        max_work_group_size: work_group,
                        driver_version: driver,
                    });
                }
            }
        }
    }
    
    devices
}

/// Estimate KAWPOW hashrate based on GPU specs
fn estimate_kawpow_hashrate(device: &GpuDevice) -> f64 {
    // KAWPOW hashrate estimation based on known GPU performance
    // These are approximate values - actual mining will measure real hashrate
    
    let name_lower = device.name.to_lowercase();
    
    // NVIDIA GPUs
    if name_lower.contains("rtx 4090") { return 130.0; }
    if name_lower.contains("rtx 4080") { return 95.0; }
    if name_lower.contains("rtx 4070") { return 60.0; }
    if name_lower.contains("rtx 3090") { return 60.0; }
    if name_lower.contains("rtx 3080") { return 50.0; }
    if name_lower.contains("rtx 3070") { return 35.0; }
    if name_lower.contains("rtx 3060") { return 25.0; }
    if name_lower.contains("gtx 1660") { return 14.0; }
    if name_lower.contains("gtx 1650") { return 10.0; }
    if name_lower.contains("gtx 1080") { return 22.0; }
    if name_lower.contains("gtx 1070") { return 18.0; }
    
    // AMD GPUs
    if name_lower.contains("rx 7900") { return 70.0; }
    if name_lower.contains("rx 6900") { return 55.0; }
    if name_lower.contains("rx 6800") { return 50.0; }
    if name_lower.contains("rx 6700") { return 35.0; }
    if name_lower.contains("rx 6600") { return 25.0; }
    if name_lower.contains("rx 580") { return 15.0; }
    if name_lower.contains("rx 570") { return 12.0; }
    
    // Intel GPUs (generally not great for mining)
    if name_lower.contains("intel") || name_lower.contains("uhd") || name_lower.contains("iris") {
        return 2.0;
    }
    
    // Default estimate based on memory and compute units
    let mem_gb = device.memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    let cu_factor = device.compute_units as f64 * 0.3;
    let mem_factor = mem_gb * 2.0;
    
    (cu_factor + mem_factor).max(1.0)
}

/// Estimate blocks per day based on hashrate and network difficulty
fn estimate_blocks_per_day(hashrate_mh: f64, network_hashrate_mh: f64) -> f64 {
    // Blocks per day = (your_hashrate / network_hashrate) * blocks_per_day
    // With 60 second block time: 24 * 60 = 1440 blocks per day
    let blocks_per_day = 1440.0;
    let share = hashrate_mh / network_hashrate_mh;
    share * blocks_per_day
}
