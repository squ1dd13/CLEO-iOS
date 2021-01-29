//
// Created by squ1dd13 on 08/11/2020.
//

#include "Script.h"

#include "../shared/Addresses.h"
#include "../shared/Memory.h"
#include "ScriptManager.h"
#include "new/Mobile.h"

#include <sys/stat.h>

Script::Script(const std::string &path) {
    // Get the size so we know how much space to allocate.
    // CLEO scripts have some junk at the end (something to do with globals)
    //  that can't be executed, so the file size is not an accurate measure
    //  of the space taken up by instructions. There's enough RAM for that
    //  not to be a problem though.
    struct stat st {};
    stat(path.c_str(), &st);

    auto size = st.st_size;

    Log("loading %s", path.c_str());

    std::FILE *scriptFile = std::fopen(path.c_str(), "rb");

    if (!scriptFile) {
        Log("failed to load script %s (unable to open file)", path.c_str());
    }

    // TODO: Zero-initialise all members.

    // If we don't set activationTime to 0, it will get some junk value that may delay the script's launch.
    activationTime = 0;

    // The script data will be leaked unless Unload() is called.
    auto *data = new uint8[size];
    std::fread(data, 1, size, scriptFile);

    startPointer = currentPointer = data;

    std::fclose(scriptFile);

    static unsigned loadNumber = 0;

    // This is only the name until the script renames itself.
    std::string tempName = "magic" + std::to_string(loadNumber++);
    std::strcpy(name, tempName.c_str());
}

void Script::RunNextBlock() {
    // A 'block' ends when RunNextInstruction() returns a non-zero value.
    while (!RunNextInstruction()) {
        // Nothing
    }
}

uint8 Script::RunNextInstruction() {
    uint16 opcodeMask = *(uint16 *)currentPointer;
    currentPointer += 2;

    // A negative opcode is written when the return value is to be inverted.
    // The actual opcode and therefore operation to perform does not change.
    uint16 opcode = opcodeMask & 0x7fffu;
    invertReturn = opcodeMask & 0x8000u;

    // Check for a custom implementation (for mobile-specific instructions like touch checks).
    Mobile::Implementation customHandler = Mobile::getImplementation(opcode);

    if (customHandler) {
        customHandler(this);
        return 0;
    }

    // The game does some weird magic to work out what script pointer to pass when
    //  the opcode is in range of one of the calculated handlers.
    Script *scriptToPass = this;

    auto handler = FindHandler(opcode, scriptToPass);
    return handler(scriptToPass, opcode);
}

void Script::ReadValueArgs(uint32 count) {
    Memory::call(Memory::Addresses::scriptReadNextArgs, this, count);
}

void *Script::ReadVariableArg() {
    return Memory::call<void *>(Memory::Addresses::scriptReadVariable, this);
}

void Script::UpdateBoolean(int flag) {
    Memory::call(Memory::Addresses::scriptFlagHandler, this, flag);
}

void Script::Unload() {
    delete[] startPointer;
    startPointer = nullptr;
}

Script::~Script() {
    Unload();
}

Script *Script::GetAlternateThis(uint64 handlerOffset) {
    auto handlerTable = Memory::slid<uint64 *>(Memory::Addresses::opcodeHandlerTable);

    // TODO: Figure this one out.
    // The game WILL crash (inconsistently) if this value is not passed instead of 'this'.
    return (Script *)((long long)&this->nextScript +
                      (*(long long *)((long long)handlerTable + handlerOffset + 8) >> 1));
}

Script::OpcodeHandler Script::FindHandler(uint16 opcode, Script *&thisPtr) {
    static auto defaultHandler = Memory::slid<OpcodeHandler>(Memory::Addresses::defaultOpcodeHandler);

    // Opcodes below 0xa8c are handled by functions from a table, and the rest are handled by defaultHandler.
    // The instructions are essentially handled by a giant 'switch' statement, and anything >= a8c goes to
    //  the default case.
    if (opcode >= 0xa8c) {
        return defaultHandler;
    }

    static auto handlerTable = Memory::slid<OpcodeHandler *>(Memory::Addresses::opcodeHandlerTable);

    // https://repl.it/repls/PeriodicGlitteringSampler#main.py
    // This calculation just steps the address offset based on the opcode.
    // I would write it in a nicer way, but it works. (It's copied from Ghidra's decompilation.)
    uint64 handlerOffset = (uint64((opcode & 0x7fffu) * 1374389535llu) >> 33) & 0x3ffffff0;

    thisPtr = thisPtr->GetAlternateThis(handlerOffset);

    if (handlerTable[handlerOffset / 8 + 1]) {
        LogInfo("Address after (index %d + 1) = 0x%x", handlerOffset / 8, handlerTable[handlerOffset / 8 + 1]);
    }

    return handlerTable[handlerOffset / 8];
}

Script::Script(Script &&script) {
    // We can copy all the fields over using memcpy because they're all simple values.
    std::memcpy(this, &script, sizeof(Script));

    // Invalidate everything (including the buffer pointer, which is what we care most about).
    std::fill_n((uint8 *)&script, sizeof(Script), 0);
}