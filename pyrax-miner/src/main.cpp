#include <iostream>
#include <string>
#include <thread>
#include <atomic>
#include <csignal>
#include <cstring>

#include "kawpow/kawpow.h"
#include "stratum/client.h"
#include "utils/logger.h"

// Global shutdown flag
std::atomic<bool> g_shutdown{false};

void signal_handler(int signum) {
    LOG_INFO("Received signal {}, shutting down...", signum);
    g_shutdown = true;
}

struct MinerConfig {
    std::string pool_url;
    std::string wallet_address;
    std::string worker_name;
    std::string password;
    bool solo_mode;
    std::string node_url;
    int cuda_device;
    int opencl_device;
    bool cpu_mining;
    int threads;
};

void print_usage() {
    std::cout << R"(
PYRAX Miner - KAWPOW GPU/CPU Miner
Usage: pyrax-miner [OPTIONS]

Pool Mining:
  -o, --pool <url>          Pool URL (stratum+tcp://...)
  -u, --user <address>      Wallet address
  -w, --worker <name>       Worker name (default: default)
  -p, --password <pass>     Pool password (default: x)

Solo Mining:
  --solo                    Enable solo mining mode
  --node <url>              Node RPC URL (default: http://127.0.0.1:8545)

Device Selection:
  --cuda <id>               CUDA device ID (-1 to disable)
  --opencl <id>             OpenCL device ID (-1 to disable)
  --cpu                     Enable CPU mining
  --threads <n>             CPU threads (default: auto)

Other:
  -h, --help                Show this help
  -v, --version             Show version

Examples:
  pyrax-miner -o stratum+tcp://pool.pyrax.org:3333 -u 0x... -w rig1
  pyrax-miner --solo --node http://127.0.0.1:8545 -u 0x...
  pyrax-miner --cpu --threads 4 --solo -u 0x...
)";
}

MinerConfig parse_args(int argc, char* argv[]) {
    MinerConfig config;
    config.pool_url = "";
    config.wallet_address = "";
    config.worker_name = "default";
    config.password = "x";
    config.solo_mode = false;
    config.node_url = "http://127.0.0.1:8545";
    config.cuda_device = 0;
    config.opencl_device = -1;
    config.cpu_mining = false;
    config.threads = std::thread::hardware_concurrency();

    for (int i = 1; i < argc; i++) {
        std::string arg = argv[i];
        
        if (arg == "-h" || arg == "--help") {
            print_usage();
            exit(0);
        } else if (arg == "-v" || arg == "--version") {
            std::cout << "pyrax-miner version 0.1.0" << std::endl;
            exit(0);
        } else if ((arg == "-o" || arg == "--pool") && i + 1 < argc) {
            config.pool_url = argv[++i];
        } else if ((arg == "-u" || arg == "--user") && i + 1 < argc) {
            config.wallet_address = argv[++i];
        } else if ((arg == "-w" || arg == "--worker") && i + 1 < argc) {
            config.worker_name = argv[++i];
        } else if ((arg == "-p" || arg == "--password") && i + 1 < argc) {
            config.password = argv[++i];
        } else if (arg == "--solo") {
            config.solo_mode = true;
        } else if (arg == "--node" && i + 1 < argc) {
            config.node_url = argv[++i];
        } else if (arg == "--cuda" && i + 1 < argc) {
            config.cuda_device = std::stoi(argv[++i]);
        } else if (arg == "--opencl" && i + 1 < argc) {
            config.opencl_device = std::stoi(argv[++i]);
        } else if (arg == "--cpu") {
            config.cpu_mining = true;
        } else if (arg == "--threads" && i + 1 < argc) {
            config.threads = std::stoi(argv[++i]);
        }
    }

    return config;
}

int main(int argc, char* argv[]) {
    // Initialize logger
    Logger::init(Logger::Level::INFO);
    
    LOG_INFO("PYRAX Miner v0.1.0");
    LOG_INFO("KAWPOW GPU/CPU Mining Software");
    
    // Parse arguments
    MinerConfig config = parse_args(argc, argv);
    
    // Validate config
    if (config.wallet_address.empty()) {
        LOG_ERROR("Wallet address required (-u)");
        return 1;
    }
    
    if (!config.solo_mode && config.pool_url.empty()) {
        LOG_ERROR("Pool URL required (-o) or use --solo for solo mining");
        return 1;
    }
    
    // Setup signal handlers
    signal(SIGINT, signal_handler);
    signal(SIGTERM, signal_handler);
    
    LOG_INFO("Wallet: {}", config.wallet_address);
    
    if (config.solo_mode) {
        LOG_INFO("Mode: Solo mining");
        LOG_INFO("Node: {}", config.node_url);
    } else {
        LOG_INFO("Mode: Pool mining");
        LOG_INFO("Pool: {}", config.pool_url);
        LOG_INFO("Worker: {}", config.worker_name);
    }
    
    // Initialize KAWPOW
    kawpow::init();
    
    // Initialize mining devices
    std::vector<kawpow::MiningDevice*> devices;
    
    if (config.cuda_device >= 0) {
        LOG_INFO("Initializing CUDA device {}...", config.cuda_device);
        auto* cuda_device = kawpow::create_cuda_device(config.cuda_device);
        if (cuda_device) {
            devices.push_back(cuda_device);
            LOG_INFO("CUDA device initialized: {}", cuda_device->get_name());
        } else {
            LOG_WARN("Failed to initialize CUDA device {}", config.cuda_device);
        }
    }
    
    if (config.opencl_device >= 0) {
        LOG_INFO("Initializing OpenCL device {}...", config.opencl_device);
        auto* opencl_device = kawpow::create_opencl_device(config.opencl_device);
        if (opencl_device) {
            devices.push_back(opencl_device);
            LOG_INFO("OpenCL device initialized: {}", opencl_device->get_name());
        } else {
            LOG_WARN("Failed to initialize OpenCL device {}", config.opencl_device);
        }
    }
    
    if (config.cpu_mining) {
        LOG_INFO("Initializing CPU mining with {} threads...", config.threads);
        auto* cpu_device = kawpow::create_cpu_device(config.threads);
        if (cpu_device) {
            devices.push_back(cpu_device);
            LOG_INFO("CPU mining initialized");
        }
    }
    
    if (devices.empty()) {
        LOG_ERROR("No mining devices available!");
        return 1;
    }
    
    // Connect to pool or node
    std::unique_ptr<stratum::Client> stratum_client;
    std::unique_ptr<kawpow::SoloMiner> solo_miner;
    
    kawpow::MiningJob current_job;
    std::atomic<bool> has_job{false};
    std::mutex job_mutex;
    
    if (config.solo_mode) {
        LOG_INFO("Connecting to node: {}", config.node_url);
        solo_miner = std::make_unique<kawpow::SoloMiner>(config.node_url, config.wallet_address);
        if (!solo_miner->connect()) {
            LOG_ERROR("Failed to connect to node");
            return 1;
        }
        LOG_INFO("Connected to node");
    } else {
        LOG_INFO("Connecting to pool: {}", config.pool_url);
        stratum_client = std::make_unique<stratum::Client>();
        
        stratum_client->set_job_callback([&](const kawpow::MiningJob& job) {
            std::lock_guard<std::mutex> lock(job_mutex);
            current_job = job;
            has_job = true;
            LOG_INFO("New job received: height={}, difficulty={}", job.height, job.difficulty);
        });
        
        stratum_client->set_accepted_callback([&](uint64_t nonce) {
            LOG_INFO("Share accepted! Nonce: 0x{:016x}", nonce);
        });
        
        stratum_client->set_rejected_callback([&](uint64_t nonce, const std::string& reason) {
            LOG_WARN("Share rejected: {} (nonce: 0x{:016x})", reason, nonce);
        });
        
        if (!stratum_client->connect(config.pool_url, config.wallet_address, config.worker_name, config.password)) {
            LOG_ERROR("Failed to connect to pool");
            return 1;
        }
        LOG_INFO("Connected to pool");
    }
    
    // Start mining threads for each device
    std::vector<std::thread> mining_threads;
    std::atomic<uint64_t> total_hashes{0};
    std::atomic<uint64_t> shares_accepted{0};
    std::atomic<uint64_t> shares_rejected{0};
    
    for (auto* device : devices) {
        mining_threads.emplace_back([&, device]() {
            LOG_INFO("Mining thread started for {}", device->get_name());
            
            uint64_t local_hashes = 0;
            auto last_report = std::chrono::steady_clock::now();
            
            while (!g_shutdown) {
                // Wait for job
                if (!has_job) {
                    std::this_thread::sleep_for(std::chrono::milliseconds(100));
                    continue;
                }
                
                // Copy current job
                kawpow::MiningJob job;
                {
                    std::lock_guard<std::mutex> lock(job_mutex);
                    job = current_job;
                }
                
                // Mine a batch
                kawpow::MiningResult result;
                uint64_t batch_hashes = device->mine_batch(job, result);
                local_hashes += batch_hashes;
                total_hashes.fetch_add(batch_hashes);
                
                // Check if we found a solution
                if (result.found) {
                    LOG_INFO("Solution found! Nonce: 0x{:016x}, Mix: 0x{}", 
                        result.nonce, kawpow::to_hex(result.mix_hash));
                    
                    // Submit result
                    if (config.solo_mode) {
                        if (solo_miner->submit_block(job, result)) {
                            LOG_INFO("Block submitted successfully!");
                            shares_accepted++;
                        } else {
                            LOG_WARN("Block submission failed");
                            shares_rejected++;
                        }
                    } else {
                        stratum_client->submit_share(job.job_id, result.nonce, result.mix_hash);
                    }
                }
                
                // Report hashrate periodically
                auto now = std::chrono::steady_clock::now();
                auto elapsed = std::chrono::duration_cast<std::chrono::seconds>(now - last_report).count();
                if (elapsed >= 10) {
                    double hashrate = local_hashes / static_cast<double>(elapsed);
                    LOG_INFO("{}: {:.2f} MH/s", device->get_name(), hashrate / 1e6);
                    local_hashes = 0;
                    last_report = now;
                }
            }
            
            LOG_INFO("Mining thread stopped for {}", device->get_name());
        });
    }
    
    LOG_INFO("Starting mining...");
    
    // Hashrate reporting thread
    std::thread stats_thread([&]() {
        auto last_hashes = total_hashes.load();
        auto last_time = std::chrono::steady_clock::now();
        
        while (!g_shutdown) {
            std::this_thread::sleep_for(std::chrono::seconds(30));
            
            auto now = std::chrono::steady_clock::now();
            auto elapsed = std::chrono::duration_cast<std::chrono::seconds>(now - last_time).count();
            auto current_hashes = total_hashes.load();
            
            if (elapsed > 0) {
                double hashrate = (current_hashes - last_hashes) / static_cast<double>(elapsed);
                LOG_INFO("Total hashrate: {:.2f} MH/s | Accepted: {} | Rejected: {}", 
                    hashrate / 1e6, shares_accepted.load(), shares_rejected.load());
            }
            
            last_hashes = current_hashes;
            last_time = now;
        }
    });
    
    // Job update thread (for solo mining)
    std::thread job_thread([&]() {
        while (!g_shutdown) {
            if (config.solo_mode && solo_miner) {
                auto new_job = solo_miner->get_work();
                if (new_job.has_value()) {
                    std::lock_guard<std::mutex> lock(job_mutex);
                    if (current_job.height != new_job->height) {
                        current_job = *new_job;
                        has_job = true;
                        LOG_INFO("New job: height={}, difficulty={}", new_job->height, new_job->difficulty);
                    }
                }
            }
            std::this_thread::sleep_for(std::chrono::milliseconds(500));
        }
    });
    
    // Main loop - wait for shutdown
    while (!g_shutdown) {
        std::this_thread::sleep_for(std::chrono::seconds(1));
    }
    
    LOG_INFO("Shutting down...");
    
    // Wait for threads to finish
    for (auto& t : mining_threads) {
        if (t.joinable()) t.join();
    }
    if (stats_thread.joinable()) stats_thread.join();
    if (job_thread.joinable()) job_thread.join();
    
    // Cleanup devices
    for (auto* device : devices) {
        delete device;
    }
    
    // Disconnect
    if (stratum_client) stratum_client->disconnect();
    if (solo_miner) solo_miner->disconnect();
    
    kawpow::cleanup();
    
    return 0;
}
