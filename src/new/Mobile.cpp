//
// Created by squ1dd13 on 29/01/2021.
//

#include "Mobile.h"

#include "Mobile.h"
#include "../shared/Memory.h"
#include "../shared/Interface.h"
#include <unordered_map>

template <typename T>
T *getArgumentsArray() {
    return Memory::slid<T *>(0x1007ad690);
}

static std::unordered_map<uint16, Mobile::Implementation> implementations {
    { 0xDD0,  Mobile::getLabelAddress },
    { 0xDD1,  Mobile::getFunctionAddressByName },
    { 0xDD2,  Mobile::contextCallFunction },
    { 0xDD3,  Mobile::contextSetReg },
    { 0xDD4,  Mobile::contextGetReg },
    { 0xDD6,  Mobile::getGameVersion },
    { 0xDD7,  Mobile::getImageBase },
    { 0xDD8,  Mobile::readMemory },
    { 0xDD9,  Mobile::writeMemory },
    { 0xDDC,  Mobile::setMutexVar },
    { 0xDDD,  Mobile::getMutexVar },

    { 0xDE0,  Mobile::getTouchPointState },

    { 0xE1,   Mobile::checkButtonPressed },
    { 0x80E1, Mobile::checkButtonNotPressed },
};

#define InstructionStub(func) \
    void func(Script *script) { /* Log::Print("Warning: %s is a stub. Expect a crash...", __func__); */ }

InstructionStub(Mobile::getLabelAddress);

InstructionStub(Mobile::getFunctionAddressByName);

InstructionStub(Mobile::contextCallFunction);

InstructionStub(Mobile::contextSetReg);

InstructionStub(Mobile::contextGetReg);

InstructionStub(Mobile::getGameVersion);

InstructionStub(Mobile::getImageBase);

InstructionStub(Mobile::readMemory);

InstructionStub(Mobile::writeMemory);

static std::unordered_map<uint32, uint32> mutexVars {};

void Mobile::setMutexVar(Script *script) {
    script->ReadValueArgs(2);

    uint32 *args = getArgumentsArray<uint32>();

//    Log::Print("setMutexVar(value: %d, to: %d)", args[0], args[1]);
    mutexVars[args[0]] = args[1];
}

InstructionStub(Mobile::getMutexVar);

bool processZoneQuery(Script *script, int pointIndex = 1) {
    // Touch zone check.
    script->ReadValueArgs(2);

    // 0x1007ad690 is the argument list address.
    int touchZone = Memory::slid<int *>(0x1007ad690)[pointIndex];

    if (0 < touchZone && touchZone < 10) {
        return Interface::Touch::testZone(touchZone);
    }

//    Log::Print("ignoring invalid touch zone %d", touchZone);
    return false;
}

void Mobile::checkButtonPressed(Script *script) {
    script->UpdateBoolean(processZoneQuery(script));
}

void Mobile::checkButtonNotPressed(Script *script) {
    script->UpdateBoolean(!processZoneQuery(script));
}

void Mobile::getTouchPointState(Script *script) {
    int *destination = (int *)(script->ReadVariableArg());
    *destination = processZoneQuery(script, 0);
}

Mobile::Implementation Mobile::getImplementation(uint16 opcode) {
    auto it = implementations.find(opcode);
    if (it != implementations.end()) {
        return it->second;
    }

    return nullptr;
}