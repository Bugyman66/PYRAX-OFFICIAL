#pragma once

#include <string>
#include <functional>
#include <thread>
#include <atomic>
#include <queue>
#include <mutex>
#include "../kawpow/kawpow.h"

namespace stratum {

struct Job {
    std::string job_id;
    kawpow::Hash256 header_hash;
    kawpow::Hash256 seed_hash;
    kawpow::Hash256 target;
    uint64_t height;
    uint64_t difficulty;
    bool clean;
};

struct Share {
    std::string job_id;
    uint64_t nonce;
    kawpow::Hash256 mix_hash;
    kawpow::Hash256 result;
};

class Client {
public:
    using JobCallback = std::function<void(const Job&)>;
    using AcceptedCallback = std::function<void(const Share&)>;
    using RejectedCallback = std::function<void(const Share&, const std::string&)>;
    using DisconnectCallback = std::function<void(const std::string&)>;

    Client();
    ~Client();

    // Connection
    bool connect(const std::string& host, int port);
    void disconnect();
    bool is_connected() const { return connected_; }

    // Authentication
    bool authorize(const std::string& user, const std::string& password);
    bool is_authorized() const { return authorized_; }

    // Mining
    void subscribe();
    void submit_share(const Share& share);

    // Callbacks
    void on_job(JobCallback callback) { job_callback_ = callback; }
    void on_accepted(AcceptedCallback callback) { accepted_callback_ = callback; }
    void on_rejected(RejectedCallback callback) { rejected_callback_ = callback; }
    void on_disconnect(DisconnectCallback callback) { disconnect_callback_ = callback; }

    // Stats
    uint64_t accepted_shares() const { return accepted_shares_; }
    uint64_t rejected_shares() const { return rejected_shares_; }
    double hashrate() const { return hashrate_; }
    void set_hashrate(double hr) { hashrate_ = hr; }

private:
    void receive_loop();
    void process_message(const std::string& message);
    void send(const std::string& message);

    // Connection state
    std::atomic<bool> connected_{false};
    std::atomic<bool> authorized_{false};
    int socket_ = -1;

    // Threading
    std::thread receive_thread_;
    std::atomic<bool> running_{false};

    // Message queue
    std::queue<std::string> send_queue_;
    std::mutex queue_mutex_;

    // Callbacks
    JobCallback job_callback_;
    AcceptedCallback accepted_callback_;
    RejectedCallback rejected_callback_;
    DisconnectCallback disconnect_callback_;

    // Stats
    std::atomic<uint64_t> accepted_shares_{0};
    std::atomic<uint64_t> rejected_shares_{0};
    std::atomic<double> hashrate_{0.0};

    // Session
    std::string session_id_;
    std::string extra_nonce_;
};

} // namespace stratum
