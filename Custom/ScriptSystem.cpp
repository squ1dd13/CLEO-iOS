#include "ScriptSystem.hpp"
#include <dirent.h>
#include <stdio.h>
#include <string.h>
#include <sys/types.h>
#include <unistd.h>
#include "../Headers/Debug.hpp"

bool hasExtension(string_ref path, std::string extension) {
    // Efficiency: 100
    extension = "." + extension;

    if(extension.length() <= path.length()) {
        return !path.compare(path.length() - extension.length(), extension.length(), extension);
    }

    return false;
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
        
        if(hasExtension(entryPath, "csa")) {
            Debug::logf("Found .csa (CLEO Android script) %s", path.c_str());
        } else if(hasExtension(entryPath, "cs") || hasExtension(entryPath, "cs3") || hasExtension(entryPath, "cs4")) {
            Debug::logf("Found .cs or .cs* file. The script will be loaded, but will likely not run.");
        } else {
            continue;
        }

        paths.push_back(path + "/" + entryPath);
    }

    closedir(directory);
    return paths;
}

ScriptSystem::ScriptSystem(string_ref scriptDirectory, string_ref configDirectory) {
    scriptDir = scriptDirectory;
    configDir = configDirectory;
}

void ScriptSystem::loadScripts() {
    auto scriptPaths = findScripts(scriptDir);
    loadedScripts.reserve(scriptPaths.size());

    for(string_ref path : scriptPaths) {
        loadedScripts.push_back(GameScript::load(path));
    }
}

void ScriptSystem::advance() {
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

void ScriptSystem::release() {
    for(GameScript &script : loadedScripts) {
        script.release();
    }
}