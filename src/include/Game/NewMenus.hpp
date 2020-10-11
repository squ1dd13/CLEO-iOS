#ifndef NEW_MENUS
#define NEW_MENUS

#include "Util/Types.hpp"

namespace NewMenus {

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
} __attribute__((packed));

class MenuBase {
    public:
    MenuVtable *vtable;
} __attribute__((packed));

// NOTE: Stub type. Use only as a pointer.
struct Icon {
    static Icon *get(string_ref name);
    // explicit Icon(string_ref name);
};

struct NavItem {
    Icon *icon;
    char *text;
    void (*callback)();

    explicit NavItem(string_ref s);
} __attribute__((packed));

static_assert(sizeof(NavItem) == 0x18, "wrong NavItem size");

struct SettingsRow {
};

class NewMenu {
    public:
    MenuVtable *vtable;
    void *unknownIcon;
    int unknownInteger;
    bool unknownBool;

  public:
    bool isGamePaused;

  private:
    char unknownBytes1[2];

  public:
    char *name;

    uint32 rowsAllocated;
    uint32 rowsUsed;
    SettingsRow *rows;

    uint32 unkAllocated;
    uint32 unkUsed;
    void *unkStorage;

  private:
    uint32 unkUInt;

  public:
    bool built;

  private:
    char unkBytes2[3];
    void *unkPtr2;
    void *unkPtr3;

  public:
    uint32 navAllocated;
    uint32 navUsed;
    void *navStorage;

  private:
    void *unkPtr4;
    void *unkPtr5;

  public:
    static NewMenu createEndMenu(string_ref name);

    void addNavItem(NavItem &item);
} __attribute__((packed));

static_assert(sizeof(NewMenu) == 0x78, "menu size must be 120 bytes");

}; // namespace NewMenus

#endif