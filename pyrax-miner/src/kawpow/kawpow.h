#pragma once

#include <cstdint>
#include <array>
#include <vector>
#include <memory>

namespace kawpow {

// Constants
constexpr uint64_t EPOCH_LENGTH = 7500;
constexpr size_t CACHE_BYTES_INIT = 16 * 1024 * 1024;  // 16 MB
constexpr size_t CACHE_BYTES_GROWTH = 128 * 1024;      // 128 KB per epoch
constexpr size_t DAG_BYTES_INIT = 1024 * 1024 * 1024;  // 1 GB
constexpr size_t DAG_BYTES_GROWTH = 8 * 1024 * 1024;   // 8 MB per epoch

constexpr int PROGPOW_LANES = 16;
constexpr int PROGPOW_REGS = 32;
constexpr int PROGPOW_DAG_LOADS = 4;
constexpr int PROGPOW_CNT_DAG = 64;
constexpr int PROGPOW_CNT_MATH = 18;
constexpr int PROGPOW_CNT_CACHE = 11;

// Types
using Hash256 = std::array<uint8_t, 32>;
using Hash512 = std::array<uint8_t, 64>;

struct Result {
    Hash256 mix_hash;
    Hash256 final_hash;
};

struct BlockHeader {
    uint32_t version;
    Hash256 parent_hash;
    Hash256 merkle_root;
    Hash256 state_root;
    uint64_t timestamp;
    uint64_t difficulty;
    uint64_t nonce;
    uint64_t height;
    uint64_t extra_nonce;
    std::array<uint8_t, 20> beneficiary;
};

// Functions
void init();
void cleanup();

uint64_t get_epoch(uint64_t height);
size_t get_cache_size(uint64_t epoch);
size_t get_dag_size(uint64_t epoch);

Hash256 compute_seed(uint64_t epoch);

// Cache management
class Cache {
public:
    explicit Cache(uint64_t epoch);
    ~Cache();
    
    const uint8_t* data() const { return data_.data(); }
    size_t size() const { return data_.size(); }
    uint64_t epoch() const { return epoch_; }
    
private:
    uint64_t epoch_;
    std::vector<uint8_t> data_;
};

// DAG management
class DAG {
public:
    explicit DAG(const Cache& cache);
    ~DAG();
    
    const uint8_t* data() const { return data_.data(); }
    size_t size() const { return data_.size(); }
    uint64_t epoch() const { return epoch_; }
    
    // For GPU DAG generation
    void* gpu_data() const { return gpu_data_; }
    void set_gpu_data(void* ptr) { gpu_data_ = ptr; }
    
private:
    uint64_t epoch_;
    std::vector<uint8_t> data_;
    void* gpu_data_ = nullptr;
};

// Mining functions
Result hash(
    const Hash256& header_hash,
    uint64_t nonce,
    uint64_t height,
    const DAG& dag
);

Result hash_light(
    const Hash256& header_hash,
    uint64_t nonce,
    uint64_t height,
    const Cache& cache
);

bool verify(
    const Hash256& header_hash,
    uint64_t nonce,
    uint64_t height,
    const Hash256& mix_hash,
    const Hash256& final_hash,
    const Cache& cache
);

Hash256 compute_header_hash(const BlockHeader& header);

bool meets_target(const Hash256& hash, uint64_t difficulty);
Hash256 difficulty_to_target(uint64_t difficulty);

} // namespace kawpow
