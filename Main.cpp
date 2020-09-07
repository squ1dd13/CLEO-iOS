#include "Game/Script.hpp"
#include "HookManager.hpp"
#include "Headers/Debug.hpp"
#include "substrate.h"
#include <mach-o/dyld.h>
#include "Game/Memory.hpp"
#include <os/log.h>

static WorkingScript theScript;

typedef void (*rna)(WorkingScript *, short);
static rna readArgumentsAddress = rna(0x1001cf474);

typedef void *(*RunScriptFunc)(WorkingScript *);
static RunScriptFunc runScript = RunScriptFunc(/*HookManager::getSlide() + */ 0x1001d1360);

typedef uint8 (*OpcodeHandler)(WorkingScript *, uint16);

uint64 getHandlerTableOffset(unsigned opcode) {
    // TODO: Examine and simplify this calculation.
    return (uint64((opcode & 0x7fff) * 1374389535llu) >> 33) & 0x3ffffff0;
}

WorkingScript *calculatePCVar3(WorkingScript *script, uint64 handlerOffset) {
    uint64 *handlerTable = (uint64 *)(0x1005c11d8 + HookManager::getSlide());
    return (WorkingScript *)((long long)&script->nextScript + (*(long long *)((long long)handlerTable + handlerOffset + 8) >> 1));
}

uint32 fetchScriptTime() {
    return Memory::fetch<uint32>(0x1007d3af8);
}

int nextIfConditionCount(WorkingScript *script) {
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

uint8 processInstruction(WorkingScript *script) {
    uint16 readOpcode = *(uint16 *)script->currentPointer;
    script->currentPointer += 2;

    uint16 actualOpcode = readOpcode & 0x7FFF;
    script->instructionIsConditional = ((readOpcode >> 0xF) & 1);// || (actualOpcode == 0xDF);

    // The default handler is for opcodes outside the automatic handler range.
    OpcodeHandler handler = Memory::slid<OpcodeHandler>(0x10020980c);

    if(actualOpcode < 0xA8C) {
        // We have to find the correct handler.
        void **handlerTable = Memory::slid<void **>(0x1005c11d8);
        uint64 handlerOffset = getHandlerTableOffset(actualOpcode);

        handler = OpcodeHandler(handlerTable[handlerOffset / 8]);
    }

    Debug::assertf(handler != nullptr, "null handler for opcode %x", actualOpcode);
    // Debug::logf("opcode %x has handler %p", actualOpcode, handler);
    uint64 instructionOffset = (uint64(script->currentPointer) - 2) - uint64(script->startPointer);
    // Debug::logf("%x: %04x", instructionOffset, actualOpcode);

    // FIXME: We should be passing pCVar3 for < A8C opcodes.
    uint8 result = handler(script, actualOpcode);
    if(script->conditionResult) {
        Debug::logf("flag register set");
    }

    return result;
}

void executeBlock(WorkingScript *script) {
    Debug::logf("CRS($%s)", script->name);

    // Each call to executeBlock() executes a block of instructions (duh...).
    // The end of the block is whenever processInstruction() returns a non-zero value.
    bool result = false;

    do {
        result = processInstruction(script);
    } while(!result);

    Debug::logf("CRS block ended");
}

WorkingScript loadScript(const std::string &path) {
    FILE *f = std::fopen(path.c_str(), "rb");
    std::fseek(f, 0, SEEK_END);
    long fsize = std::ftell(f);
    std::rewind(f);

    // Memory leak here but it's ok for now.
    uint8 *scriptData = new uint8[fsize];
    std::fread(scriptData, 1, fsize, f);
    std::fclose(f);

    WorkingScript script;
    std::strcpy(script.name, "magic");
    script.startPointer = (uint8 *)scriptData;
    script.currentPointer = (uint8 *)scriptData;

    Debug::logf("magic script starts at 0x%x", scriptData);

    // customRunScript(&script);
    // runScript(&script);
    return script;
}

typedef void (*AdvanceFunc)();
static AdvanceFunc advanceScriptsOrig;

// Hooks 0x1001d0f40.
// We hook advanceScripts() because it's easier to handle the execution of our own scripts
//  than it is to integrate them into the game enough that they execute properly.
// If we're executing our own scripts we can also track what's going on more easily,
//  which helps when debugging.
void advanceScripts() {
    // Load the script if required.
    // NOTE: This will also reload the script if the instruction pointer is set to null, which may be undesired.
    if(theScript.currentPointer == nullptr) {
        Debug::logf("script not loaded, loading now");
        theScript = loadScript("/var/mobile/Media/Documents/top_down.csa");
    }

    // The script's activation time is the next time it will get focus.
    // wait(n) for any n != 0 offsets the activation time by n and returns 1
    //  to stop the current execution cycle. When n == 0, wait() returns zero
    //  and execution continues.
    if(theScript.activationTime <= fetchScriptTime()) {
        executeBlock(&theScript);
    }

    // Do the rest.
    advanceScriptsOrig();
}

typedef void (*ConditionHandler)(WorkingScript *script, int flag);
static ConditionHandler conditionHandlerOrig;

void conditionStuff(WorkingScript *script, int flag) {
    // Simplified reimplementation (from decompiled code).
    bool boolFlag = flag;

    if(script->instructionIsConditional) {
        boolFlag = !flag;
    }

    uint16 conditionCount = script->conditionCount;

    if(conditionCount == 0) {
        script->conditionResult = boolFlag;
        return;
    }

    uint16 conditionsLeft = 0;

    if(conditionCount < 9) {
        // "IF AND"
        script->conditionResult &= boolFlag;
        conditionsLeft = conditionCount - 1;

        if(conditionCount == 1) {
            script->conditionCount = 0;
            return;
        }
    } else {
        // "IF OR"
        if(conditionCount - 21 >= 7) {
            return;
        }

        script->conditionResult |= boolFlag;
        
        if(conditionCount == 21) {
            script->conditionCount = 0;
            return;
        }

        conditionsLeft = conditionCount - 1;
    }

    script->conditionCount = conditionsLeft;
}

void conditionHandlerHook(WorkingScript *script, int flag) {
    if(script == &theScript) {
        Debug::logf("condition handler called with flag %d", flag);
        if(script->conditionCount == 0 && !script->instructionIsConditional) {
            script->conditionCount = 1;
            script->instructionIsConditional = true;
        }
    }

    // conditionHandlerOrig(script, flag);
    conditionStuff(script, flag);
}

void runHooks() {
    Debug::logf("ASLR slide is 0x%llx (%llu decimal)", HookManager::getSlide(), HookManager::getSlide());
    Debug::logf("sizeof(WorkingScript) = %d", sizeof(WorkingScript));

    advanceScriptsOrig = AdvanceFunc(HookManager::HookAddress(0x1001d0f40, (void *)advanceScripts));
    // conditionHandlerOrig = ConditionHandler(HookManager::HookAddress(0x1001df890, (void *)conditionHandlerHook));
}