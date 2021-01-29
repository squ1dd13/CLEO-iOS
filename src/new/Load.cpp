//
// Created by squ1dd13 on 11/01/2021.
//

#include "Hook/NotLogos.h"
#include "scripts/ScriptManager.h"
#include "shared/Interface.h"
#include "shared/Memory.h"
#include "shared/Text.h"

functionhook GameLoadHook {
    void Original(const char *);

    // FIXME: Probably runs again when the player loads up another game.
    void Body(const char *datPath) {
        Original(datPath);

        Interface::Touch::interceptTouches = true;
        ScriptManager::Init();
    }

    HookSave(0x100240178)
}

Constructor {
    Log("ASLR slide is 0x%llx (%llu decimal)", Memory::getASLRSlide(), Memory::getASLRSlide());
    Text::hook();
}

Destructor {
    ScriptManager::UnloadAll();
}