#include "Text.h"
#include <codecvt>
#include <fstream>
#include <iostream>
#include <locale>
#include <sstream>
#include <string>

// Strings that have been added/modified. This is searched before the
//  original game function is called, so replacements of existing strings
//  will show up.
static std::unordered_map<std::string, std::u16string> customStrings {};

const char *operator "" _gxt(const char *value, size_t length) {
    // Null-terminate.
    // TODO: Make GXT string literal more efficient.
    value = std::string(value, length).c_str();

    auto key = std::to_string(std::hash<std::string>()(value));
    return Text::registered(key, value);
}

std::string Text::registerString(const std::string &value) {
    auto key = std::to_string(std::hash<std::string>()(value));
    return Text::registered(key, value);
}

DeclareFunctionType(GetGameStringFunc, const char16 *, void *, const char *);
static void *textObject = Memory::slid<void *>(0x1008f5690);

std::string Text::forceASCII(const char *s) {
    std::stringstream stream;

    while (*s) {
        stream << *s;
        s += 2;
    }

    return stream.str();
}

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

const char *Text::registered(string_ref key, string_ref value) {
    auto iter = customStrings.find(key);
    if (iter == customStrings.end()) {
        setGameString(key, value);
        iter = customStrings.find(key);
    }

    // Return a pointer to the stored key rather than the passed one;
    //  the passed key is likely to be destroyed before getGameString
    //  is called with it. The stored key is unlikely to move/destruct
    //  in that period.
    return iter->first.c_str();
}

void skipLeadingSpaces(std::string &str) {
    if (str.empty())
        return;

    auto firstNotSpace = std::find_if_not(str.begin(), str.end(), ::isspace);
    if (firstNotSpace == str.end()) {
        str = "";
        return;
    }

    str = std::string(firstNotSpace, str.end());
}

void Text::loadFXT(string_ref path) {
    std::ifstream stream(path);

    for (std::string fxtLine; std::getline(stream, fxtLine);) {
        skipLeadingSpaces(fxtLine);

        // Remove end-of-line comments.
        fxtLine = fxtLine.substr(0, fxtLine.find("//")).substr(0, fxtLine.find('#'));

        // The string may now be empty, so check again.
        if (fxtLine.empty())
            continue;

        // <key> <value>
        // Split by the middle space.
        auto firstSpaceIter = std::find_if(fxtLine.begin(), fxtLine.end(), ::isspace);
        if (firstSpaceIter == fxtLine.end()) {
            // The game will crash later, so we don't need to worry about that now.
            Log::Print("error: FXT entry must have at least 1 separating space. (Line is '%s')", fxtLine.c_str());
            continue;
        }

        std::string valueStr(firstSpaceIter, fxtLine.end());
        skipLeadingSpaces(valueStr);

        if (valueStr.empty()) {
            Log::Print(
                "error: FXT value must not be empty. Set value will be '<empty>'. (Line is '%s')", fxtLine.c_str());
            valueStr = "<empty>";
        }

        std::string keyStr(fxtLine.begin(), firstSpaceIter);
        if (keyStr.empty()) {
            // This shouldn't actually happen.
            Log::Print("error: FXT key must not be empty. (Line is '%s')", fxtLine.c_str());
            continue;
        }

        valueStr = valueStr.substr(0, valueStr.find_first_of("\r\n"));

        setGameString(keyStr, valueStr);
    }
}

GetGameStringFunc originalGetGameString;

string16 getGameStringHook(void *self, const char *key) {
    if (!key || std::strlen(key) == 0) {
        return u"<EMPTY GXT KEY>";
    }

    auto it = customStrings.find(key);
    if (it != customStrings.end()) {
        return it->second.c_str();
    }

    auto ret = originalGetGameString(textObject, key);

    // Log::Print("'%s' --> '%s'", key, forceASCII((const char *)ret).c_str());

    return ret;
}

void Text::hook() {
    originalGetGameString = Memory::hook(0x10044142c, getGameStringHook);
}