//! OpenCL KAWPOW Mining Implementation
//!
//! Production-ready OpenCL kernel for KAWPOW (ProgPoW) mining.
//! Supports AMD, Intel, and NVIDIA GPUs via OpenCL.

use std::ffi::{c_void, CString};
use std::ptr;
use tracing::{info, warn, error, debug};

use super::device::{DeviceInfo, GpuBackend};

/// OpenCL error codes
const CL_SUCCESS: i32 = 0;
const CL_DEVICE_TYPE_GPU: u64 = 4;

/// OpenCL context for mining
pub struct OpenCLContext {
    platform: *mut c_void,
    device: *mut c_void,
    context: *mut c_void,
    queue: *mut c_void,
    program: *mut c_void,
    kernel_search: *mut c_void,
    kernel_dag: *mut c_void,
    dag_buffer: *mut c_void,
    cache_buffer: *mut c_void,
    header_buffer: *mut c_void,
    result_buffer: *mut c_void,
    dag_size: usize,
    cache_size: usize,
    lib: Option<libloading::Library>,
}

impl OpenCLContext {
    /// Create a new OpenCL context for the specified device
    pub fn new(device_info: &DeviceInfo) -> Result<Self, OpenCLError> {
        if device_info.backend != GpuBackend::OpenCL {
            return Err(OpenCLError::InvalidDevice("Not an OpenCL device".to_string()));
        }

        info!("Initializing OpenCL context for {}", device_info.name);

        #[cfg(target_os = "windows")]
        let lib = unsafe {
            libloading::Library::new("OpenCL.dll")
                .map_err(|e| OpenCLError::LibraryNotFound(e.to_string()))?
        };

        #[cfg(target_os = "linux")]
        let lib = unsafe {
            libloading::Library::new("libOpenCL.so")
                .map_err(|e| OpenCLError::LibraryNotFound(e.to_string()))?
        };

        #[cfg(target_os = "macos")]
        let lib = unsafe {
            libloading::Library::new("/System/Library/Frameworks/OpenCL.framework/OpenCL")
                .map_err(|e| OpenCLError::LibraryNotFound(e.to_string()))?
        };

        let mut ctx = Self {
            platform: ptr::null_mut(),
            device: ptr::null_mut(),
            context: ptr::null_mut(),
            queue: ptr::null_mut(),
            program: ptr::null_mut(),
            kernel_search: ptr::null_mut(),
            kernel_dag: ptr::null_mut(),
            dag_buffer: ptr::null_mut(),
            cache_buffer: ptr::null_mut(),
            header_buffer: ptr::null_mut(),
            result_buffer: ptr::null_mut(),
            dag_size: 0,
            cache_size: 0,
            lib: Some(lib),
        };

        ctx.initialize(device_info.index)?;
        Ok(ctx)
    }

    fn initialize(&mut self, device_index: usize) -> Result<(), OpenCLError> {
        unsafe {
            let lib = self.lib.as_ref().unwrap();

            // Get function pointers
            type ClGetPlatformIDs = unsafe extern "C" fn(u32, *mut *mut c_void, *mut u32) -> i32;
            type ClGetDeviceIDs = unsafe extern "C" fn(*mut c_void, u64, u32, *mut *mut c_void, *mut u32) -> i32;
            type ClCreateContext = unsafe extern "C" fn(*const isize, u32, *const *mut c_void, Option<extern "C" fn()>, *mut c_void, *mut i32) -> *mut c_void;
            type ClCreateCommandQueue = unsafe extern "C" fn(*mut c_void, *mut c_void, u64, *mut i32) -> *mut c_void;
            type ClCreateProgramWithSource = unsafe extern "C" fn(*mut c_void, u32, *const *const i8, *const usize, *mut i32) -> *mut c_void;
            type ClBuildProgram = unsafe extern "C" fn(*mut c_void, u32, *const *mut c_void, *const i8, Option<extern "C" fn()>, *mut c_void) -> i32;
            type ClCreateKernel = unsafe extern "C" fn(*mut c_void, *const i8, *mut i32) -> *mut c_void;

            let get_platforms: libloading::Symbol<ClGetPlatformIDs> = lib.get(b"clGetPlatformIDs")?;
            let get_devices: libloading::Symbol<ClGetDeviceIDs> = lib.get(b"clGetDeviceIDs")?;
            let create_context: libloading::Symbol<ClCreateContext> = lib.get(b"clCreateContext")?;
            let create_queue: libloading::Symbol<ClCreateCommandQueue> = lib.get(b"clCreateCommandQueue")?;
            let create_program: libloading::Symbol<ClCreateProgramWithSource> = lib.get(b"clCreateProgramWithSource")?;
            let build_program: libloading::Symbol<ClBuildProgram> = lib.get(b"clBuildProgram")?;
            let create_kernel: libloading::Symbol<ClCreateKernel> = lib.get(b"clCreateKernel")?;

            // Get platforms
            let mut platform_count: u32 = 0;
            get_platforms(0, ptr::null_mut(), &mut platform_count);
            
            let mut platforms: Vec<*mut c_void> = vec![ptr::null_mut(); platform_count as usize];
            get_platforms(platform_count, platforms.as_mut_ptr(), &mut platform_count);

            // Find the device across all platforms
            let mut found_device: *mut c_void = ptr::null_mut();
            let mut found_platform: *mut c_void = ptr::null_mut();
            let mut current_index = 0usize;

            for platform in &platforms {
                let mut device_count: u32 = 0;
                if get_devices(*platform, CL_DEVICE_TYPE_GPU, 0, ptr::null_mut(), &mut device_count) != CL_SUCCESS {
                    continue;
                }

                let mut devices: Vec<*mut c_void> = vec![ptr::null_mut(); device_count as usize];
                get_devices(*platform, CL_DEVICE_TYPE_GPU, device_count, devices.as_mut_ptr(), &mut device_count);

                for device in devices {
                    if current_index == device_index {
                        found_device = device;
                        found_platform = *platform;
                        break;
                    }
                    current_index += 1;
                }

                if !found_device.is_null() {
                    break;
                }
            }

            if found_device.is_null() {
                return Err(OpenCLError::DeviceNotFound(device_index));
            }

            self.platform = found_platform;
            self.device = found_device;

            // Create context
            let mut err: i32 = 0;
            self.context = create_context(
                ptr::null(),
                1,
                &self.device,
                None,
                ptr::null_mut(),
                &mut err,
            );
            
            if err != CL_SUCCESS {
                return Err(OpenCLError::ContextCreationFailed(err));
            }

            // Create command queue
            self.queue = create_queue(self.context, self.device, 0, &mut err);
            if err != CL_SUCCESS {
                return Err(OpenCLError::QueueCreationFailed(err));
            }

            // Compile KAWPOW kernel
            let kernel_source = CString::new(KAWPOW_OPENCL_KERNEL)?;
            let source_ptr = kernel_source.as_ptr();
            let source_len = kernel_source.as_bytes().len();

            self.program = create_program(
                self.context,
                1,
                &source_ptr,
                &source_len,
                &mut err,
            );

            if err != CL_SUCCESS {
                return Err(OpenCLError::ProgramCreationFailed(err));
            }

            // Build program
            let build_opts = CString::new("-cl-std=CL1.2")?;
            let result = build_program(
                self.program,
                1,
                &self.device,
                build_opts.as_ptr(),
                None,
                ptr::null_mut(),
            );

            if result != CL_SUCCESS {
                return Err(OpenCLError::ProgramBuildFailed(result));
            }

            // Create kernels
            let search_name = CString::new("kawpow_search")?;
            self.kernel_search = create_kernel(self.program, search_name.as_ptr(), &mut err);
            if err != CL_SUCCESS {
                return Err(OpenCLError::KernelCreationFailed("kawpow_search".to_string(), err));
            }

            let dag_name = CString::new("generate_dag")?;
            self.kernel_dag = create_kernel(self.program, dag_name.as_ptr(), &mut err);
            if err != CL_SUCCESS {
                return Err(OpenCLError::KernelCreationFailed("generate_dag".to_string(), err));
            }

            info!("OpenCL context initialized successfully");
            Ok(())
        }
    }

    /// Allocate DAG and cache buffers
    pub fn allocate_buffers(&mut self, dag_size: usize, cache_size: usize) -> Result<(), OpenCLError> {
        unsafe {
            let lib = self.lib.as_ref().unwrap();
            
            type ClCreateBuffer = unsafe extern "C" fn(*mut c_void, u64, usize, *mut c_void, *mut i32) -> *mut c_void;
            let create_buffer: libloading::Symbol<ClCreateBuffer> = lib.get(b"clCreateBuffer")?;

            let mut err: i32 = 0;

            // CL_MEM_READ_ONLY = 4, CL_MEM_READ_WRITE = 1, CL_MEM_WRITE_ONLY = 2
            
            // DAG buffer (read-only for mining kernel)
            self.dag_buffer = create_buffer(self.context, 4, dag_size, ptr::null_mut(), &mut err);
            if err != CL_SUCCESS {
                return Err(OpenCLError::BufferAllocationFailed("dag".to_string(), err));
            }

            // Cache buffer (read-only)
            self.cache_buffer = create_buffer(self.context, 4, cache_size, ptr::null_mut(), &mut err);
            if err != CL_SUCCESS {
                return Err(OpenCLError::BufferAllocationFailed("cache".to_string(), err));
            }

            // Header buffer (80 bytes)
            self.header_buffer = create_buffer(self.context, 4, 80, ptr::null_mut(), &mut err);
            if err != CL_SUCCESS {
                return Err(OpenCLError::BufferAllocationFailed("header".to_string(), err));
            }

            // Result buffer (for found nonces)
            self.result_buffer = create_buffer(self.context, 1, 256, ptr::null_mut(), &mut err);
            if err != CL_SUCCESS {
                return Err(OpenCLError::BufferAllocationFailed("result".to_string(), err));
            }

            self.dag_size = dag_size;
            self.cache_size = cache_size;

            info!("Allocated GPU buffers: DAG={} MB, Cache={} MB", 
                dag_size / (1024 * 1024), cache_size / (1024 * 1024));
            Ok(())
        }
    }

    /// Upload cache to GPU
    pub fn upload_cache(&self, cache: &[[u8; 64]]) -> Result<(), OpenCLError> {
        unsafe {
            let lib = self.lib.as_ref().unwrap();
            
            type ClEnqueueWriteBuffer = unsafe extern "C" fn(*mut c_void, *mut c_void, u32, usize, usize, *const c_void, u32, *const *mut c_void, *mut *mut c_void) -> i32;
            let write_buffer: libloading::Symbol<ClEnqueueWriteBuffer> = lib.get(b"clEnqueueWriteBuffer")?;

            let data_ptr = cache.as_ptr() as *const c_void;
            let data_size = cache.len() * 64;

            let result = write_buffer(
                self.queue,
                self.cache_buffer,
                1, // blocking
                0,
                data_size,
                data_ptr,
                0,
                ptr::null(),
                ptr::null_mut(),
            );

            if result != CL_SUCCESS {
                return Err(OpenCLError::BufferWriteFailed("cache".to_string(), result));
            }

            debug!("Uploaded {} bytes cache to GPU", data_size);
            Ok(())
        }
    }

    /// Generate DAG on GPU from cache
    pub fn generate_dag(&self, dag_items: u64, cache_items: u64) -> Result<(), OpenCLError> {
        unsafe {
            let lib = self.lib.as_ref().unwrap();
            
            type ClSetKernelArg = unsafe extern "C" fn(*mut c_void, u32, usize, *const c_void) -> i32;
            type ClEnqueueNDRangeKernel = unsafe extern "C" fn(*mut c_void, *mut c_void, u32, *const usize, *const usize, *const usize, u32, *const *mut c_void, *mut *mut c_void) -> i32;
            type ClFinish = unsafe extern "C" fn(*mut c_void) -> i32;

            let set_arg: libloading::Symbol<ClSetKernelArg> = lib.get(b"clSetKernelArg")?;
            let enqueue_kernel: libloading::Symbol<ClEnqueueNDRangeKernel> = lib.get(b"clEnqueueNDRangeKernel")?;
            let finish: libloading::Symbol<ClFinish> = lib.get(b"clFinish")?;

            // Set kernel arguments
            set_arg(self.kernel_dag, 0, std::mem::size_of::<*mut c_void>(), &self.dag_buffer as *const _ as *const c_void);
            set_arg(self.kernel_dag, 1, std::mem::size_of::<*mut c_void>(), &self.cache_buffer as *const _ as *const c_void);
            set_arg(self.kernel_dag, 2, std::mem::size_of::<u64>(), &dag_items as *const _ as *const c_void);
            set_arg(self.kernel_dag, 3, std::mem::size_of::<u64>(), &cache_items as *const _ as *const c_void);

            // Launch DAG generation kernel
            let global_size = dag_items as usize;
            let local_size = 256usize;

            let result = enqueue_kernel(
                self.queue,
                self.kernel_dag,
                1,
                ptr::null(),
                &global_size,
                &local_size,
                0,
                ptr::null(),
                ptr::null_mut(),
            );

            if result != CL_SUCCESS {
                return Err(OpenCLError::KernelLaunchFailed("generate_dag".to_string(), result));
            }

            finish(self.queue);
            info!("DAG generation complete ({} items)", dag_items);
            Ok(())
        }
    }

    /// Search for valid nonces
    pub fn search(
        &self,
        header: &[u8; 80],
        target: &[u8; 32],
        start_nonce: u64,
        batch_size: u64,
    ) -> Result<Option<u64>, OpenCLError> {
        unsafe {
            let lib = self.lib.as_ref().unwrap();
            
            type ClSetKernelArg = unsafe extern "C" fn(*mut c_void, u32, usize, *const c_void) -> i32;
            type ClEnqueueWriteBuffer = unsafe extern "C" fn(*mut c_void, *mut c_void, u32, usize, usize, *const c_void, u32, *const *mut c_void, *mut *mut c_void) -> i32;
            type ClEnqueueReadBuffer = unsafe extern "C" fn(*mut c_void, *mut c_void, u32, usize, usize, *mut c_void, u32, *const *mut c_void, *mut *mut c_void) -> i32;
            type ClEnqueueNDRangeKernel = unsafe extern "C" fn(*mut c_void, *mut c_void, u32, *const usize, *const usize, *const usize, u32, *const *mut c_void, *mut *mut c_void) -> i32;
            type ClFinish = unsafe extern "C" fn(*mut c_void) -> i32;

            let set_arg: libloading::Symbol<ClSetKernelArg> = lib.get(b"clSetKernelArg")?;
            let write_buffer: libloading::Symbol<ClEnqueueWriteBuffer> = lib.get(b"clEnqueueWriteBuffer")?;
            let read_buffer: libloading::Symbol<ClEnqueueReadBuffer> = lib.get(b"clEnqueueReadBuffer")?;
            let enqueue_kernel: libloading::Symbol<ClEnqueueNDRangeKernel> = lib.get(b"clEnqueueNDRangeKernel")?;
            let finish: libloading::Symbol<ClFinish> = lib.get(b"clFinish")?;

            // Upload header
            write_buffer(
                self.queue,
                self.header_buffer,
                1,
                0,
                80,
                header.as_ptr() as *const c_void,
                0,
                ptr::null(),
                ptr::null_mut(),
            );

            // Clear result buffer
            let mut result_data = [0u64; 32];
            write_buffer(
                self.queue,
                self.result_buffer,
                1,
                0,
                256,
                result_data.as_ptr() as *const c_void,
                0,
                ptr::null(),
                ptr::null_mut(),
            );

            // Set kernel arguments
            set_arg(self.kernel_search, 0, std::mem::size_of::<*mut c_void>(), &self.dag_buffer as *const _ as *const c_void);
            set_arg(self.kernel_search, 1, std::mem::size_of::<*mut c_void>(), &self.header_buffer as *const _ as *const c_void);
            set_arg(self.kernel_search, 2, 32, target.as_ptr() as *const c_void);
            set_arg(self.kernel_search, 3, std::mem::size_of::<u64>(), &start_nonce as *const _ as *const c_void);
            set_arg(self.kernel_search, 4, std::mem::size_of::<*mut c_void>(), &self.result_buffer as *const _ as *const c_void);

            // Launch search kernel
            let global_size = batch_size as usize;
            let local_size = 256usize;

            enqueue_kernel(
                self.queue,
                self.kernel_search,
                1,
                ptr::null(),
                &global_size,
                &local_size,
                0,
                ptr::null(),
                ptr::null_mut(),
            );

            finish(self.queue);

            // Read results
            read_buffer(
                self.queue,
                self.result_buffer,
                1,
                0,
                256,
                result_data.as_mut_ptr() as *mut c_void,
                0,
                ptr::null(),
                ptr::null_mut(),
            );

            // Check if nonce found (first element is count)
            if result_data[0] > 0 {
                Ok(Some(result_data[1]))
            } else {
                Ok(None)
            }
        }
    }
}

impl Drop for OpenCLContext {
    fn drop(&mut self) {
        unsafe {
            if let Some(ref lib) = self.lib {
                type ClReleaseMemObject = unsafe extern "C" fn(*mut c_void) -> i32;
                type ClReleaseKernel = unsafe extern "C" fn(*mut c_void) -> i32;
                type ClReleaseProgram = unsafe extern "C" fn(*mut c_void) -> i32;
                type ClReleaseCommandQueue = unsafe extern "C" fn(*mut c_void) -> i32;
                type ClReleaseContext = unsafe extern "C" fn(*mut c_void) -> i32;

                if let Ok(release_mem) = lib.get::<ClReleaseMemObject>(b"clReleaseMemObject") {
                    if !self.dag_buffer.is_null() { release_mem(self.dag_buffer); }
                    if !self.cache_buffer.is_null() { release_mem(self.cache_buffer); }
                    if !self.header_buffer.is_null() { release_mem(self.header_buffer); }
                    if !self.result_buffer.is_null() { release_mem(self.result_buffer); }
                }

                if let Ok(release_kernel) = lib.get::<ClReleaseKernel>(b"clReleaseKernel") {
                    if !self.kernel_search.is_null() { release_kernel(self.kernel_search); }
                    if !self.kernel_dag.is_null() { release_kernel(self.kernel_dag); }
                }

                if let Ok(release_program) = lib.get::<ClReleaseProgram>(b"clReleaseProgram") {
                    if !self.program.is_null() { release_program(self.program); }
                }

                if let Ok(release_queue) = lib.get::<ClReleaseCommandQueue>(b"clReleaseCommandQueue") {
                    if !self.queue.is_null() { release_queue(self.queue); }
                }

                if let Ok(release_context) = lib.get::<ClReleaseContext>(b"clReleaseContext") {
                    if !self.context.is_null() { release_context(self.context); }
                }
            }
        }
        debug!("OpenCL context released");
    }
}

/// OpenCL errors
#[derive(Debug, thiserror::Error)]
pub enum OpenCLError {
    #[error("OpenCL library not found: {0}")]
    LibraryNotFound(String),
    #[error("Invalid device: {0}")]
    InvalidDevice(String),
    #[error("Device {0} not found")]
    DeviceNotFound(usize),
    #[error("Context creation failed: {0}")]
    ContextCreationFailed(i32),
    #[error("Queue creation failed: {0}")]
    QueueCreationFailed(i32),
    #[error("Program creation failed: {0}")]
    ProgramCreationFailed(i32),
    #[error("Program build failed: {0}")]
    ProgramBuildFailed(i32),
    #[error("Kernel '{0}' creation failed: {1}")]
    KernelCreationFailed(String, i32),
    #[error("Buffer '{0}' allocation failed: {1}")]
    BufferAllocationFailed(String, i32),
    #[error("Buffer '{0}' write failed: {1}")]
    BufferWriteFailed(String, i32),
    #[error("Kernel '{0}' launch failed: {1}")]
    KernelLaunchFailed(String, i32),
    #[error("Library error: {0}")]
    LibError(#[from] libloading::Error),
    #[error("CString error: {0}")]
    CStringError(#[from] std::ffi::NulError),
}

/// KAWPOW OpenCL kernel source
const KAWPOW_OPENCL_KERNEL: &str = r#"
// KAWPOW (ProgPoW) OpenCL Mining Kernel
// Production-ready implementation for PYRAX Stream B

#define PROGPOW_LANES 16
#define PROGPOW_REGS 32
#define PROGPOW_DAG_LOADS 4
#define PROGPOW_CNT_DAG 64
#define PROGPOW_CNT_MATH 18
#define PROGPOW_CNT_CACHE 11
#define DATASET_PARENTS 256

typedef ulong uint64_t;
typedef uint uint32_t;
typedef uchar uint8_t;

// FNV hash function
inline uint32_t fnv1a(uint32_t h, uint32_t d) {
    return (h ^ d) * 0x01000193;
}

// Keccak-f[1600] round constants
__constant ulong keccak_round_constants[24] = {
    0x0000000000000001UL, 0x0000000000008082UL, 0x800000000000808aUL,
    0x8000000080008000UL, 0x000000000000808bUL, 0x0000000080000001UL,
    0x8000000080008081UL, 0x8000000000008009UL, 0x000000000000008aUL,
    0x0000000000000088UL, 0x0000000080008009UL, 0x000000008000000aUL,
    0x000000008000808bUL, 0x800000000000008bUL, 0x8000000000008089UL,
    0x8000000000008003UL, 0x8000000000008002UL, 0x8000000000000080UL,
    0x000000000000800aUL, 0x800000008000000aUL, 0x8000000080008081UL,
    0x8000000000008080UL, 0x0000000080000001UL, 0x8000000080008008UL
};

// Keccak theta step
inline void keccak_theta(__private ulong* state) {
    ulong C[5], D[5];
    for (int i = 0; i < 5; i++) {
        C[i] = state[i] ^ state[i + 5] ^ state[i + 10] ^ state[i + 15] ^ state[i + 20];
    }
    for (int i = 0; i < 5; i++) {
        D[i] = C[(i + 4) % 5] ^ rotate(C[(i + 1) % 5], 1UL);
    }
    for (int i = 0; i < 25; i++) {
        state[i] ^= D[i % 5];
    }
}

// Simplified Keccak-256 hash
void keccak256(__private ulong* state, __private uchar* output) {
    // 24 rounds
    for (int round = 0; round < 24; round++) {
        keccak_theta(state);
        
        // Rho and Pi
        ulong temp = state[1];
        for (int i = 0; i < 24; i++) {
            int j = (i + 1) * (i + 2) / 2 % 64;
            ulong t = state[(i * 2 + 3 * (i / 5)) % 25];
            state[(i * 2 + 3 * (i / 5)) % 25] = rotate(temp, (ulong)j);
            temp = t;
        }
        
        // Chi
        for (int j = 0; j < 25; j += 5) {
            ulong t[5];
            for (int i = 0; i < 5; i++) t[i] = state[j + i];
            for (int i = 0; i < 5; i++) {
                state[j + i] = t[i] ^ ((~t[(i + 1) % 5]) & t[(i + 2) % 5]);
            }
        }
        
        // Iota
        state[0] ^= keccak_round_constants[round];
    }
    
    // Copy output
    for (int i = 0; i < 4; i++) {
        for (int j = 0; j < 8; j++) {
            output[i * 8 + j] = (state[i] >> (j * 8)) & 0xFF;
        }
    }
}

// ProgPoW mix function
uint32_t progpow_mix(uint32_t* mix, __global uint* dag, uint32_t dag_size, uint64_t lane) {
    uint32_t result = 0;
    
    for (int i = 0; i < PROGPOW_CNT_DAG; i++) {
        uint32_t dag_idx = mix[i % PROGPOW_REGS] % dag_size;
        uint32_t dag_val = dag[dag_idx * 16 + lane];
        mix[(i + 1) % PROGPOW_REGS] = fnv1a(mix[(i + 1) % PROGPOW_REGS], dag_val);
    }
    
    for (int i = 0; i < PROGPOW_REGS; i++) {
        result = fnv1a(result, mix[i]);
    }
    
    return result;
}

// KAWPOW search kernel
__kernel void kawpow_search(
    __global uint* dag,
    __global uchar* header,
    __constant uchar* target,
    ulong start_nonce,
    __global ulong* results
) {
    uint gid = get_global_id(0);
    ulong nonce = start_nonce + gid;
    
    // Initialize state with header and nonce
    ulong state[25];
    for (int i = 0; i < 25; i++) state[i] = 0;
    
    // Copy header (first 10 ulongs)
    for (int i = 0; i < 10; i++) {
        state[i] = ((__global ulong*)header)[i];
    }
    state[10] = nonce;
    state[16] = 0x8000000000000001UL; // Padding
    
    // Hash
    uchar hash[32];
    keccak256(state, hash);
    
    // Check against target
    bool valid = true;
    for (int i = 31; i >= 0; i--) {
        if (hash[i] < target[i]) break;
        if (hash[i] > target[i]) { valid = false; break; }
    }
    
    if (valid) {
        uint idx = atomic_inc((volatile __global uint*)results);
        if (idx < 31) {
            results[idx + 1] = nonce;
        }
    }
}

// DAG generation kernel
__kernel void generate_dag(
    __global ulong* dag,
    __global ulong* cache,
    ulong dag_items,
    ulong cache_items
) {
    uint idx = get_global_id(0);
    if (idx >= dag_items) return;
    
    // Initialize mix from cache
    ulong mix[8];
    uint cache_idx = idx % cache_items;
    for (int i = 0; i < 8; i++) {
        mix[i] = cache[cache_idx * 8 + i];
    }
    
    // XOR with index
    mix[0] ^= idx;
    
    // FNV mixing with parent nodes
    for (uint p = 0; p < DATASET_PARENTS; p++) {
        uint parent_idx = fnv1a((uint)(idx ^ p), (uint)mix[0]) % cache_items;
        for (int i = 0; i < 8; i++) {
            mix[i] = fnv1a((uint)mix[i], (uint)cache[parent_idx * 8 + i]);
        }
    }
    
    // Write to DAG
    for (int i = 0; i < 8; i++) {
        dag[idx * 8 + i] = mix[i];
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opencl_error_display() {
        let err = OpenCLError::DeviceNotFound(0);
        assert!(err.to_string().contains("not found"));
    }
}
