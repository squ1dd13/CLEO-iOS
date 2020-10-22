#include <Game/Addresses.hpp>
#include <Game/Script.hpp>
#include <Custom/Android.hpp>

namespace Addresses = Memory::Addresses;

GameScript GameScript::load(string_ref path) {
    FILE *scriptFile = std::fopen(path.c_str(), "rb");

    // Get the file size.
    std::fseek(scriptFile, 0, SEEK_END);
    long size = std::ftell(scriptFile);
    std::rewind(scriptFile);

    // This gets deleted by GameScript::free.
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

void GameScript::executeBlock() {
    // Each call to executeBlock() executes a block of instructions (duh...).
    // The end of the block is whenever processInstruction() returns a non-zero value.
    bool result;

    do {
        result = executeInstruction();
    } while(!result);
}

typedef uint8(*OpcodeHandler)(GameScript *, uint16);
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

        passedScript = getAlternateScriptPointer(handlerOffset);
    }

    return handler(passedScript, opcode);
}

// From decompiled code.
GameScript *GameScript::getAlternateScriptPointer(uint64 handlerOffset) {
    auto handlerTable = Memory::slid<uint64 *>(Addresses::opcodeHandlerTable);
    return (GameScript *)((long long)&this->nextScript + (*(long long *)((long long)handlerTable + handlerOffset + 8) >> 1));
}

uint32 GameScript::time() {
    return Memory::fetch<uint32>(Addresses::scriptTime);
}

uint64 GameScript::calculateHandlerOffset(unsigned opcode) {
    // https://repl.it/repls/PeriodicGlitteringSampler#main.py
    return (uint64((opcode & 0x7fff) * 1374389535llu) >> 33) & 0x3ffffff0;
}

void GameScript::handleFlag(int flag) {
    Memory::call(Addresses::scriptFlagHandler, this, flag);
}

void GameScript::readArguments(uint32 count) {
    Memory::call(Addresses::scriptReadNextArgs, this, count);
}

void *GameScript::readVariable() {
    return Memory::call<void *>(Addresses::scriptReadVariable, this);
}

void GameScript::unload() const {
    delete[] startPointer;
}