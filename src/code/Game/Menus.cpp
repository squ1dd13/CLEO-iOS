#include "Game/Menus.hpp"
#include "Game/Memory.hpp"
#include "Game/NewMenus.hpp"
#include "Game/Text.hpp"
#include "Util/Types.hpp"
#include <Custom/Scripts.hpp>
#include <cstring>

DeclareFunctionType(OptionsMenuFunction, Menus::Menu *, Menus::Menu *);
static OptionsMenuFunction originalOMF;

DeclareFunctionType(CreateMenuWithTitle, NewMenus::NewMenu *, NewMenus::NewMenu *, const char *, bool);

DeclareFunctionType(LoadIconFunction, void *, const char *);
DeclareFunctionType(PushMenuDown, void, void *);

DeclareFunctionType(CreateAudioOptions, NewMenus::NewMenu *, NewMenus::NewMenu *);

DeclareFunctionType(UsedMenusNotZero, void);

DeclareFunctionType(MenuStuff, void, void *, void *, void *, int, void *);
static MenuStuff origMSF;

static UsedMenusNotZero origUMNZ;
// static LoadIconFunction loadIcon;

struct MenuManager {
    uint8 unknownBytes1[32];
    int allocatedMenus;
    int usedMenus;
    NewMenus::NewMenu **menus;
    NewMenus::NewMenu *currentMenu;
    NewMenus::Icon *backgroundMap,
    *sliderEmpty,
    *sliderFull,
    *sliderThumb,
    *adjback,
    *adjback2;

    uint8 unknownBytes2[72];
} squished;

static_assert(sizeof(MenuManager) == 176, "wrong size");

static MenuManager *menuManager = Memory::slid<MenuManager *>(0x100867750);

void selectionCallback() {
    Debug::logf("selected");
    
    NewMenus::NewMenu *menu = new NewMenus::NewMenu;
    Memory::slid<CreateMenuWithTitle>(0x10033d428)(menu, "Scripts"_gxt, true);

    menu->vtable = Memory::slid<NewMenus::MenuVtable *>(0x1005ca0b0);//0x1005bc4c8 + 0x10);

    static std::vector<std::string> registeredStrings;
    Debug::logf("%d path(s)", Scripts::fileNames.size());
    for(auto &path : Scripts::fileNames) {
        Menus::Button *btn = new Menus::Button;
        btn->unkPtr = (void *)(Memory::fetch<uint64>(0x1005bc7e8) + 0x10);
        auto regd = Text::registerString(path);
        registeredStrings.push_back(regd);
        btn->text = registeredStrings.back().c_str(); //"Custom button"_gxt;
        btn->callback = [](){
            Debug::logf("callback called");
        };

        btn->unkNumber = 0;
        Menus::addItemToMenu(menu, (void *)btn);

        Debug::logf("add %s", path.c_str());
    }



    int *numMenusPtr = Memory::slid<int *>(0x100867774);
    Debug::logf("menus = %d", *numMenusPtr);

    Debug::logf("alt menus = %d", menuManager->usedMenus);

    // Memory::slid<CreateAudioOptions>(0x100344ad0)(menu);


    if(menuManager->usedMenus != 0) {
        Debug::logf("switching");
        menu->vtable->switchThing(menu, menuManager->menus[menuManager->usedMenus - 1]);
        Debug::logf("done switching");
    }

    if(menuManager->currentMenu != nullptr) {
        Debug::logf("current menu exists");
    }

    auto menuPtr = Memory::slid<NewMenus::NewMenu **>(0x100867780);

    if(Memory::fetch<void *>(0x100867780) != nullptr) {
        Debug::logf("pushing menu down");
        Memory::slid<PushMenuDown>(0x100338f5c)(Memory::slid<void *>(0x100867750));
        Debug::logf("pushed down menu");
    }

    Debug::logf("switching menu");
    
    *menuPtr = menu;
}

Menus::Menu *optionsMenuHook(Menus::Menu *menu) {
    menu = originalOMF(menu);

    /*
    const char *carmod = "CARMOD1";
    menu->navItems[0].text = new char[8];
    std::strcpy(menu->navItems[0].text, carmod);
    menu->navItems[0].callback = &selectionCallback;
    */
    // if(menu->allocatedCount < menu->usedCount + 1) {
    //     Debug::logf("Can't add nav menu item: no more space allocated for nav items.");
    //     return menu;
    // }

    // Menus::NavigationItem item = menu->navItems[0];
    // item.callback = &selectionCallback;
    // item.text = new char[24];
    // std::strcpy(item.text, "Script Setup"_gxt);
    
    // menu->navItems[menu->usedCount++] = item;

    NewMenus::NavItem item("Script Setup"_gxt);
    item.callback = &selectionCallback;//Memory::slid<void (*)()>(0x10033c5c0);//&selectionCallback;
    item.icon = NewMenus::Icon::get("menu_maindisplay");
    
    Debug::logf("icon = %p", item.icon);

    NewMenus::NewMenu *newMenu = (NewMenus::NewMenu *)menu;
    newMenu->addNavItem(item);

    // menu->navItems[menu->usedCount - 1].icon = menu->navItems[0].icon;

    // menu->navItems[0].icon = Memory::fetch<LoadIconFunction>(0x1001310c0)("menu_maindisplay");

    return menu;
}

DeclareFunctionType(OriginalControlBuilder, void *, void *, void *, void *, uint32, void *);
OriginalControlBuilder origBCM {};

void buttonCallback() {
    void *fptr = Memory::fetch<void *>(0x1005c9a68);
    Debug::logf("fptr = %p (%p without slide)", fptr, (void *)(uint64(fptr) - Memory::getASLRSlide()));

    Debug::logf("button pressed");
    Debug::logf("thing = %s", Memory::fetch<char *>(0x10091e6f8));

    int *numMenusPtr = Memory::slid<int *>(0x100867774);
    Debug::logf("menus = %d", *numMenusPtr);
}

void umnz() {
    Debug::logf("used menus not zero");
    // return true;//origUMNZ();
}

DeclareFunctionType(StoreMenu, void, void *);
static StoreMenu origSM;

void storeMenuHook(void *manager) {
    Debug::logf("storing menu");
    origSM(manager);
}

void menuStuffHook(void *a, void *b, void *c, int d, void *e) {
    Debug::logf("menuStuff(...)");
    origMSF(a, b, c, d, e);
}

DeclareFunctionType(AddMenuItem, void, void *, void *);
namespace Menus {

void addItemToMenu(void *menuPtr, void *itemPtr) {
    Memory::slid<AddMenuItem>(0x10033d5d4)(menuPtr, itemPtr);
}

void *buildControlsMenuHook(void *menu, void *param_2, void *param_3, uint32 param_4, void *param_5) {
    Debug::logf("buildControlsMenu(%p (%llu), %p (%llu), %p (%llu), %lu, %p (%llu))", menu, param_2, param_3, param_4, param_5);

    void *builtMenu = origBCM(menu, param_2, param_3, param_4, param_5);

    Button *btn = new Button;
    btn->unkPtr = (void *)(Memory::fetch<uint64>(0x1005bc7e8) + 0x10);
    btn->text = "Custom button"_gxt;
    btn->callback = buttonCallback;
    btn->unkNumber = 0;

    addItemToMenu(builtMenu, (void *)btn);
    return builtMenu;
}

void hook() {
    originalOMF = Memory::hook(0x10033c918, optionsMenuHook);
    origBCM = Memory::hook(0x10033d078, buildControlsMenuHook);
    origSM = Memory::hook(0x100338f5c, storeMenuHook);
    // origMSF = Memory::hook(0x1004b6a54, menuStuffHook);
}

};