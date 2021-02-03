//
// Created by squ1dd13 on 30/12/2020.
//

#pragma once

#include <fstream>
#include <memory>
#include <stdexcept>
#include <string>

enum class MessageType {
    Normal, Info, Error, Warning, Important
};

[[maybe_unused]] void SendBuf(void *data, size_t length);

#define DEBUG_LOGGING 1

#if DEBUG_LOGGING

template <typename... Args>
[[maybe_unused]] inline void Logf(MessageType messageType, const std::string &format, Args... args) {
    int size = snprintf(nullptr, 0, format.c_str(), args...) + 1; // Extra space for '\0'

    if (size <= 0) {
        throw std::runtime_error("Error during formatting.");
    }

    std::unique_ptr<char[]> buf(new char[size + 1]);
    snprintf(buf.get() + 1, size, format.c_str(), args...);

    buf.get()[0] = (unsigned char)messageType;

    // Send `size` bytes instead of `size + 1` because we don't want the
    //  null terminator in there.
    SendBuf(buf.get(), size);
}

#else
template <typename... Args>
[[maybe_unused]] inline void Logf(MessageType messageType, const std::string &format, Args... args) {}
#endif

#define Log(f, ...)          Logf(MessageType::Normal, f, ##__VA_ARGS__)
#define LogError(f, ...)     Logf(MessageType::Error, f, ##__VA_ARGS__)
#define LogInfo(f, ...)      Logf(MessageType::Info, f, ##__VA_ARGS__)
#define LogWarning(f, ...)   Logf(MessageType::Warning, f, ##__VA_ARGS__)
#define LogImportant(f, ...) Logf(MessageType::Important, f, ##__VA_ARGS__)