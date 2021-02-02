//
// Created by squ1dd13 on 11/01/2021.
//

#include "hook/Func.h"
#include "user/Touch.h"
#include "scripts/Manager.h"
#include "bridge/Memory.h"
#include "Logging.h"

functionhook GameLoadHook {
    void Original(const char *);

    // FIXME: Probably runs again when the player loads up another game.
    void Body(const char *datPath) {
        Original(datPath);

        Touch::interceptTouches = true;
        Scripts::Manager::Init();
    }

    HookSave(0x100240178)
}

Constructor {
    Log("ASLR slide is 0x%llx (%llu decimal)", Memory::AslrSlide(), Memory::AslrSlide());
}