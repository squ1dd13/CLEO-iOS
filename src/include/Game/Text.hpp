#ifndef TEXT_HEADER
#define TEXT_HEADER

#include "Memory.hpp"
#include <unordered_map>

namespace Text {

// Wrappers for game code.
std::string getGameString(string_ref key);
std::u16string getGameStringUTF16(string_ref key);

// Custom.
void setGameString(string_ref key, string_ref value);
void setGameStringUTF16(string_ref key, const std::u16string &value);

// For CLEO FXT files.
void loadFXT(string_ref path);

void hook();

};

#endif