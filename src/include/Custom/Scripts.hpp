#include "Game/Script.hpp"
#include "Util/Types.hpp"
#include <vector>

#ifndef SCRIPT_SYSTEM
#define SCRIPT_SYSTEM

namespace Scripts {

extern std::vector<GameScript> loadedScripts;
extern std::vector<std::string> fileNames;

// Load scripts and supporting files from the given directory.
// configDirectory is for future config files.
void load(string_ref scriptDirectory, string_ref configDirectory);

// Stop all scripts and free up resources used by the script system.
void unload();

// Advance scripts.
void advance();

// Hook the game's version of Scripts::advance to add a call to our Scripts::advance.
void hook();

}; // namespace Scripts

#endif