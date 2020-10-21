#include "Game/Script.hpp"
#include <cstring>
#include "Custom/Android.hpp"
#include <Game/Addresses.hpp>

namespace Addresses = Memory::Addresses;

GameScript GameScript::load(string_ref path) {
    FILE *scriptFile = std::fopen(path.c_str(), "rb");

    // Get size.
    std::fseek(scriptFile, 0, SEEK_END);
    long size = std::ftell(scriptFile);
    std::rewind(scriptFile);

    auto scriptData = new uint8[size];
    std::fread(scriptData, 1, size, scriptFile);

    std::fclose(scriptFile);

    static size_t scriptNumber = 0;
    std::string name = "magic" + std::to_string(scriptNumber++);

    GameScript script {};
    std::strcpy(script.name, name.c_str());
    script.startPointer = scriptData;
    script.currentPointer = scriptData;

    return script;
}

uint32 GameScript::time() {
    return Memory::fetch<uint32>(Addresses::scriptTime);
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
    // https://repl.it/repls/PeriodicGlitteringSampler#main.py
    return (uint64((opcode & 0x7fff) * 1374389535llu) >> 33) & 0x3ffffff0;
}

DeclareFunctionType(OpcodeHandler, uint8, GameScript *, uint16);

// 0x1001df890
DeclareFunctionType(ProcessBool, void, GameScript *, int);
void GameScript::handleFlag(int flag) {
    Memory::slid<ProcessBool>(Addresses::scriptFlagHandler)(this, flag);
}

DeclareFunctionType(ReadNextArguments, void, GameScript *, uint32);
void GameScript::readArguments(uint32 count) {
    Memory::slid<ReadNextArguments>(Addresses::scriptReadNextArgs)(this, count);
}

DeclareFunctionType(ReadVariable, void *, GameScript *);
void *GameScript::readVariable() {
    return Memory::slid<ReadVariable>(Addresses::scriptReadVariable)(this);
}

// From decompiled code.
GameScript *getAlternateScriptPointer(GameScript *script, uint64 handlerOffset) {
    auto handlerTable = Memory::slid<uint64 *>(Addresses::opcodeHandlerTable);
    return (GameScript *)((long long)&script->nextScript + (*(long long *)((long long)handlerTable + handlerOffset + 8) >> 1));
}

uint8 GameScript::executeInstruction() {
    uint16 opcodeMask = *(uint16 *)currentPointer;
    currentPointer += 2;

    // When the return value should be inverted, the opcode's sign bit is set.
    // We need to unset the sign bit, but also remember whether it was set.
    uint16 opcode = opcodeMask & 0x7FFF;
    invertReturn = opcodeMask & 0x8000;

    // Find the default handler for opcodes outside the automatic range.
    // This handler is equivalent to the 'default' case in a switch.
    auto handler = Memory::slid<OpcodeHandler>(Addresses::defaultOpcodeHandler);

    // Check for a custom implementation.
    Android::Implementation customHandler = Android::getImplementation(opcode);
    if(customHandler) {
        customHandler(this);
        return 0;
    }

    // The script we pass to the handler varies depending on whether or not we're using
    //  the default handler.
    GameScript *passedScript = this;

    if(opcode < 0xA8C) {
        // We can work out the handlers for opcodes < 0xA8C.
        auto handlerTable = Memory::slid<void **>(Addresses::opcodeHandlerTable);

        uint64 handlerOffset = calculateHandlerOffset(opcode);
        handler = OpcodeHandler(handlerTable[handlerOffset / 8]);

        passedScript = getAlternateScriptPointer(this, handlerOffset);
    }

    return handler(passedScript, opcode);
}

void GameScript::free() const {
    delete[] startPointer;
}