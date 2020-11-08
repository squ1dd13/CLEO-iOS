//
// Created by squ1dd13 on 08/11/2020.
//

#pragma once
#include "other/Types.h"

class Script {
public:
    Script *nextScript;
    Script *previousScript;

    char name[8];

    uint8 *startPointer;
    uint8 *currentPointer;

    uint8 *callStack[8];
    uint16 callStackPos;

private:
    uint8 field_0x6A, field_0x6B;

public:
    // Unsure about size here (probably really 32 and not 42, but we don't use this ATM anyway).
    uint32 localStorage[42];

private:
    uint8 field_0x114;

public:
    bool conditionResult;

private:
    uint8 field_0x116,
        field_0x117,
        field_0x118,
        field_0x119,
        field_0x11A,
        field_0x11B;

public:
    // When the script will next receive focus.
    uint32 activationTime;
    uint16 conditionCount;
    bool invertReturn;

private:
    uint8 field_0x123,
        field_0x124,
        field_0x125,
        field_0x126,
        field_0x127,
        field_0x128,
        field_0x129,
        field_0x12A,
        field_0x12B;

public:
    bool localStorageIsGlobalStorage;

    explicit Script(string_ref path);

    void RunNextBlock();
    uint8 RunNextInstruction();

    void ReadValueArgs(uint32 count);
    void *ReadVariableArg();

    void UpdateBoolean(int flag);

    void Unload();

    ~Script();
private:
    using OpcodeHandler = uint8(*)(Script *, uint16);

    Script *GetAlternateThis(uint64 handlerOffset);
    static OpcodeHandler FindHandler(uint16 opcode, Script *&thisPtr);
};

