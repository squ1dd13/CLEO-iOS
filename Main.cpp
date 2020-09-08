#include "Game/Script.hpp"
#include "Headers/Debug.hpp"
#include "Headers/substrate.h"
#include <mach-o/dyld.h>
#include "Game/Memory.hpp"
#include <os/log.h>
#include "Headers/Util.hpp"
#include <vector>
#include "Custom/ScriptSystem.hpp"

DeclareFunctionType(AdvanceFunction, void);
static AdvanceFunction advanceGameScripts;

static ScriptSystem scriptSystem("/var/mobile/Media/Documents/CustomScripts", "");

void advanceScripts() {
    // We want to load the ScriptSystem only when the game is ready. Therefore,
    //  loading it on the first advanceScripts call makes sense.
    static bool systemLoaded = false;
    if(!systemLoaded) {
        systemLoaded = true;
        scriptSystem.loadScripts();
    }

    // Advance custom scripts.
    scriptSystem.advance();

    // Advance game scripts.
    advanceGameScripts();
}

void inject() {
    Debug::logf("ASLR slide is 0x%llx (%llu decimal)", Memory::getASLRSlide(), Memory::getASLRSlide());
    Debug::logf("sizeof(GameScript) = %d", sizeof(GameScript));

    advanceGameScripts = Memory::hook(0x1001d0f40, advanceScripts);
}

void cleanUp() {
    scriptSystem.release();
}