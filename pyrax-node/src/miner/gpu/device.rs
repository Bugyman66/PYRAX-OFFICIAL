//! GPU Device Detection and Management
//!
//! Production-ready GPU device enumeration for CUDA and OpenCL backends.

use std::fmt;
use tracing::{info, warn, debug};

/// GPU Backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuBackend {
    Cuda,
    OpenCL,
}

impl fmt::Display for GpuBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpuBackend::Cuda => write!(f, "CUDA"),
            GpuBackend::OpenCL => write!(f, "OpenCL"),
        }
    }
}

/// GPU Device information
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub index: usize,
    pub name: String,
    pub backend: GpuBackend,
    pub compute_units: u32,
    pub memory_bytes: u64,
    pub max_work_group_size: usize,
    pub driver_version: String,
    pub pcie_bus_id: Option<String>,
}

impl DeviceInfo {
    pub fn memory_mb(&self) -> u64 {
        self.memory_bytes / (1024 * 1024)
    }

    pub fn memory_gb(&self) -> f64 {
        self.memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }
}

impl fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} ({}) - {} CUs, {:.1} GB",
            self.index,
            self.name,
            self.backend,
            self.compute_units,
            self.memory_gb()
        )
    }
}

/// GPU Device handle for mining operations
pub struct GpuDevice {
    pub info: DeviceInfo,
    pub platform_index: usize,
    pub device_index: usize,
    backend: GpuBackend,
}

impl GpuDevice {
    pub fn new(info: DeviceInfo, platform_index: usize, device_index: usize) -> Self {
        Self {
            backend: info.backend,
            info,
            platform_index,
            device_index,
        }
    }

    pub fn backend(&self) -> GpuBackend {
        self.backend
    }

    pub fn is_cuda(&self) -> bool {
        self.backend == GpuBackend::Cuda
    }

    pub fn is_opencl(&self) -> bool {
        self.backend == GpuBackend::OpenCL
    }
}

/// Detect all available GPU devices
pub fn detect_devices() -> Vec<DeviceInfo> {
    let mut devices = Vec::new();
    let mut global_index = 0;

    // Detect CUDA devices
    #[cfg(feature = "cuda")]
    {
        if let Ok(cuda_devices) = detect_cuda_devices() {
            for mut device in cuda_devices {
                device.index = global_index;
                global_index += 1;
                devices.push(device);
            }
        }
    }

    // Detect OpenCL devices
    if let Ok(opencl_devices) = detect_opencl_devices() {
        for mut device in opencl_devices {
            device.index = global_index;
            global_index += 1;
            devices.push(device);
        }
    }

    if devices.is_empty() {
        warn!("No GPU devices detected");
    } else {
        info!("Detected {} GPU device(s):", devices.len());
        for device in &devices {
            info!("  {}", device);
        }
    }

    devices
}

/// Detect OpenCL devices
fn detect_opencl_devices() -> Result<Vec<DeviceInfo>, String> {
    let mut devices = Vec::new();

    // Try to load OpenCL dynamically
    #[cfg(target_os = "windows")]
    let lib_path = "OpenCL.dll";
    #[cfg(target_os = "linux")]
    let lib_path = "libOpenCL.so";
    #[cfg(target_os = "macos")]
    let lib_path = "/System/Library/Frameworks/OpenCL.framework/OpenCL";

    // Check if OpenCL is available by trying to get platform count
    let platform_count = unsafe {
        match get_opencl_platform_count() {
            Ok(count) => count,
            Err(e) => {
                debug!("OpenCL not available: {}", e);
                return Ok(devices);
            }
        }
    };

    debug!("Found {} OpenCL platform(s)", platform_count);

    for platform_idx in 0..platform_count {
        if let Ok(platform_devices) = get_opencl_platform_devices(platform_idx) {
            for (device_idx, device_info) in platform_devices.into_iter().enumerate() {
                devices.push(device_info);
            }
        }
    }

    Ok(devices)
}

/// Get OpenCL platform count using raw API
unsafe fn get_opencl_platform_count() -> Result<u32, String> {
    #[cfg(target_os = "windows")]
    {
        use std::ffi::c_void;
        
        type ClGetPlatformIDs = unsafe extern "C" fn(u32, *mut *mut c_void, *mut u32) -> i32;
        
        let lib = libloading::Library::new("OpenCL.dll")
            .map_err(|e| format!("Failed to load OpenCL.dll: {}", e))?;
        
        let func: libloading::Symbol<ClGetPlatformIDs> = lib
            .get(b"clGetPlatformIDs")
            .map_err(|e| format!("Failed to get clGetPlatformIDs: {}", e))?;
        
        let mut count: u32 = 0;
        let result = func(0, std::ptr::null_mut(), &mut count);
        
        if result != 0 {
            return Err(format!("clGetPlatformIDs failed with code {}", result));
        }
        
        Ok(count)
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // For non-Windows, try dlopen
        Err("OpenCL detection not implemented for this platform".to_string())
    }
}

/// Get devices for an OpenCL platform
fn get_opencl_platform_devices(platform_idx: u32) -> Result<Vec<DeviceInfo>, String> {
    let mut devices = Vec::new();
    
    #[cfg(target_os = "windows")]
    {
        use std::ffi::c_void;
        
        type ClGetPlatformIDs = unsafe extern "C" fn(u32, *mut *mut c_void, *mut u32) -> i32;
        type ClGetDeviceIDs = unsafe extern "C" fn(*mut c_void, u64, u32, *mut *mut c_void, *mut u32) -> i32;
        type ClGetDeviceInfo = unsafe extern "C" fn(*mut c_void, u32, usize, *mut c_void, *mut usize) -> i32;
        
        unsafe {
            let lib = libloading::Library::new("OpenCL.dll")
                .map_err(|e| format!("Failed to load OpenCL.dll: {}", e))?;
            
            let get_platforms: libloading::Symbol<ClGetPlatformIDs> = lib
                .get(b"clGetPlatformIDs")
                .map_err(|e| format!("Failed to get clGetPlatformIDs: {}", e))?;
            
            let get_devices: libloading::Symbol<ClGetDeviceIDs> = lib
                .get(b"clGetDeviceIDs")
                .map_err(|e| format!("Failed to get clGetDeviceIDs: {}", e))?;
            
            let get_device_info: libloading::Symbol<ClGetDeviceInfo> = lib
                .get(b"clGetDeviceInfo")
                .map_err(|e| format!("Failed to get clGetDeviceInfo: {}", e))?;
            
            // Get platforms
            let mut platform_count: u32 = 0;
            get_platforms(0, std::ptr::null_mut(), &mut platform_count);
            
            if platform_idx >= platform_count {
                return Ok(devices);
            }
            
            let mut platforms: Vec<*mut c_void> = vec![std::ptr::null_mut(); platform_count as usize];
            get_platforms(platform_count, platforms.as_mut_ptr(), &mut platform_count);
            
            let platform = platforms[platform_idx as usize];
            
            // Get GPU devices (CL_DEVICE_TYPE_GPU = 4)
            let mut device_count: u32 = 0;
            let result = get_devices(platform, 4, 0, std::ptr::null_mut(), &mut device_count);
            
            if result != 0 || device_count == 0 {
                return Ok(devices);
            }
            
            let mut device_ids: Vec<*mut c_void> = vec![std::ptr::null_mut(); device_count as usize];
            get_devices(platform, 4, device_count, device_ids.as_mut_ptr(), &mut device_count);
            
            for (idx, &device_id) in device_ids.iter().enumerate() {
                if device_id.is_null() {
                    continue;
                }
                
                // Get device name (CL_DEVICE_NAME = 0x102B)
                let mut name_size: usize = 0;
                get_device_info(device_id, 0x102B, 0, std::ptr::null_mut(), &mut name_size);
                let mut name_buf = vec![0u8; name_size];
                get_device_info(device_id, 0x102B, name_size, name_buf.as_mut_ptr() as *mut c_void, &mut name_size);
                let name = String::from_utf8_lossy(&name_buf).trim_end_matches('\0').to_string();
                
                // Get compute units (CL_DEVICE_MAX_COMPUTE_UNITS = 0x1002)
                let mut compute_units: u32 = 0;
                get_device_info(device_id, 0x1002, 4, &mut compute_units as *mut u32 as *mut c_void, &mut name_size);
                
                // Get global memory (CL_DEVICE_GLOBAL_MEM_SIZE = 0x101F)
                let mut memory: u64 = 0;
                get_device_info(device_id, 0x101F, 8, &mut memory as *mut u64 as *mut c_void, &mut name_size);
                
                // Get max work group size (CL_DEVICE_MAX_WORK_GROUP_SIZE = 0x1004)
                let mut work_group_size: usize = 0;
                get_device_info(device_id, 0x1004, std::mem::size_of::<usize>(), &mut work_group_size as *mut usize as *mut c_void, &mut name_size);
                
                // Get driver version (CL_DRIVER_VERSION = 0x102D)
                let mut version_size: usize = 0;
                get_device_info(device_id, 0x102D, 0, std::ptr::null_mut(), &mut version_size);
                let mut version_buf = vec![0u8; version_size];
                get_device_info(device_id, 0x102D, version_size, version_buf.as_mut_ptr() as *mut c_void, &mut version_size);
                let driver_version = String::from_utf8_lossy(&version_buf).trim_end_matches('\0').to_string();
                
                devices.push(DeviceInfo {
                    index: idx,
                    name,
                    backend: GpuBackend::OpenCL,
                    compute_units,
                    memory_bytes: memory,
                    max_work_group_size: work_group_size,
                    driver_version,
                    pcie_bus_id: None,
                });
            }
        }
    }
    
    Ok(devices)
}

#[cfg(feature = "cuda")]
fn detect_cuda_devices() -> Result<Vec<DeviceInfo>, String> {
    use std::ffi::c_void;
    
    type CuInit = unsafe extern "C" fn(u32) -> i32;
    type CuDeviceGetCount = unsafe extern "C" fn(*mut i32) -> i32;
    type CuDeviceGet = unsafe extern "C" fn(*mut i32, i32) -> i32;
    type CuDeviceGetName = unsafe extern "C" fn(*mut u8, i32, i32) -> i32;
    type CuDeviceTotalMem = unsafe extern "C" fn(*mut usize, i32) -> i32;
    type CuDeviceGetAttribute = unsafe extern "C" fn(*mut i32, i32, i32) -> i32;
    
    let mut devices = Vec::new();
    
    unsafe {
        #[cfg(target_os = "windows")]
        let lib = libloading::Library::new("nvcuda.dll")
            .map_err(|e| format!("CUDA not available: {}", e))?;
        
        #[cfg(not(target_os = "windows"))]
        let lib = libloading::Library::new("libcuda.so")
            .map_err(|e| format!("CUDA not available: {}", e))?;
        
        let cu_init: libloading::Symbol<CuInit> = lib.get(b"cuInit")?;
        let cu_device_get_count: libloading::Symbol<CuDeviceGetCount> = lib.get(b"cuDeviceGetCount")?;
        let cu_device_get: libloading::Symbol<CuDeviceGet> = lib.get(b"cuDeviceGet")?;
        let cu_device_get_name: libloading::Symbol<CuDeviceGetName> = lib.get(b"cuDeviceGetName")?;
        let cu_device_total_mem: libloading::Symbol<CuDeviceTotalMem> = lib.get(b"cuDeviceTotalMem_v2")?;
        let cu_device_get_attribute: libloading::Symbol<CuDeviceGetAttribute> = lib.get(b"cuDeviceGetAttribute")?;
        
        // Initialize CUDA
        if cu_init(0) != 0 {
            return Err("cuInit failed".to_string());
        }
        
        // Get device count
        let mut count: i32 = 0;
        if cu_device_get_count(&mut count) != 0 {
            return Err("cuDeviceGetCount failed".to_string());
        }
        
        for i in 0..count {
            let mut device: i32 = 0;
            if cu_device_get(&mut device, i) != 0 {
                continue;
            }
            
            // Get name
            let mut name_buf = [0u8; 256];
            cu_device_get_name(name_buf.as_mut_ptr(), 256, device);
            let name = String::from_utf8_lossy(&name_buf).trim_end_matches('\0').to_string();
            
            // Get memory
            let mut memory: usize = 0;
            cu_device_total_mem(&mut memory, device);
            
            // Get compute units (SM count) - CU_DEVICE_ATTRIBUTE_MULTIPROCESSOR_COUNT = 16
            let mut sm_count: i32 = 0;
            cu_device_get_attribute(&mut sm_count, 16, device);
            
            // Get max threads per block - CU_DEVICE_ATTRIBUTE_MAX_THREADS_PER_BLOCK = 1
            let mut max_threads: i32 = 0;
            cu_device_get_attribute(&mut max_threads, 1, device);
            
            // Get PCI bus ID - CU_DEVICE_ATTRIBUTE_PCI_BUS_ID = 33
            let mut pci_bus: i32 = 0;
            cu_device_get_attribute(&mut pci_bus, 33, device);
            
            // Get PCI device ID - CU_DEVICE_ATTRIBUTE_PCI_DEVICE_ID = 34
            let mut pci_device: i32 = 0;
            cu_device_get_attribute(&mut pci_device, 34, device);
            
            devices.push(DeviceInfo {
                index: i as usize,
                name,
                backend: GpuBackend::Cuda,
                compute_units: sm_count as u32,
                memory_bytes: memory as u64,
                max_work_group_size: max_threads as usize,
                driver_version: "CUDA".to_string(),
                pcie_bus_id: Some(format!("{:02x}:{:02x}.0", pci_bus, pci_device)),
            });
        }
    }
    
    Ok(devices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_info_display() {
        let info = DeviceInfo {
            index: 0,
            name: "Test GPU".to_string(),
            backend: GpuBackend::OpenCL,
            compute_units: 64,
            memory_bytes: 8 * 1024 * 1024 * 1024, // 8 GB
            max_work_group_size: 1024,
            driver_version: "1.0".to_string(),
            pcie_bus_id: None,
        };
        
        assert!(info.to_string().contains("Test GPU"));
        assert!(info.to_string().contains("64 CUs"));
        assert_eq!(info.memory_gb(), 8.0);
    }

    #[test]
    fn test_gpu_backend() {
        assert_eq!(format!("{}", GpuBackend::Cuda), "CUDA");
        assert_eq!(format!("{}", GpuBackend::OpenCL), "OpenCL");
    }
}
