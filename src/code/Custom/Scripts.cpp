#include "Custom/Scripts.hpp"
#include "Util/Debug.hpp"
#include <cctype>
#include <algorithm>
#include <dirent.h>
#include "Game/Text.hpp"
#include <Game/Addresses.hpp>

std::vector<GameScript> Scripts::loadedScripts;
std::vector<std::string> Scripts::fileNames;
static std::string scriptDir, configDir;

namespace Scripts {

class ScriptFile {
    inline static std::string getExtension(string_ref path) {
        auto dotPos = path.find_last_of('.');
        if(dotPos == std::string::npos) {
            return "";
        }

        std::string ext = path.substr(dotPos + 1);
        std::transform(ext.begin(), ext.end(), ext.begin(), ::tolower);

        return ext;
    }

    GameScript loadAsScript() const {
        return GameScript::load(path);
    }

    void loadSupporting() const {
        if(type == FileType::TextExtension) {
            Text::loadFXT(path);
        }
    }
public:
    // TODO: Put extensions in a map.
    enum class FileType {
        AndroidRunningScript, // .csa
        AndroidInvokedScript, // .csi
        WindowsScript,        // .cs
        TextExtension,        // .fxt

        OtherFile
    };

    std::string path;
    FileType type = FileType::OtherFile;
    bool isSupportingFile;

    explicit ScriptFile(std::string name) {
        path = std::move(name);

        std::string ext = getExtension(path);

        if(ext.starts_with("cs")) {
            isSupportingFile = false;

            if(ext == "csa") {
                // Android script. Starts when the game starts.
                type = FileType::AndroidRunningScript;
            } else if(ext == "csi") {
                // Android script. Started manually by the player.
                type = FileType::AndroidInvokedScript;
            } else if(ext == "cs") {
                // Windows script. *Very* unlikely to work.
                type = FileType::WindowsScript;
            }
        } else if(ext == "fxt") {
            isSupportingFile = true;

            // Only other file type supported currently is .fxt.
            type = FileType::TextExtension;
        }

        if(type == FileType::OtherFile) {
            return;
        }
    }

    static bool loadModDir(string_ref dirPath, std::vector<GameScript> &outScripts) {
        DIR *directory = opendir(dirPath.c_str());

        if(!directory) {
            return false;
        }

        dirent *entry;
        while((entry = readdir(directory))) {
            std::string entryPath = dirPath + '/' + entry->d_name;

            if(entry->d_type == DT_DIR) {
                std::string name(entry->d_name);
                if(name == ".." || name == ".") continue;

                // Found another directory, so search it.
                if(!loadModDir(entryPath, outScripts)) {
                    screenLog.logf("Failed to open dir '%s'.", entryPath.c_str());
                }

                continue;
            }

            ScriptFile file(entryPath);

            if(file.type == FileType::OtherFile) {
                screenLog.logf("Ignoring file '%s' (unknown/unsupported type)", file.path.c_str());
                continue;
            }

            if(!file.isSupportingFile) {
                outScripts.push_back(file.loadAsScript());
            } else {
                // We only collect the scripts we create. Supporting files
                //  are loaded as we discover them.
                file.loadSupporting();
            }
        }

        return true;
    }
};

void load(string_ref scriptDirectory, string_ref configDirectory) {
    scriptDir = scriptDirectory;
    configDir = configDirectory;

    bool didOpenScriptDir = ScriptFile::loadModDir(scriptDir, loadedScripts);

    if(!didOpenScriptDir) {
        screenLog.logf("Failed to open script directory.");
    } else {
        screenLog.logf("%d script(s) loaded", loadedScripts.size());
    }
}

void advance() {
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

void unload() {
    for(GameScript &script : loadedScripts) {
        script.unload();
    }
}

hookf(advanceGameScripts, Memory::Addresses::advanceGameScripts, {
    // TODO: Move to game load sequence.

    // If this is the first call, we need to load our own scripts.
    // Loading on the first FDE cycle ensures that the game is ready.
    static bool customScriptsLoaded = false;
    if(!customScriptsLoaded) {
        customScriptsLoaded = true;

        // No config directory at the moment.
        Scripts::load("/var/mobile/Documents/CS", "");
    }

    // Every time the game advances its scripts, we advance our own.
    Scripts::advance();

    // Original behaviour.
    original();
}, void)

void Scripts::hook() {}

}