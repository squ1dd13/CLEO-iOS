
#include "../Types.h"

enum ConcreteType : uint8 {
    End,
    Signed32,
    GlobalNumber,
    LocalNumber,
    Signed8,
    Signed16,
    Flt32,
    GlobalNumberArray,
    LocalNumberArray,
    String8,
    GlobalString8,
    LocalString8,
    GlobalString8Array,
    LocalString8Array,
    VarString,
    String16,
    GlobalString16,
    LocalString16,
    GlobalString16Array,
    LocalString16Array,
};

union AnyValue {
    uint32 unsignedValue;
    int32 signedValue;
    float floatValue;
    void *pointerValue;
    char *stringValue;
};

#pragma pack(1)
struct WorkingScript {
    WorkingScript *nextScript;
    WorkingScript *previousScript;
    char name[8];
    uint8 * startPointer;
    uint8 * currentPointer;
    uint8 * callStack[8];
    uint16 callStackPos; /* Created by retype action */
    uint8 field_0x6a;
    uint8 field_0x6b;
    uint32 localStorage[42]; /* Created by retype action */
    uint8 field_0x114;
    bool conditionResult; /* Created by retype action */
    uint8 field_0x116;
    uint8 field_0x117;
    uint8 field_0x118;
    uint8 field_0x119;
    uint8 field_0x11a;
    uint8 field_0x11b;
    uint32 activationTime;
    uint16 conditionCount; /* Created by retype action */
    bool instructionIsConditional;
    uint8 field_0x123;
    uint8 field_0x124;
    uint8 field_0x125;
    uint8 field_0x126;
    uint8 field_0x127;
    uint8 field_0x128;
    uint8 field_0x129;
    uint8 field_0x12a;
    uint8 field_0x12b;
    bool localStorageIsGlobalStorage;
};
#pragma options align=reset


// 112

// Based on https://github.com/DK22Pac/plugin-sdk/blob/master/plugin_sa/game_sa/CRunningScript.h
/*
struct Script {
  public:
    Script *nextScript;
    Script *previousScript;
    char name[8];
    void *startAddress;
    void *currentAddress;
    void *addressStack[8];
    uint16 stackSize;

  private:
    uint8 pad[6];

  public:
    AnyValue localVariables[32];
    int timers[2];
    bool active;
    bool conditionalFlag;
    bool useMissionCleanup;
    bool external;
    bool textBlockOverride;

  private:
    uint8 pad_[3];

  public:
    int wakeTime;
    // ??
    unsigned short m_nLogicalOp;
    bool notFlag;
    bool wastedBustedCheck;
    bool wastedOrBusted;

  private:
    uint8 pad__[3];

  public:
    void *sceneSkipAddress;
    bool isMission;

  private:
    uint8 pad___[3];
} __attribute__((packed));
*/