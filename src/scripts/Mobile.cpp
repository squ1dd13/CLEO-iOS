//
// Created by squ1dd13 on 29/01/2021.
//

#include "Mobile.h"

#include "bridge/Memory.h"
#include "user/Touch.h"
#include "Logging.h"

#include <map>

static std::map<uint16, Scripts::Mobile::Handler> implementations {
    { 0xDD0,  Scripts::Mobile::GetLabelAddress },
    { 0xDD1,  Scripts::Mobile::GetFunctionAddressByName },
    { 0xDD2,  Scripts::Mobile::ContextCallFunction },
    { 0xDD3,  Scripts::Mobile::ContextSetReg },
    { 0xDD4,  Scripts::Mobile::ContextGetReg },
    { 0xDD6,  Scripts::Mobile::GetGameVersion },
    { 0xDD7,  Scripts::Mobile::GetImageBase },
    { 0xDD8,  Scripts::Mobile::ReadMemory },
    { 0xDD9,  Scripts::Mobile::WriteMemory },
    { 0xDDC,  Scripts::Mobile::SetMutexVar },
    { 0xDDD,  Scripts::Mobile::GetMutexVar },
    { 0xDE0,  Scripts::Mobile::GetZoneState },
    { 0xE1,   Scripts::Mobile::IsZonePressed },
};

#define InstructionStub(func) \
    void Scripts::Mobile::func(Script *script) { LogWarning("%s is a stub. Expect a crash...", __func__); }

InstructionStub(GetLabelAddress)

InstructionStub(GetFunctionAddressByName)

InstructionStub(ContextCallFunction)

InstructionStub(ContextSetReg)

InstructionStub(ContextGetReg)

InstructionStub(GetGameVersion)

InstructionStub(GetImageBase)

InstructionStub(ReadMemory)

InstructionStub(WriteMemory)

InstructionStub(GetMutexVar)

void Scripts::Mobile::SetMutexVar(Script *script) {
    // The game will crash if we don't read the correct number of arguments to
    //  keep the instruction pointer in the correct place.
    script->ReadValueArgs(2);
}

bool QueryTouchZone(Scripts::Script *script, int pointIndex = 1) {
    // Touch zone check.
    script->ReadValueArgs(2);

    // 0x1007ad690 is the argument list address.
    int touchZone = Memory::Slid<int *>(0x1007ad690)[pointIndex];

    if (0 < touchZone && touchZone < 10) {
        return Touch::TestZone(touchZone);
    }

    LogWarning("ignoring invalid touch zone %d", touchZone);
    return false;
}

void Scripts::Mobile::IsZonePressed(Script *script) {
    script->UpdateBoolean(QueryTouchZone(script));
}

void Scripts::Mobile::GetZoneState(Script *script) {
    int *destination = (int *)(script->ReadVariableArg());
    *destination = QueryTouchZone(script, 0);
}

Scripts::Mobile::Handler Scripts::Mobile::GetHandler(uint16 opcode) {
    auto it = implementations.find(opcode);
    if (it != implementations.end()) {
        return it->second;
    }

    return nullptr;
}