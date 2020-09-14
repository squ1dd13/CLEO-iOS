// 10033c918 for loadOptionsMenu

#ifndef UI_MENU_HEADER
#define UI_MENU_HEADER

#include "../Headers/Types.h"

DeclareFunctionType(SelectionCallback, void);

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
    uint32 strangeIndexRelatedInteger;
    uint32 navItemCount;
    NavigationItem *navItems;
} __attribute__((packed));

#endif