#include "Android.h"
#include "../shared/Memory.h"
#include "../shared/Interface.h"
#include <unordered_map>

template <typename T>
T *getArgumentsArray() {
    return Memory::slid<T *>(0x1007ad690);
}

static std::unordered_map<uint16, Android::Implementation> implementations {
    { 0xDD0,  Android::getLabelAddress },
    { 0xDD1,  Android::getFunctionAddressByName },
    { 0xDD2,  Android::contextCallFunction },
    { 0xDD3,  Android::contextSetReg },
    { 0xDD4,  Android::contextGetReg },
    { 0xDD6,  Android::getGameVersion },
    { 0xDD7,  Android::getImageBase },
    { 0xDD8,  Android::readMemory },
    { 0xDD9,  Android::writeMemory },
    { 0xDDC,  Android::setMutexVar },
    { 0xDDD,  Android::getMutexVar },

    { 0xDE0,  Android::getTouchPointState },

    { 0xE1,   Android::checkButtonPressed },
    { 0x80E1, Android::checkButtonNotPressed },
};

#define InstructionStub(func) \
    void func(Script *script) { /* Log("Warning: %s is a stub. Expect a crash...", __func__); */ }

InstructionStub(Android::getLabelAddress);

InstructionStub(Android::getFunctionAddressByName);

InstructionStub(Android::contextCallFunction);

InstructionStub(Android::contextSetReg);

InstructionStub(Android::contextGetReg);

InstructionStub(Android::getGameVersion);

InstructionStub(Android::getImageBase);

InstructionStub(Android::readMemory);

InstructionStub(Android::writeMemory);

static std::unordered_map<uint32, uint32> mutexVars {};

void Android::setMutexVar(Script *script) {
    script->ReadValueArgs(2);

    uint32 *args = getArgumentsArray<uint32>();

//    Log("setMutexVar(value: %d, to: %d)", args[0], args[1]);
    mutexVars[args[0]] = args[1];
}

InstructionStub(Android::getMutexVar);

bool processZoneQuery(Script *script, int pointIndex = 1) {
    // Touch zone check.
    script->ReadValueArgs(2);

    // 0x1007ad690 is the argument list address.
    int touchZone = Memory::slid<int *>(0x1007ad690)[pointIndex];

    if (0 < touchZone && touchZone < 10) {
        return Interface::Touch::testZone(touchZone);
    }

    // https://stackoverflow.com/a/26221725/8622854
    Log("ignoring invalid touch zone %d", touchZone);
//        size_t size = (size_t)std::snprintf(nullptr, 0, format.c_str(), args...) + 1;
//
//        if (size <= 0) {
//            throw std::runtime_error("Formatting error.");
//        }
//
//        char *buf = new char[size];
//        snprintf(buf, size, format.c_str(), args...);
//
//        Commit(std::string(buf, buf + size - 1));
//        delete[] buf;
    return false;
}

void Android::checkButtonPressed(Script *script) {
    script->UpdateBoolean(processZoneQuery(script));
}

void Android::checkButtonNotPressed(Script *script) {
    script->UpdateBoolean(!processZoneQuery(script));
}

void Android::getTouchPointState(Script *script) {
    int *destination = (int *)(script->ReadVariableArg());
    *destination = processZoneQuery(script, 0);
}

Android::Implementation Android::getImplementation(uint16 opcode) {
    auto it = implementations.find(opcode);
    if (it != implementations.end()) {
        return it->second;
    }

    return nullptr;
}