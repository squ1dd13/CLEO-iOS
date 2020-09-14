#include "Instructions.hpp"
#include "../Game/Memory.hpp"
#include "../Game/Touch.hpp"
#include <unordered_map>

template <typename T>
T *getArgumentsArray() {
    return Memory::slid<T *>(0x1007ad690);
}

static std::unordered_map<uint16, Instructions::Implementation> implementations {
    {0xDD0, Instructions::getLabelAddress},
    {0xDD1, Instructions::getFunctionAddressByName},
    {0xDD2, Instructions::contextCallFunction},
    {0xDD3, Instructions::contextSetReg},
    {0xDD4, Instructions::contextGetReg},
    {0xDD6, Instructions::getGameVersion},
    {0xDD7, Instructions::getImageBase},
    {0xDD8, Instructions::readMemory},
    {0xDD9, Instructions::writeMemory},
    {0xDDC, Instructions::setMutexVar},
    {0xDDD, Instructions::getMutexVar},

    {0xDE0, Instructions::getTouchPointState},

    {0xE1, Instructions::checkButtonPressed},
    {0x80E1, Instructions::checkButtonNotPressed},
};

#define InstructionStub(func) \
    void func(GameScript *script) { Debug::logf("Warning: %s is a stub. Expect a crash...", __func__); }

InstructionStub(Instructions::getLabelAddress);
InstructionStub(Instructions::getFunctionAddressByName);
InstructionStub(Instructions::contextCallFunction);
InstructionStub(Instructions::contextSetReg);
InstructionStub(Instructions::contextGetReg);
InstructionStub(Instructions::getGameVersion);
InstructionStub(Instructions::getImageBase);
InstructionStub(Instructions::readMemory);
InstructionStub(Instructions::writeMemory);

static std::unordered_map<uint32, uint32> mutexVars {};

void Instructions::setMutexVar(GameScript *script) {
    script->readArguments(2);

    uint32 *args = getArgumentsArray<uint32>();

    Debug::logf("setMutexVar(value: %d, to: %d)", args[0], args[1]);
    mutexVars[args[0]] = args[1];
}

InstructionStub(Instructions::getMutexVar);

bool processZoneQuery(GameScript *script, int pointIndex = 1) {
    // Touch zone check.
    script->readArguments(2);

    // 0x1007ad690 is the argument list address.
    int touchZone = Memory::slid<int *>(0x1007ad690)[pointIndex];

    if(0 < touchZone && touchZone < 10) {
        Debug::logf("checking touch zone %d", touchZone);

        bool zoneStatus = Touch::touchAreaPressed(touchZone);
        if(zoneStatus) {
            Debug::logf("zone is pressed");
        }

        return zoneStatus;
    }

    Debug::logf("ignoring invalid touch zone %d", touchZone);
    return false;
}

void Instructions::checkButtonPressed(GameScript *script) {
    script->handleFlag(processZoneQuery(script));
}

void Instructions::checkButtonNotPressed(GameScript *script) {
    script->handleFlag(!processZoneQuery(script));
}

void Instructions::getTouchPointState(GameScript *script) {
    int *destination = (int *)(script->readVariable());
    *destination = processZoneQuery(script, 0);
}

Instructions::Implementation Instructions::getImplementation(uint16 opcode) {
    auto it = implementations.find(opcode);
    if(it != implementations.end()) {
        return it->second;
    }

    return nullptr;
}