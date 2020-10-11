#include "Custom/Scripts.hpp"
#include "Util/Debug.hpp"
#include <ctype.h>
#include <algorithm>
#include <dirent.h>
#include <stdio.h>
#include <string.h>
#include <sys/types.h>
#include <unistd.h>
#include "Game/Text.hpp"

static std::vector<GameScript> loadedScripts;
static std::string scriptDir, configDir;

enum FileType {
    AndroidScript, // .csa
    WindowsScript, // .cs
    TextExtension, // .fxt

    OtherFile
};

inline FileType getFileType(string_ref name) {
    std::string extension = name.substr(name.find_last_of('.') + 1);
    std::transform(extension.begin(), extension.end(), extension.begin(), ::tolower);

    if(extension == "csa") {
        return AndroidScript;
    }

    if(extension == "cs") {
        return WindowsScript;
    }

    if(extension == "fxt") {
        return TextExtension;
    }

    // Enum system used so more types may go here...

    return OtherFile;
}

std::vector<std::string> findScripts(string_ref path) {
    DIR *directory = opendir(path.c_str());

    if(!directory) {
        return {};
    }

    dirent *entry;

    std::vector<std::string> paths;
    while((entry = readdir(directory)) != nullptr) {
        std::string entryPath = entry->d_name;
        FileType type = getFileType(entryPath);

        switch(type) {
            case AndroidScript: {
                Debug::logf("Found Android script '%s'", entry->d_name);
                break;
            }

            case WindowsScript: {
                // It would be cool if we could port scripts after loading them...
                Debug::logf("Found Windows script (!!) '%s'... Expect a crash", entry->d_name);
                break;
            }

            case TextExtension: {
                Debug::logf("Found text extension file '%s'", entry->d_name);
                Text::loadFXT(path + "/" + entryPath);

                continue;
            }

            default: {
                Debug::logf("Ignoring '%s'", entry->d_name);
                continue;
            }
        }

        // If we're at this point, this is a script.
        paths.push_back(path + "/" + entryPath);
    }

    closedir(directory);
    return paths;
}

void Scripts::load(string_ref scriptDirectory, string_ref configDirectory) {
    scriptDir = scriptDirectory;
    configDir = configDirectory;

    auto scriptPaths = findScripts(scriptDir);
    loadedScripts.reserve(scriptPaths.size());

    for(string_ref path : scriptPaths) {
        loadedScripts.push_back(GameScript::load(path));
    }
}

void Scripts::advance() {
    for(GameScript &script : loadedScripts) {
        // The script's activation time is the next time it will get focus.
        // wait(n) for any n != 0 offsets the activation time by n and returns 1
        //  to stop the current execution cycle. When n == 0, wait() returns zero
        //  and execution continues.

        if(script.activationTime <= GameScript::time()) {
            script.executeBlock();
        }
    }
}

void Scripts::release() {
    for(GameScript &script : loadedScripts) {
        script.release();
    }
}

DeclareFunctionType(AdvanceFunction, void);
static AdvanceFunction advanceGameScripts;

void advanceScripts() {
    // We want to load the Scripts only when the game is ready. Therefore,
    //  loading it on the first advanceScripts call makes sense.
    static bool systemLoaded = false;
    if(!systemLoaded) {
        systemLoaded = true;
        Scripts::load("/var/mobile/Media/Documents/CustomScripts", "");
    }

    // Advance custom scripts.
    Scripts::advance();

    // Advance game scripts.
    advanceGameScripts();
}

void Scripts::hook() {
    advanceGameScripts = Memory::hook(0x1001d0f40, advanceScripts);
}