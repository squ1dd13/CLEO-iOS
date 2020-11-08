// Support for stuff added in the Android version.

#ifndef CUSTOM_OPCODES
#define CUSTOM_OPCODES

#include "Script.h"
#include "Core.h"

namespace Android {

DeclareFunctionType(Implementation, void, Script *);

// Added with CLEO Android.
// TODO: Learn about Android addressing modes.
void getLabelAddress(Script *script);
void getFunctionAddressByName(Script *script);
void contextCallFunction(Script *script);
void contextSetReg(Script *script);
void contextGetReg(Script *script);
void getGameVersion(Script *script);
void getImageBase(Script *script);
void readMemory(Script *script);
void writeMemory(Script *script);
void setMutexVar(Script *script);
void getMutexVar(Script *script);
void getTouchPointState(Script *script);

// Reimplemented with new behaviour.
void checkButtonPressed(Script *script);
void checkButtonNotPressed(Script *script);

// Meta.
Implementation getImplementation(uint16 opcode);

}; // namespace Android

#endif