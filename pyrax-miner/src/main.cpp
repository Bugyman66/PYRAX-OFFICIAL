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
    
    // TODO: Initialize mining devices
    // TODO: Connect to pool or node
    // TODO: Start mining loops
    
    LOG_INFO("Starting mining...");
    
    // Main loop
    while (!g_shutdown) {
        std::this_thread::sleep_for(std::chrono::seconds(1));
        
        // TODO: Report hashrate
        // TODO: Handle job updates
        // TODO: Submit shares/blocks
    }
    
    LOG_INFO("Shutting down...");
    kawpow::cleanup();
    
    return 0;
}
