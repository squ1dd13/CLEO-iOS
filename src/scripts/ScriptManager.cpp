//
// Created by squ1dd13 on 08/11/2020.
//

#include "ScriptManager.h"

#include "../shared/Addresses.h"
#include "../shared/Text.h"
#include "../shared/UserFolder.h"
#include "Script.h"
#include "new/Hook/NotLogos.h"

static std::vector<Script> startupScripts;
static std::vector<Script> invokedScripts;

void ScriptManager::Init() {
    std::vector<Directory::File> foundScripts;
    userFolder.FindAllOfType(Directory::FileType::AndroidRunningScript, foundScripts);

    for (auto &f : foundScripts) {
        LoadScript(f.fullPath);
    }

    std::vector<Directory::File> foundTextFiles;
    userFolder.FindAllOfType(Directory::FileType::TextExtension, foundTextFiles);

    for (auto &f : foundTextFiles) {
        Text::loadFXT(f.fullPath);
    }
}

void ScriptManager::LoadScript(const std::string &path) {
    if (path.ends_with("csi")) {
        // Invoked script, so don't launch it.
        invokedScripts.push_back(std::move(Script(path)));
    } else {
        startupScripts.push_back(std::move(Script(path)));
    }
}

void ScriptManager::UnloadAll() {
    /* nop */
}

uint32 ScriptManager::GetScriptTime() {
    return Memory::fetch<uint32>(Memory::Addresses::scriptTime);
}

void ScriptManager::AdvanceScripts() {
    for (auto &script : startupScripts) {
        // The script's activation time is the next time it will get focus.
        // wait(n) for any n != 0 offsets the activation time by n and returns 1
        //  to stop the current execution cycle. If n == 0, wait() returns zero
        //  and execution continues.

        if (script.activationTime <= GetScriptTime()) {
            script.RunNextBlock();
        }
    }
}

functionhook ScriptUpdate {
    void Original();

    void Body() {
        // Every time the game advances its scripts, we advance our own.
        ScriptManager::AdvanceScripts();

        (*Memory::slid<float *>(0x1007d3b18)) = 0.3f;

        // The game scripts still need to run, so call the original implementation.
        Original();
    }

    HookSave(0x1001d0f40)
}