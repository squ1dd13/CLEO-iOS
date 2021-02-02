//
// Created by squ1dd13 on 08/11/2020.
//

#include "Manager.h"

#include "Script.h"
#include "user/Directory.h"
#include "user/Text.h"
#include "bridge/Addresses.h"
#include "hook/Func.h"

#include <list>
#include <map>

class InvokedScript {
    std::string path;

public:
    std::string menuName;

    explicit InvokedScript(std::string p) {
        path = std::move(p);

        menuName = path;

        size_t lastSeparator = menuName.find_last_of("\\/");
        if (lastSeparator != std::string::npos) {
            menuName.erase(0, lastSeparator + 1);
        }

        size_t extensionBegin = menuName.rfind('.');
        if (extensionBegin != std::string::npos) {
            menuName.erase(extensionBegin);
        }
    }

    Scripts::Script Load() const {
        return Scripts::Script(path);
    }
};

static std::list<Scripts::Script> running;
static std::map<std::string, InvokedScript> invokedScripts;
static std::set<std::string> invokedNames;

static bool initialized = false;

void Scripts::Manager::Init() {
    std::vector<Directory::File> foundFiles;

    userFolder.FindAllOfType(Directory::FileType::AndroidRunningScript, foundFiles);
    userFolder.FindAllOfType(Directory::FileType::AndroidInvokedScript, foundFiles);

    for (auto &f : foundFiles) {
        LoadScript(f.fullPath);
    }

    foundFiles.clear();

    userFolder.FindAllOfType(Directory::FileType::TextExtension, foundFiles);

    for (auto &f : foundFiles) {
        Text::LoadFxt(f.fullPath);
    }

    initialized = true;
}

void Scripts::Manager::LoadScript(const std::string &path) {
    if (path.ends_with("csi")) {
        InvokedScript script(path);
        invokedNames.insert(script.menuName);
        invokedScripts.emplace(script.menuName, std::move(script));
    } else {
        running.push_back(std::move(Script(path)));
        running.back().active = true;
    }
}

uint32 Scripts::Manager::GetScriptTime() {
    return Memory::Fetch<uint32>(Memory::Addresses::scriptTime);
}

void Scripts::Manager::AdvanceScripts() {
    for (auto scriptIter = running.begin(); scriptIter != running.end();) {
        // Check for inactive scripts and remove them.
        if (!scriptIter->active) {
            scriptIter = running.erase(scriptIter);
            continue;
        }

        // The script's activation time is the next time it will get focus.
        // wait(n) for any n != 0 offsets the activation time by n and returns 1
        //  to stop the current execution cycle. If n == 0, wait() returns zero
        //  and execution continues.
        if (scriptIter->activationTime <= GetScriptTime()) {
            scriptIter->RunNextBlock();
        }

        ++scriptIter;
    }
}

bool Scripts::Manager::Initialized() {
    return initialized;
}

void Scripts::Manager::Invoke(string_ref name) {
    Script loaded = invokedScripts.at(name).Load();
    loaded.active = true;

    // Add the invoked script to the list of running scripts. It will
    //  be automatically removed when it becomes inactive.
    running.push_back(std::move(loaded));
}

std::set<std::string> &Scripts::Manager::InvokedScripts() {
    return invokedNames;
}

functionhook ScriptUpdate {
    void Original();

    void Body() {
        // Every time the game advances its scripts, we advance our own.
        Scripts::Manager::AdvanceScripts();

        // The game scripts still need to run, so call the original implementation.
        Original();
    }

    HookSave(0x1001d0f40)
}