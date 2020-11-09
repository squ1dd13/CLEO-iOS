#pragma once

#include <memory>
#include <stdexcept>
#include <string>
#include <vector>
#include <os/log.h>
#include <queue>
#import <sstream>
#import <fstream>
#import <cstdio>

#define LOG_OVERLAY

class Log {
    static const size_t messageLimit = 30;

public:
    static std::deque<std::string> log;
    static bool updated;

    static void Commit(const std::string &s);

    template <typename... Args>
    static void Print(const std::string &format, Args... args) {
        // https://stackoverflow.com/a/26221725/8622854
        size_t size = (size_t)std::snprintf(nullptr, 0, format.c_str(), args...) + 1;

        if(size <= 0) {
            throw std::runtime_error("Formatting error.");
        }

        char *buf = new char[size];
        snprintf(buf, size, format.c_str(), args...);

        Commit(std::string(buf, buf + size - 1));
        delete[] buf;
    }
};