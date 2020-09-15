#include "Game/Menus.hpp"
#include "Game/Text.hpp"

DeclareFunctionType(OptionsMenuFunction, Menus::Menu *, Menus::Menu *);
static OptionsMenuFunction originalOMF;

DeclareFunctionType(LoadIconFunction, void *, const char *);
static LoadIconFunction loadIcon;

void selectionCallback() {
    Debug::logf("selected");

    std::string s = Text::getGameString("tweak_name");
    Debug::logf("TWS: %s", s.data());
}

Menus::Menu *optionsMenuHook(Menus::Menu *menu) {
    menu = originalOMF(menu);

    const char *carmod = "CARMOD1";
    menu->navItems[0].text = new char[8];
    std::strcpy(menu->navItems[0].text, carmod);
    menu->navItems[0].callback = &selectionCallback;

    // menu->navItems[0].icon = Memory::fetch<LoadIconFunction>(0x1001310c0)("menu_maindisplay");

    return menu;
}

namespace Menus {

void hook() {
    originalOMF = Memory::hook(0x10033c918, optionsMenuHook);
}

};