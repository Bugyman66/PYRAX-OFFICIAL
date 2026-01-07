cmake_minimum_required(VERSION 3.20)
project(pyrax-miner VERSION 0.1.0 LANGUAGES CXX CUDA)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Options
option(ENABLE_CUDA "Enable CUDA support" ON)
option(ENABLE_OPENCL "Enable OpenCL support" ON)
option(ENABLE_CPU "Enable CPU mining (reference)" ON)

# Find packages
find_package(Threads REQUIRED)

if(ENABLE_CUDA)
    find_package(CUDAToolkit)
    if(CUDAToolkit_FOUND)
        enable_language(CUDA)
        set(CMAKE_CUDA_STANDARD 17)
        add_definitions(-DHAVE_CUDA)
        message(STATUS "CUDA support enabled")
    else()
        message(WARNING "CUDA not found, disabling CUDA support")
        set(ENABLE_CUDA OFF)
    endif()
endif()

if(ENABLE_OPENCL)
    find_package(OpenCL)
    if(OpenCL_FOUND)
        add_definitions(-DHAVE_OPENCL)
        message(STATUS "OpenCL support enabled")
    else()
        message(WARNING "OpenCL not found, disabling OpenCL support")
        set(ENABLE_OPENCL OFF)
    endif()
endif()

# Source files
set(COMMON_SOURCES
    src/main.cpp
    src/kawpow/kawpow.cpp
    src/kawpow/dag.cpp
    src/kawpow/cache.cpp
    src/stratum/client.cpp
    src/stratum/job.cpp
    src/utils/logger.cpp
    src/utils/hex.cpp
)

set(CPU_SOURCES
    src/cpu/cpu_miner.cpp
)

if(ENABLE_CUDA)
    set(CUDA_SOURCES
        src/cuda/cuda_miner.cu
        src/cuda/kawpow_kernel.cu
        src/cuda/dag_kernel.cu
    )
endif()

if(ENABLE_OPENCL)
    set(OPENCL_SOURCES
        src/opencl/opencl_miner.cpp
        src/opencl/kernels.cpp
    )
endif()

# Main executable
add_executable(pyrax-miner
    ${COMMON_SOURCES}
    ${CPU_SOURCES}
    $<$<BOOL:${ENABLE_CUDA}>:${CUDA_SOURCES}>
    $<$<BOOL:${ENABLE_OPENCL}>:${OPENCL_SOURCES}>
)

target_include_directories(pyrax-miner PRIVATE
    ${CMAKE_SOURCE_DIR}/src
    ${CMAKE_SOURCE_DIR}/include
)

target_link_libraries(pyrax-miner PRIVATE
    Threads::Threads
)

if(ENABLE_CUDA)
    target_link_libraries(pyrax-miner PRIVATE CUDA::cudart)
    set_target_properties(pyrax-miner PROPERTIES
        CUDA_SEPARABLE_COMPILATION ON
        CUDA_ARCHITECTURES "61;70;75;80;86;89;90"
    )
endif()

if(ENABLE_OPENCL)
    target_link_libraries(pyrax-miner PRIVATE OpenCL::OpenCL)
endif()

# Installation
install(TARGETS pyrax-miner
    RUNTIME DESTINATION bin
)

# CPack configuration
set(CPACK_PACKAGE_NAME "pyrax-miner")
set(CPACK_PACKAGE_VERSION ${PROJECT_VERSION})
set(CPACK_PACKAGE_VENDOR "PYRAX Team")
include(CPack)
