#pragma once

#include <string>
#include <iostream>
#include <sstream>
#include <mutex>
#include <ctime>
#include <iomanip>

class Logger {
public:
    enum class Level {
        TRACE = 0,
        DEBUG = 1,
        INFO = 2,
        WARN = 3,
        ERROR = 4,
        FATAL = 5
    };

    static void init(Level level = Level::INFO) {
        instance().level_ = level;
    }

    static Level level() {
        return instance().level_;
    }

    static void set_level(Level level) {
        instance().level_ = level;
    }

    template<typename... Args>
    static void log(Level level, const char* file, int line, const std::string& fmt, Args... args) {
        if (level < instance().level_) return;

        std::lock_guard<std::mutex> lock(instance().mutex_);

        auto now = std::time(nullptr);
        auto tm = *std::localtime(&now);

        std::cout << std::put_time(&tm, "%Y-%m-%d %H:%M:%S") << " ";
        std::cout << "[" << level_string(level) << "] ";
        
        print_formatted(fmt, args...);
        std::cout << std::endl;
    }

private:
    Level level_ = Level::INFO;
    std::mutex mutex_;

    static Logger& instance() {
        static Logger logger;
        return logger;
    }

    static const char* level_string(Level level) {
        switch (level) {
            case Level::TRACE: return "TRACE";
            case Level::DEBUG: return "DEBUG";
            case Level::INFO:  return "INFO ";
            case Level::WARN:  return "WARN ";
            case Level::ERROR: return "ERROR";
            case Level::FATAL: return "FATAL";
            default: return "?????";
        }
    }

    // Simple format string implementation
    static void print_formatted(const std::string& fmt) {
        std::cout << fmt;
    }

    template<typename T, typename... Args>
    static void print_formatted(const std::string& fmt, T value, Args... args) {
        size_t pos = fmt.find("{}");
        if (pos != std::string::npos) {
            std::cout << fmt.substr(0, pos) << value;
            print_formatted(fmt.substr(pos + 2), args...);
        } else {
            std::cout << fmt;
        }
    }
};

#define LOG_TRACE(...) Logger::log(Logger::Level::TRACE, __FILE__, __LINE__, __VA_ARGS__)
#define LOG_DEBUG(...) Logger::log(Logger::Level::DEBUG, __FILE__, __LINE__, __VA_ARGS__)
#define LOG_INFO(...)  Logger::log(Logger::Level::INFO,  __FILE__, __LINE__, __VA_ARGS__)
#define LOG_WARN(...)  Logger::log(Logger::Level::WARN,  __FILE__, __LINE__, __VA_ARGS__)
#define LOG_ERROR(...) Logger::log(Logger::Level::ERROR, __FILE__, __LINE__, __VA_ARGS__)
#define LOG_FATAL(...) Logger::log(Logger::Level::FATAL, __FILE__, __LINE__, __VA_ARGS__)
