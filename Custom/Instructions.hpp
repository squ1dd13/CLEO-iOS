// Non-standard instruction support and custom instruction implementations.

#ifndef CUSTOM_OPCODES
#define CUSTOM_OPCODES

#include "../Game/Script.hpp"
#include "../Headers/Debug.hpp"
#include "../Headers/Types.h"

namespace Instructions {

DeclareFunctionType(Implementation, void, GameScript *);

// Added with CLEO Android.
void getLabelAddress(GameScript *script);
void getFunctionAddressByName(GameScript *script);
void contextCallFunction(GameScript *script);
void contextSetReg(GameScript *script);
void contextGetReg(GameScript *script);
void getGameVersion(GameScript *script);
void getImageBase(GameScript *script);
void readMemory(GameScript *script);
void writeMemory(GameScript *script);
void setMutexVar(GameScript *script);
void getMutexVar(GameScript *script);
void getTouchPointState(GameScript *script);

// Reimplemented with new behaviour.
void checkButtonPressed(GameScript *script);
void checkButtonNotPressed(GameScript *script);

// Meta.
Implementation getImplementation(uint16 opcode);

}; // namespace Instructions

#endif