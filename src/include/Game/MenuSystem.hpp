#ifndef MENU_SYSTEM
#define MENU_SYSTEM

#include "Util/Types.hpp"

namespace MenuSystem {

struct MenuVtable {
    FunctionMember(func0, void);
    FunctionMember(func1, void);
    FunctionMember(func2, void);
    FunctionMember(func3, void);
    FunctionMember(func4, void);

    // Called before the displayed menu changes. Takes another menu.
    FunctionMember(switchThing, void, void *, void *);

    FunctionMember(func6, void);
    FunctionMember(func7, void);
    FunctionMember(func8, void);
    FunctionMember(func9, void);
    FunctionMember(func10, void);
    FunctionMember(func11, void);
    FunctionMember(func12, void);
    FunctionMember(func13, void);
    FunctionMember(func14, void);
    FunctionMember(func15, void);
    FunctionMember(func16, void);
    FunctionMember(func17, void);
    FunctionMember(func18, void);
    FunctionMember(func19, void);
    FunctionMember(func20, void);
} squished;

class MenuBase {
  public:
    MenuVtable *vtable;
} squished;

struct SettingsRowVtable {
    FunctionMember(func0, void);
    FunctionMember(func1, void);
    FunctionMember(func2, void);
    FunctionMember(func3, void);
    FunctionMember(func4, void);
    FunctionMember(func5, void);
    FunctionMember(performAction, void);
    FunctionMember(func7, void);
    FunctionMember(func8, void);
} squished;

struct Icon {
  private:
    uint8 unknown[100];

  public:
    // Could be a reference count (so unused icons can be released),
    //  but I haven't checked that.
    int useCount;

    static Icon *get(string_ref name);
} squished;

class SettingsRowBase {
  public:
    SettingsRowVtable *vtable;
    char *text;
} squished;

class SettingsButton : SettingsRowBase {
    /* vtable at 0x1005bc7e8 */
  public:
    // The callback is passed the menu and category of the button.
    // This allows the StatsScreen::StatsCat (real name – the symbol isn't stripped)
    //  method to be used as a callback. These arguments do not have to be used.
    void (*callback)();

    // The index in the sidebar of this button. The game sets this to 0
    //  if the button is not in a sidebar, but 0 can also mean the first
    //  category. This is used in the stats menu.
    uint32 category;

  private:
    uint32 padding;
} squished;

struct NavItem {
    Icon *icon;
    char *text;
    void (*callback)();
} squished;

// Menu with members from both navigation and settings menus.
// Not sure how many members were mixed between classes originally,
//  so here they're all mixed.
class MixedMenu : MenuBase {
    void *unkPtr;
    int unkInt;

  public:
    bool hasBackButton;

    // If this is false, an extra check is required before the game adds the
    //  "Resume" button in the main menu. That extra check could be to find 
    //  if there is a saved game, but I'm not sure. Also, if this is true,
    //  the stats button will be added to the main menu.
    bool isGamePaused;

  private:
    uint8 unkBytes1[2];

  public:
    char *nameKey;

    uint32 rowCapacity;
    uint32 rowsUsed;
    SettingsRowBase *rows;

    uint32 unknownCapacity;
    uint32 unknownUsed;
    void *unknownStorage;

  private:
    int unkInt1;

  public:
    bool built;

  private:
    uint8 unkBytes2[3];
    int unkInt2;
    float unkFloat;

    // Allocated with 'new', 0x860 bytes.
    void *unkPtr1;

  public:
    uint32 navCapacity;
    uint32 navUsed;
    NavItem *navItems;

  private:
    void *unkPtr2;
    void *unkPtr3;
} squished;

static_assert(sizeof(MixedMenu) == 120, "wrong size");

}; // namespace MenuSystem

#endif