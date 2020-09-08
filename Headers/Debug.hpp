#include <memory>
#include <stdexcept>
#include <string>
#include <vector>
#include <os/log.h>

#ifndef DEBUG_HEADER
#define DEBUG_HEADER

// #define SHOW_DEBUG_OVERLAY

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