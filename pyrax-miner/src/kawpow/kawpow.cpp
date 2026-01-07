#include "kawpow.h"
#include "../utils/logger.h"
#include <cstring>
#include <algorithm>

// Keccak implementation (simplified - use a proper library in production)
extern "C" {
    void keccak_256(const uint8_t* input, size_t input_len, uint8_t* output);
    void keccak_512(const uint8_t* input, size_t input_len, uint8_t* output);
}

namespace kawpow {

static bool g_initialized = false;

void init() {
    if (g_initialized) return;
    LOG_INFO("Initializing KAWPOW...");
    g_initialized = true;
}

void cleanup() {
    if (!g_initialized) return;
    LOG_INFO("Cleaning up KAWPOW...");
    g_initialized = false;
}

uint64_t get_epoch(uint64_t height) {
    return height / EPOCH_LENGTH;
}

size_t get_cache_size(uint64_t epoch) {
    return CACHE_BYTES_INIT + CACHE_BYTES_GROWTH * epoch;
}

size_t get_dag_size(uint64_t epoch) {
    return DAG_BYTES_INIT + DAG_BYTES_GROWTH * epoch;
}

Hash256 compute_seed(uint64_t epoch) {
    Hash256 seed{};
    for (uint64_t i = 0; i < epoch; i++) {
        keccak_256(seed.data(), 32, seed.data());
    }
    return seed;
}

// FNV-1a hash
static uint32_t fnv1a(uint32_t u, uint32_t v) {
    constexpr uint32_t FNV_PRIME = 0x01000193;
    return (u ^ v) * FNV_PRIME;
}

// ProgPoW math operations
static uint32_t progpow_math(uint32_t a, uint32_t b, uint32_t r) {
    switch (r % 11) {
        case 0: return a + b;
        case 1: return a * b;
        case 2: return static_cast<uint32_t>((static_cast<uint64_t>(a) * b) >> 32);
        case 3: return std::min(a, b);
        case 4: return (a << (b % 32)) | (a >> (32 - (b % 32)));
        case 5: return (a >> (b % 32)) | (a << (32 - (b % 32)));
        case 6: return a & b;
        case 7: return a | b;
        case 8: return a ^ b;
        case 9: return __builtin_clz(a) + __builtin_clz(b);
        case 10: return __builtin_popcount(a) + __builtin_popcount(b);
        default: return a;
    }
}

Cache::Cache(uint64_t epoch) : epoch_(epoch) {
    LOG_INFO("Generating cache for epoch {}...", epoch);
    
    size_t cache_size = get_cache_size(epoch);
    size_t num_items = cache_size / 64;
    data_.resize(cache_size);
    
    // Generate seed
    Hash256 seed = compute_seed(epoch);
    
    // Initialize first item with Keccak-512
    keccak_512(seed.data(), 32, data_.data());
    
    // Generate remaining items
    for (size_t i = 1; i < num_items; i++) {
        keccak_512(data_.data() + (i - 1) * 64, 64, data_.data() + i * 64);
    }
    
    // RandMemoHash passes
    for (int pass = 0; pass < 3; pass++) {
        for (size_t i = 0; i < num_items; i++) {
            uint32_t v;
            std::memcpy(&v, data_.data() + i * 64, 4);
            v = v % num_items;
            
            size_t prev_idx = (i == 0) ? num_items - 1 : i - 1;
            
            uint8_t temp[64];
            for (int j = 0; j < 64; j++) {
                temp[j] = data_[prev_idx * 64 + j] ^ data_[v * 64 + j];
            }
            keccak_512(temp, 64, data_.data() + i * 64);
        }
    }
    
    LOG_INFO("Cache generated ({} MB)", cache_size / (1024 * 1024));
}

Cache::~Cache() = default;

DAG::DAG(const Cache& cache) : epoch_(cache.epoch()) {
    LOG_INFO("Generating DAG for epoch {}...", epoch_);
    
    size_t dag_size = get_dag_size(epoch_);
    size_t num_items = dag_size / 64;
    data_.resize(dag_size);
    
    const uint8_t* cache_data = cache.data();
    size_t cache_items = cache.size() / 64;
    
    // Generate DAG items
    for (size_t i = 0; i < num_items; i++) {
        // Mix cache items
        uint32_t mix_idx = i % cache_items;
        
        uint8_t mix[64];
        std::memcpy(mix, cache_data + mix_idx * 64, 64);
        
        // XOR with index
        uint32_t* mix32 = reinterpret_cast<uint32_t*>(mix);
        mix32[0] ^= static_cast<uint32_t>(i);
        
        keccak_512(mix, 64, mix);
        
        // Mix with cache lookups
        for (int j = 0; j < 256; j++) {
            uint32_t parent;
            std::memcpy(&parent, mix + (j % 16) * 4, 4);
            parent = fnv1a(static_cast<uint32_t>(i ^ j), parent) % cache_items;
            
            for (int k = 0; k < 64; k++) {
                mix[k] = fnv1a(mix[k], cache_data[parent * 64 + k]);
            }
        }
        
        keccak_512(mix, 64, data_.data() + i * 64);
        
        if (i % (num_items / 100 + 1) == 0) {
            LOG_DEBUG("DAG progress: {}%", i * 100 / num_items);
        }
    }
    
    LOG_INFO("DAG generated ({} GB)", dag_size / (1024 * 1024 * 1024));
}

DAG::~DAG() = default;

Result hash_light(
    const Hash256& header_hash,
    uint64_t nonce,
    uint64_t height,
    const Cache& cache
) {
    Result result{};
    
    // Compute mix seed
    uint8_t seed_input[40];
    std::memcpy(seed_input, header_hash.data(), 32);
    std::memcpy(seed_input + 32, &nonce, 8);
    
    Hash512 seed;
    keccak_512(seed_input, 40, seed.data());
    
    // Initialize mix
    uint32_t mix[PROGPOW_LANES][32];
    for (int lane = 0; lane < PROGPOW_LANES; lane++) {
        for (int i = 0; i < 32; i++) {
            uint32_t seed_val;
            std::memcpy(&seed_val, seed.data() + (i * 4) % 64, 4);
            mix[lane][i] = fnv1a(seed_val, lane ^ i);
        }
    }
    
    size_t dag_size = get_dag_size(cache.epoch());
    size_t num_dag_items = dag_size / 256;
    
    // Main ProgPoW loop
    for (int round = 0; round < PROGPOW_CNT_DAG; round++) {
        uint32_t prog_seed = fnv1a(seed[0], round);
        
        for (int lane = 0; lane < PROGPOW_LANES; lane++) {
            // DAG access (simulated from cache)
            uint32_t dag_idx = mix[lane][0] % num_dag_items;
            uint32_t cache_idx = (dag_idx * 4) % (cache.size() / 64);
            
            const uint8_t* dag_data = cache.data() + cache_idx * 64;
            
            for (int i = 0; i < 32; i++) {
                uint32_t dag_val;
                std::memcpy(&dag_val, dag_data + (i % 16) * 4, 4);
                mix[lane][i] = fnv1a(mix[lane][i], dag_val);
            }
        }
        
        // Math operations
        for (int m = 0; m < PROGPOW_CNT_MATH; m++) {
            for (int lane = 0; lane < PROGPOW_LANES; lane++) {
                uint32_t src1 = mix[lane][prog_seed % 32];
                uint32_t src2 = mix[lane][(prog_seed + 1) % 32];
                uint32_t dst_idx = (prog_seed + 2) % 32;
                mix[lane][dst_idx] = progpow_math(src1, src2, prog_seed);
            }
        }
    }
    
    // Final mix
    uint32_t digest[8] = {0};
    for (int lane = 0; lane < PROGPOW_LANES; lane++) {
        for (int i = 0; i < 8; i++) {
            digest[i] = fnv1a(digest[i], mix[lane][i]);
        }
    }
    
    // Convert to bytes
    for (int i = 0; i < 8; i++) {
        std::memcpy(result.mix_hash.data() + i * 4, &digest[i], 4);
    }
    
    // Final hash
    uint8_t final_input[96];
    std::memcpy(final_input, header_hash.data(), 32);
    std::memcpy(final_input + 32, &nonce, 8);
    std::memcpy(final_input + 40, result.mix_hash.data(), 32);
    // Pad to 96 bytes
    std::memset(final_input + 72, 0, 24);
    
    keccak_256(final_input, 72, result.final_hash.data());
    
    return result;
}

Result hash(
    const Hash256& header_hash,
    uint64_t nonce,
    uint64_t height,
    const DAG& dag
) {
    // Full DAG version - similar to hash_light but uses actual DAG
    // For now, delegate to light version
    Cache cache(dag.epoch());
    return hash_light(header_hash, nonce, height, cache);
}

bool verify(
    const Hash256& header_hash,
    uint64_t nonce,
    uint64_t height,
    const Hash256& mix_hash,
    const Hash256& final_hash,
    const Cache& cache
) {
    Result result = hash_light(header_hash, nonce, height, cache);
    return result.mix_hash == mix_hash && result.final_hash == final_hash;
}

Hash256 compute_header_hash(const BlockHeader& header) {
    // Serialize header
    uint8_t buffer[200];
    size_t offset = 0;
    
    std::memcpy(buffer + offset, &header.version, 4); offset += 4;
    std::memcpy(buffer + offset, header.parent_hash.data(), 32); offset += 32;
    std::memcpy(buffer + offset, header.merkle_root.data(), 32); offset += 32;
    std::memcpy(buffer + offset, header.state_root.data(), 32); offset += 32;
    std::memcpy(buffer + offset, &header.timestamp, 8); offset += 8;
    std::memcpy(buffer + offset, &header.difficulty, 8); offset += 8;
    std::memcpy(buffer + offset, &header.nonce, 8); offset += 8;
    std::memcpy(buffer + offset, &header.height, 8); offset += 8;
    std::memcpy(buffer + offset, &header.extra_nonce, 8); offset += 8;
    std::memcpy(buffer + offset, header.beneficiary.data(), 20); offset += 20;
    
    Hash256 hash;
    keccak_256(buffer, offset, hash.data());
    return hash;
}

bool meets_target(const Hash256& hash, uint64_t difficulty) {
    Hash256 target = difficulty_to_target(difficulty);
    return hash < target;
}

Hash256 difficulty_to_target(uint64_t difficulty) {
    Hash256 target;
    target.fill(0xff);
    
    if (difficulty == 0) return target;
    
    // target = max_target / difficulty
    // Simplified calculation
    uint64_t top = 0xFFFFFFFFFFFFFFFFULL / difficulty;
    std::memcpy(target.data(), &top, 8);
    std::memset(target.data() + 8, 0, 24);
    
    return target;
}

} // namespace kawpow
