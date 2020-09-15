#include "Game/Text.hpp"
#include <codecvt>
#include <fstream>
#include <iostream>
#include <locale>
#include <sstream>
#include <string>

DeclareFunctionType(GetGameStringFunc, const char16 *, void *, const char *);
static void *textObject = Memory::slid<void *>(0x1008f5690);

std::string forceASCII(const char *s) {
    std::stringstream stream;

    while(*s) {
        stream << *s;
        s += 2;
    }

    return stream.str();
}

// Strings that have been added/modified. This is searched before the
//  original game function is called, so replacements of existing strings
//  will show up.
static std::unordered_map<std::string, std::u16string> customStrings {
    { "tweak_name", u"CSiOS" }
};

std::string Text::getGameString(string_ref key) {
    // This will mess up Japanese/Russian text, but the UTF16 version should
    //  be used if that's a concern.
    return forceASCII((const char *)(getGameStringUTF16(key).data()));
}

std::u16string Text::getGameStringUTF16(string_ref key) {
    return Memory::slid<GetGameStringFunc>(0x10044142c)(textObject, key.data());
}

void Text::setGameString(string_ref key, string_ref value) {
    static std::wstring_convert<std::codecvt_utf8_utf16<char16>, char16> converter;

    std::u16string converted = converter.from_bytes(value);
    customStrings[key] = converted;
}

void Text::setGameStringUTF16(string_ref key, const std::u16string &value) {
    customStrings[key] = value;
}

void skipLeadingSpaces(std::string &str) {
    if(str.empty()) return;

    auto firstNotSpace = std::find_if_not(str.begin(), str.end(), ::isspace);
    if(firstNotSpace == str.end()) {
        str = "";
        return;
    }

    str = std::string(firstNotSpace, str.end());
}

void Text::loadFXT(string_ref path) {
    std::ifstream stream(path);

    for(std::string fxtLine; std::getline(stream, fxtLine);) {
        skipLeadingSpaces(fxtLine);

        // Remove end-of-line comments.
        fxtLine = fxtLine.substr(0, fxtLine.find("//")).substr(0, fxtLine.find('#'));

        // The string may now be empty, so check again.
        if(fxtLine.empty()) continue;

        // <key> <value>
        // Split by the middle space.
        auto firstSpaceIter = std::find_if(fxtLine.begin(), fxtLine.end(), ::isspace);
        if(firstSpaceIter == fxtLine.end()) {
            // The game will crash later, so we don't need to worry about that now.
            Debug::logf("error: FXT entry must have at least 1 separating space. (Line is '%s')", fxtLine.c_str());
            continue;
        }

        std::string valueStr(firstSpaceIter, fxtLine.end());
        skipLeadingSpaces(valueStr);

        if(valueStr.empty()) {
            Debug::logf("error: FXT value must not be empty. Set value will be '<empty>'. (Line is '%s')", fxtLine.c_str());
            valueStr = "<empty>";
        }

        std::string keyStr(fxtLine.begin(), firstSpaceIter);
        if(keyStr.empty()) {
            // This shouldn't actually happen.
            Debug::logf("error: FXT key must not be empty. (Line is '%s')", fxtLine.c_str());
            continue;
        }

        valueStr = valueStr.substr(0, valueStr.find_first_of("\r\n"));

        Debug::logf("(FXT) '%s' -> '%s'", keyStr.c_str(), valueStr.c_str());
        setGameString(keyStr, valueStr);
    }
}

GetGameStringFunc originalGetGameString;

string16 getGameStringHook(void *self, const char *key) {
    auto it = customStrings.find(key);
    if(it != customStrings.end()) {
        return it->second.c_str();
    }

    return originalGetGameString(textObject, key);
}

void Text::hook() {
    originalGetGameString = Memory::hook(0x10044142c, getGameStringHook);
}