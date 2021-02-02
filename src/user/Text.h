#ifndef TEXT_HEADER
#define TEXT_HEADER

#include "bridge/Memory.h"

#include <unordered_map>

namespace Text {

    // Custom.
    void setGameString(string_ref key, string_ref value);
    std::string registerString(string_ref value);

    // Registers the string and returns the key.
    const char *operator"" _gxt(const char *, size_t);

    // Get a string value by a key, registering the key with the given value if it isn't found.
    // The key is returned.
    const char *registered(string_ref key, string_ref value);

    // For CLEO FXT files.
    void LoadFxt(string_ref path);

    std::string forceASCII(const char *s);

    void hook();
};

#endif