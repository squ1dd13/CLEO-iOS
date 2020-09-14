#include "Custom/ScriptSystem.hpp"
#include "Game/Memory.hpp"
#include "Game/Menu.hpp"
#include "Game/Script.hpp"
#include "Game/Text.hpp"
#include "Game/Touch.hpp"
#include "Headers/Debug.hpp"
#include "Headers/substrate.h"
#include <mach-o/dyld.h>
#include <os/log.h>
#include <sstream>
#include <vector>
#include <unordered_map>

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

DeclareFunctionType(OptionsMenuFunction, Menu *, Menu *);
static OptionsMenuFunction originalOMF;

DeclareFunctionType(LoadIconFunction, void *, const char *);
static LoadIconFunction loadIcon;

void selectionCallback() {
    Debug::logf("selected");

    std::string s = Text::getGameString("tweak_name");
    Debug::logf("TWS: %s", s.data());
}

Menu *optionsMenuHook(Menu *menu) {
    menu = originalOMF(menu);

    // BUG: Null icon causes a crash because it gets dereferenced at 0x100337948.
    // menu->navItems[0] = NavigationItem();
    const char *carmod = "CARMOD1";
    menu->navItems[0].text = new char[8];
    std::strcpy(menu->navItems[0].text, carmod);
    menu->navItems[0].callback = &selectionCallback;

    // menu->navItems[0].icon = Memory::fetch<LoadIconFunction>(0x1001310c0)("menu_maindisplay");

    return menu;
}

void inject() {
    Debug::logf("ASLR slide is 0x%llx (%llu decimal)", Memory::getASLRSlide(), Memory::getASLRSlide());
    Debug::logf("sizeof(GameScript) = %d", sizeof(GameScript));

    advanceGameScripts = Memory::hook(0x1001d0f40, advanceScripts);
    originalOMF = Memory::hook(0x10033c918, optionsMenuHook);

    Touch::hook();
    Text::hook();
}

void cleanUp() {
    scriptSystem.release();
}