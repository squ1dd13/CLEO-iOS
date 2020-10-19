#include "Game/Script.hpp"
#include "Game/Memory.hpp"
#include "Game/Touch.hpp"
#include <cstring>
#include "Custom/Android.hpp"

GameScript GameScript::load(const std::string &path) {
    // TODO: Remove normal limits on scripts - increase call stack size and local variable storage space.

    FILE *file = std::fopen(path.c_str(), "rb");
    std::fseek(file, 0, SEEK_END);
    long size = std::ftell(file);
    std::rewind(file);

    uint8 *scriptData = new uint8[size];
    std::fread(scriptData, 1, size, file);
    std::fclose(file);

    GameScript script;

    static int scriptn = 0;

    // TODO: Check if name collisions cause issues.
    std::strcpy(script.name, ("magic" + std::to_string(scriptn++)).c_str());
    script.startPointer = (uint8 *)scriptData;
    script.currentPointer = (uint8 *)scriptData;

    return script;
}

uint32 GameScript::time() {
    return Memory::fetch<uint32>(0x1007d3af8);
}

void GameScript::executeBlock() {
    // Each call to executeBlock() executes a block of instructions (duh...).
    // The end of the block is whenever processInstruction() returns a non-zero value.
    bool result;

    do {
        result = executeInstruction();
    } while(!result);
}

uint64 GameScript::calculateHandlerOffset(unsigned opcode) {
    // TODO: Examine and simplify this calculation.
    // https://www.desmos.com/calculator/xyeaqddxgt
    // https://repl.it/repls/PeriodicGlitteringSampler#main.py
    return (uint64((opcode & 0x7fff) * 1374389535llu) >> 33) & 0x3ffffff0;
}

DeclareFunctionType(OpcodeHandler, uint8, GameScript *, uint16);

int getIfConditions(GameScript *script) {
    // Read the byte after the type identifier.
    int numType = int(*(char *)(uint64(script->currentPointer) + 1));

    if(numType == 0) {
        return 1;
    } else if(numType < 8) {
        return numType + 1;
    } else if(numType < 28) {
        return numType - 19;
    }

    return -1;
}

// 0x1001df890
DeclareFunctionType(ProcessBool, void, GameScript *, int);
void GameScript::handleFlag(int flag) {
    Memory::slid<ProcessBool>(0x1001df890)(this, flag);
}

DeclareFunctionType(ReadNextArguments, void, GameScript *, uint32);
void GameScript::readArguments(uint32 count) {
    Memory::slid<ReadNextArguments>(0x1001cf474)(this, count);
}

DeclareFunctionType(ReadVariable, void *, GameScript *);
void *GameScript::readVariable() {
    return Memory::slid<ReadVariable>(0x1001cfb04)(this);
}

GameScript *calculatePCVar3(GameScript *script, uint64 handlerOffset) {
    uint64 *handlerTable = Memory::slid<uint64 *>(0x1005c11d8);
    return (GameScript *)((long long)&script->nextScript + (*(long long *)((long long)handlerTable + handlerOffset + 8) >> 1));
}

uint8 GameScript::executeInstruction() {
    uint16 readOpcode = *(uint16 *)currentPointer;
    currentPointer += 2;

    uint16 actualOpcode = readOpcode & 0x7FFF;
    invertReturn = ((readOpcode >> 0xF) & 1);

//    Debug::logf("opcode %x (from %x)", actualOpcode, readOpcode);

    // The default handler is for opcodes outside the automatic handler range.
    OpcodeHandler handler = Memory::slid<OpcodeHandler>(0x10020980c);

    auto customHandler = Android::getImplementation(actualOpcode);
    if(customHandler) {
//        Debug::logf("opcode %x has a custom implementation", actualOpcode);
        customHandler(this);
        return 0;
    }

//    Debug::logf("  computing handler offset...");
    if(actualOpcode < 0xA8C) {
        // We have to find the correct handler.
        void **handlerTable = Memory::slid<void **>(0x1005c11d8);
        uint64 handlerOffset = calculateHandlerOffset(actualOpcode);

        handler = OpcodeHandler(handlerTable[handlerOffset / 8]);
    }

//    Debug::assertf(handler != nullptr, "null handler for opcode %x", actualOpcode);

    // _FIXME: We should be passing pCVar3 for < A8C opcodes.
    // TODO: Clean up.
//    Debug::logf("  calling handler...");
    uint8 handlerReturn = handler(actualOpcode < 0xA8C ? calculatePCVar3(this, calculateHandlerOffset(actualOpcode)) : this, actualOpcode);

    return handlerReturn;
}

void GameScript::release() {
    delete[] startPointer;
}