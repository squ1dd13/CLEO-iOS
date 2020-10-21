#include "Custom/Scripts.hpp"
#include "Util/Debug.hpp"
#include <cctype>
#include <algorithm>
#include <dirent.h>
#include "Game/Text.hpp"

std::vector<GameScript> Scripts::loadedScripts;
std::vector<std::string> Scripts::fileNames;
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
        screenLog.logf("Failed to find script directory");
        return {};
    }

    screenLog.logf("Successfully found script directory");

    dirent *entry;

    std::vector<std::string> paths;
    while((entry = readdir(directory)) != nullptr) {
        std::string entryPath = entry->d_name;
        FileType type = getFileType(entryPath);

        switch(type) {
            case AndroidScript: {
                Debug::logf("Found Android script '%s'", entry->d_name);
                screenLog.logf("Found Android script '%s'", entry->d_name);
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
        fileNames.push_back(path);
        Debug::logf("load %s", path.c_str());
        loadedScripts.push_back(GameScript::load(path));
    }

    Debug::logf("%d script(s) loaded", loadedScripts.size());
}

void Scripts::advance() {
    static bool didPrint = false;
    if(!didPrint) {
        screenLog.logf("BEGIN SCRIPT LIST");
        for(auto &path : fileNames) {
            screenLog.logf("  %s", path.c_str());
        }
        screenLog.logf("END SCRIPT LIST");
        didPrint = true;
    }

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
        script.free();
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

        // NOTE: Directory has changed.
        Scripts::load("/var/mobile/Documents/CS", "");
    }

    // Advance custom scripts.
    Scripts::advance();

    // Advance game scripts.
    advanceGameScripts();
}

void Scripts::hook() {
    advanceGameScripts = Memory::hook(0x1001d0f40, advanceScripts);
}