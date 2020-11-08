#include <memory>
#include <stdexcept>
#include <string>
#include <vector>
#include <os/log.h>
#include <queue>
#import <sstream>
#import <fstream>
#import <cstdio>

#ifndef DEBUG_HEADER
#define DEBUG_HEADER

#define SHOW_DEBUG_OVERLAY

class ScreenLog {
    static const size_t messageLimit = 30;
    std::ofstream stream;

public:
    std::deque<std::string> log;
    bool updated = false;

    ScreenLog() {
        // Open the log file, clearing it if it isn't empty.
        stream = std::ofstream("/var/mobile/Documents/CSiOS.log", std::ofstream::out | std::ofstream::trunc);
    }

    inline void commit(std::string s) {
        if(log.size() >= messageLimit) {
            // Remove the oldest message to make room for the newest.
            log.pop_front();
        }

        stream << s << '\n';
        stream.flush();
        log.push_back(s);
        updated = true;
    }

    template <typename... Args>
    inline void logf(const std::string &format, Args... args) {
        // https://stackoverflow.com/a/26221725/8622854
        size_t size = (size_t)std::snprintf(nullptr, 0, format.c_str(), args...) + 1;

        if(size <= 0) {
            throw std::runtime_error("Formatting error.");
        }

        char *buf = new char[size];
        snprintf(buf, size, format.c_str(), args...);

        commit(std::string(buf, buf + size - 1));
        delete[] buf;
    }
};

extern ScreenLog screenLog;

struct Debug {
    static std::vector<std::string> logStrings;

    template <typename... Args>
    static inline void logf(const std::string &format, Args... args) {
        // https://stackoverflow.com/a/26221725/8622854
        size_t size = snprintf(nullptr, 0, format.c_str(), args...) + 1;
        
        if(size <= 0) {
            throw std::runtime_error("Formatting error.");
        }

        std::unique_ptr<char[]> buf(new char[size]);
        snprintf(buf.get(), size, format.c_str(), args...);

#ifdef SHOW_DEBUG_OVERLAY
        logStrings.emplace_back(buf.get(), buf.get() + size - 1);
#endif

        os_log(OS_LOG_DEFAULT, "[CSiOS] %{public}s", std::string(buf.get(), buf.get() + size - 1).c_str());
    }

    template <typename... Args>
    static inline void assertf(bool condition, const std::string &format, Args... args) {
        if(!condition) {
            logf("err: " + format, args...);
        }
    }

    static inline bool needsUpdate() {
        return !logStrings.empty();
    }
};

#endif