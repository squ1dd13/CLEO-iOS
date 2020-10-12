#ifndef TEXT_HEADER
#define TEXT_HEADER

#include "Memory.hpp"
#include <unordered_map>

// Registers the string and returns the key.
const char *operator "" _gxt(const char *, size_t);

namespace Text {

// Wrappers for game code.
std::string getGameString(string_ref key);
std::u16string getGameStringUTF16(string_ref key);

// Custom.
void setGameString(string_ref key, string_ref value);
void setGameStringUTF16(string_ref key, const std::u16string &value);

// Get a string value by a key, registering the key with the given value if it isn't found.
// The key is returned.
const char *registered(string_ref key, string_ref value);

// For CLEO FXT files.
void loadFXT(string_ref path);

void hook();

std::string registerString(const std::string& value);
};

#endif