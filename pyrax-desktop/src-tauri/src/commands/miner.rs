use crate::state::AppState;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::ffi::c_void;
use tauri::State;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerStatus {
    pub running: bool,
    pub hashrate: f64,
    pub hashrate_unit: String,
    pub accepted_shares: u64,
    pub rejected_shares: u64,
    pub blocks_found: u64,
    pub temperature: Option<u32>,
    pub fan_speed: Option<u32>,
    pub power_usage: Option<u32>,
    pub device_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MinerConfig {
    pub address: String,
    pub threads: Option<u32>,
    pub cuda_device: Option<i32>,
    pub opencl_device: Option<i32>,
    pub pool_url: Option<String>,
}

#[tauri::command]
pub async fn start_miner(
    config: MinerConfig,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<MinerStatus, String> {
    let mut app_state = state.lock();
    
    if app_state.miner_running {
        return Err("Miner is already running".to_string());
    }
    
    if !app_state.node_running && config.pool_url.is_none() {
        return Err("Node must be running for solo mining, or provide pool URL".to_string());
    }
    
    // Validate address
    if config.address.is_empty() {
        return Err("Miner address is required".to_string());
    }
    
    // TODO: Actually start miner process
    app_state.miner_running = true;
    app_state.settings.miner_address = Some(config.address);
    
    Ok(MinerStatus {
        running: true,
        hashrate: 0.0,
        hashrate_unit: "MH/s".to_string(),
        accepted_shares: 0,
        rejected_shares: 0,
        blocks_found: 0,
        temperature: None,
        fan_speed: None,
        power_usage: None,
        device_name: "Initializing...".to_string(),
    })
}

#[tauri::command]
pub async fn stop_miner(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let mut app_state = state.lock();
    
    if !app_state.miner_running {
        return Err("Miner is not running".to_string());
    }
    
    // TODO: Actually stop miner process
    app_state.miner_running = false;
    
    Ok(())
}

#[tauri::command]
pub async fn get_miner_status(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<MinerStatus, String> {
    let app_state = state.lock();
    
    if !app_state.miner_running {
        return Ok(MinerStatus {
            running: false,
            hashrate: 0.0,
            hashrate_unit: "MH/s".to_string(),
            accepted_shares: 0,
            rejected_shares: 0,
            blocks_found: 0,
            temperature: None,
            fan_speed: None,
            power_usage: None,
            device_name: "Not mining".to_string(),
        });
    }
    
    // TODO: Get real miner status
    Ok(MinerStatus {
        running: true,
        hashrate: 25.5,
        hashrate_unit: "MH/s".to_string(),
        accepted_shares: 100,
        rejected_shares: 2,
        blocks_found: 0,
        temperature: Some(65),
        fan_speed: Some(70),
        power_usage: Some(120),
        device_name: "GPU 0".to_string(),
    })
}

#[tauri::command]
pub async fn get_hashrate(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<f64, String> {
    let app_state = state.lock();
    
    if !app_state.miner_running {
        return Ok(0.0);
    }
    
    // TODO: Get real hashrate
    Ok(25.5)
}

/// GPU Device Information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuDevice {
    pub index: usize,
    pub name: String,
    pub vendor: String,
    pub memory_gb: f64,
    pub compute_units: u32,
    pub driver_version: String,
    pub estimated_hashrate: f64,
    pub estimated_blocks_per_day: f64,
    pub is_supported: bool,
}

/// Benchmark Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub device_index: usize,
    pub device_name: String,
    pub hashrate: f64,
    pub hashrate_unit: String,
    pub power_usage: Option<u32>,
    pub efficiency: f64,
    pub estimated_daily_blocks: f64,
    pub estimated_daily_earnings: f64,
}

/// Detect all available GPU devices
#[tauri::command]
pub async fn detect_gpus() -> Result<Vec<GpuDevice>, String> {
    info!("Detecting GPU devices...");
    let devices = detect_opencl_devices();
    
    if devices.is_empty() {
        warn!("No GPU devices detected");
    } else {
        info!("Detected {} GPU device(s)", devices.len());
    }
    
    Ok(devices)
}

/// Run GPU benchmark
#[tauri::command]
pub async fn benchmark_gpu(
    device_index: usize,
    duration_secs: u64,
) -> Result<BenchmarkResult, String> {
    let devices = detect_opencl_devices();
    
    let device = devices.iter()
        .find(|d| d.index == device_index)
        .ok_or_else(|| format!("Device {} not found", device_index))?;
    
    // For now, return estimated values
    // TODO: Implement actual KAWPOW benchmarking
    let hashrate = device.estimated_hashrate;
    let daily_blocks = device.estimated_blocks_per_day;
    
    Ok(BenchmarkResult {
        device_index,
        device_name: device.name.clone(),
        hashrate,
        hashrate_unit: "MH/s".to_string(),
        power_usage: estimate_power_usage(&device.name),
        efficiency: hashrate / estimate_power_usage(&device.name).unwrap_or(200) as f64,
        estimated_daily_blocks: daily_blocks,
        estimated_daily_earnings: daily_blocks * 5000.0, // 5000 PYRAX per block
    })
}

/// Estimate power usage based on GPU model
fn estimate_power_usage(name: &str) -> Option<u32> {
    let name_lower = name.to_lowercase();
    
    if name_lower.contains("rtx 4090") { return Some(450); }
    if name_lower.contains("rtx 4080") { return Some(320); }
    if name_lower.contains("rtx 4070") { return Some(200); }
    if name_lower.contains("rtx 3090") { return Some(350); }
    if name_lower.contains("rtx 3080") { return Some(320); }
    if name_lower.contains("rtx 3070") { return Some(220); }
    if name_lower.contains("rtx 3060") { return Some(170); }
    if name_lower.contains("gtx 1660") { return Some(120); }
    if name_lower.contains("gtx 1650") { return Some(75); }
    if name_lower.contains("rx 7900") { return Some(355); }
    if name_lower.contains("rx 6900") { return Some(300); }
    if name_lower.contains("rx 6800") { return Some(250); }
    if name_lower.contains("rx 6700") { return Some(230); }
    if name_lower.contains("rx 6600") { return Some(132); }
    
    Some(150) // Default
}

/// Estimate KAWPOW hashrate based on GPU model
fn estimate_kawpow_hashrate(name: &str, memory_gb: f64) -> f64 {
    let name_lower = name.to_lowercase();
    
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
    
    // Intel GPUs
    if name_lower.contains("intel") || name_lower.contains("uhd") || name_lower.contains("iris") {
        return 2.0;
    }
    
    // Default estimate based on memory
    memory_gb * 2.0
}

/// Detect OpenCL GPU devices
fn detect_opencl_devices() -> Vec<GpuDevice> {
    let mut devices = Vec::new();
    
    #[cfg(target_os = "windows")]
    {
        type ClGetPlatformIDs = unsafe extern "C" fn(u32, *mut *mut c_void, *mut u32) -> i32;
        type ClGetDeviceIDs = unsafe extern "C" fn(*mut c_void, u64, u32, *mut *mut c_void, *mut u32) -> i32;
        type ClGetDeviceInfo = unsafe extern "C" fn(*mut c_void, u32, usize, *mut c_void, *mut usize) -> i32;
        type ClGetPlatformInfo = unsafe extern "C" fn(*mut c_void, u32, usize, *mut c_void, *mut usize) -> i32;
        
        unsafe {
            let lib = match libloading::Library::new("OpenCL.dll") {
                Ok(lib) => lib,
                Err(_) => return devices,
            };
            
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
            
            let mut platform_count: u32 = 0;
            if get_platforms(0, std::ptr::null_mut(), &mut platform_count) != 0 || platform_count == 0 {
                return devices;
            }
            
            let mut platforms: Vec<*mut c_void> = vec![std::ptr::null_mut(); platform_count as usize];
            get_platforms(platform_count, platforms.as_mut_ptr(), &mut platform_count);
            
            let mut global_index = 0usize;
            
            for platform in platforms {
                if platform.is_null() { continue; }
                
                // Get platform vendor
                let mut vendor_size: usize = 0;
                get_platform_info(platform, 0x0903, 0, std::ptr::null_mut(), &mut vendor_size);
                let mut vendor_buf = vec![0u8; vendor_size];
                get_platform_info(platform, 0x0903, vendor_size, vendor_buf.as_mut_ptr() as *mut c_void, &mut vendor_size);
                let vendor = String::from_utf8_lossy(&vendor_buf).trim_end_matches('\0').to_string();
                
                // Get GPU devices
                let mut device_count: u32 = 0;
                if get_devices(platform, 4, 0, std::ptr::null_mut(), &mut device_count) != 0 || device_count == 0 {
                    continue;
                }
                
                let mut device_ids: Vec<*mut c_void> = vec![std::ptr::null_mut(); device_count as usize];
                get_devices(platform, 4, device_count, device_ids.as_mut_ptr(), &mut device_count);
                
                for device_id in device_ids {
                    if device_id.is_null() { continue; }
                    
                    let mut size: usize = 0;
                    
                    // Get device name
                    get_device_info(device_id, 0x102B, 0, std::ptr::null_mut(), &mut size);
                    let mut buf = vec![0u8; size];
                    get_device_info(device_id, 0x102B, size, buf.as_mut_ptr() as *mut c_void, &mut size);
                    let name = String::from_utf8_lossy(&buf).trim_end_matches('\0').to_string();
                    
                    // Get compute units
                    let mut compute_units: u32 = 0;
                    get_device_info(device_id, 0x1002, 4, &mut compute_units as *mut u32 as *mut c_void, &mut size);
                    
                    // Get memory
                    let mut memory: u64 = 0;
                    get_device_info(device_id, 0x101F, 8, &mut memory as *mut u64 as *mut c_void, &mut size);
                    let memory_gb = memory as f64 / (1024.0 * 1024.0 * 1024.0);
                    
                    // Get driver version
                    get_device_info(device_id, 0x102D, 0, std::ptr::null_mut(), &mut size);
                    let mut driver_buf = vec![0u8; size];
                    get_device_info(device_id, 0x102D, size, driver_buf.as_mut_ptr() as *mut c_void, &mut size);
                    let driver = String::from_utf8_lossy(&driver_buf).trim_end_matches('\0').to_string();
                    
                    let estimated_hashrate = estimate_kawpow_hashrate(&name, memory_gb);
                    // Assuming 1 TH/s network hashrate, 1440 blocks/day
                    let network_hashrate = 1_000_000.0; // 1 TH/s in MH/s
                    let estimated_blocks = (estimated_hashrate / network_hashrate) * 1440.0;
                    
                    let is_supported = !name.to_lowercase().contains("intel") && memory_gb >= 4.0;
                    
                    devices.push(GpuDevice {
                        index: global_index,
                        name,
                        vendor: vendor.clone(),
                        memory_gb,
                        compute_units,
                        driver_version: driver,
                        estimated_hashrate,
                        estimated_blocks_per_day: estimated_blocks,
                        is_supported,
                    });
                    
                    global_index += 1;
                }
            }
        }
    }
    
    devices
}
