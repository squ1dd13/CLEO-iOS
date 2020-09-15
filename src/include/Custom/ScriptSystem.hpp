#include "Game/Script.hpp"
#include "Util/Types.h"
#include <vector>

#ifndef SCRIPT_SYSTEM
#define SCRIPT_SYSTEM

class ScriptSystem {
    std::vector<GameScript> loadedScripts;
    std::string scriptDir, configDir;

  public:
    ScriptSystem(string_ref scriptDirectory, string_ref configDirectory);
    void loadScripts();
    
    void advance();
    void release();
};

#endif