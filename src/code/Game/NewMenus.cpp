#include "Game/NewMenus.hpp"
#include "Core.hpp"
#include "Game/Memory.hpp"
#include "Util/Types.hpp"

DeclareFunctionType(GetIconFunction, NewMenus::Icon *, const char *);
DeclareFunctionType(CreateMenuWithTitle, NewMenus::NewMenu *, NewMenus::NewMenu *, const char *, void *, int, void *);

namespace NewMenus {

NavItem::NavItem(string_ref s) {
    text = new char[s.size()];
    std::strcpy(text, s.c_str());
}

Icon *Icon::get(string_ref name) {
    return Memory::slid<GetIconFunction>(0x1001310c0)(name.c_str());
}

void NewMenu::addNavItem(NavItem &item) {
    // We have to implement this because the original seems to have been 
    //  inlined by the compiler (there's a lot of repeated code in the 
    //  decompilation).

    if(navAllocated < navUsed + 1) {
        // Allocates slightly more than we need in most cases, but reduces future reallocations.
        navAllocated = ((navUsed + 1) * 4) / 3 + 3;

        NavItem *items = (NavItem *)std::malloc(navAllocated * sizeof(NavItem));
        if(navStorage) {
            // Copy the old components to the new storage.
            std::memcpy(items, navStorage, navUsed * sizeof(NavItem));

            // Free the old storage.
            std::free(navStorage);
        }

        // Update the storage.
        navStorage = items;
    }

    // The game sets all of the members of the nav item individually, so we're
    //  deviating from the original implementation here.
    std::memcpy((void *)(uint64(navStorage) + (navUsed++ * sizeof(NavItem))), &item, sizeof(NavItem));
}

NewMenu NewMenu::createEndMenu(string_ref name) {
    NewMenu theMenu;
    Memory::slid<CreateMenuWithTitle>(0x10033d428)(&theMenu, name.c_str(), nullptr, 0, nullptr);

    theMenu.vtable = Memory::slid<MenuVtable *>(0x1005bc4c8 + 0x10);
    

    return theMenu;
}

}; // namespace NewMenus