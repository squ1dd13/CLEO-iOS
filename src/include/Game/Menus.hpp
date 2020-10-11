// 10033c918 for loadOptionsMenu

#ifndef UI_MENU_HEADER
#define UI_MENU_HEADER

#include "Core.hpp"

namespace Menus {

DeclareFunctionType(SelectionCallback, void);

struct Button {
    void *unkPtr;
    const char *text;
    SelectionCallback callback;
    uint32 unkNumber;
    uint8 unkBytes[4];    
} __attribute__((packed));

static_assert(sizeof(Button) == 0x20, "wrong size for Button struct");

// Navigation items are the side-scrolling options you get in the pause menu 
//  and the first screen of the "Options" menu.
struct NavigationItem {
    // Type not added until most struct fields are known for the icons.
    void *icon;

    // GXT key.
    char *text;

    // Called when the item is touched (or something else depending on the controls).
    SelectionCallback callback;
} __attribute__((packed));

struct Menu {
    // uint8 *unknownAddress[16];
    void *addr;
    uint8 unknownBytes[80];
    uint32 allocatedCount;
    uint32 usedCount;
    NavigationItem *navItems;
} __attribute__((packed));

void hook();

void addItemToMenu(void *, void *);

};

#endif