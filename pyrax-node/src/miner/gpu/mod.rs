//! GPU Mining Module for PYRAX
//!
//! Production-ready GPU mining with CUDA and OpenCL support.
//! Implements KAWPOW (ProgPoW variant) for Stream B mining.

pub mod device;
pub mod opencl;
pub mod cuda;
pub mod kernel;

pub use device::{GpuDevice, GpuBackend, DeviceInfo, detect_devices};
pub use kernel::{GpuMiner, GpuMinerConfig, MiningResult, GpuError};
