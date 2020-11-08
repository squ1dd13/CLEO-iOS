//
// Created on 21/10/2020.
//

#ifndef CSIOS_CMAKE_ADDRESSES_H
#define CSIOS_CMAKE_ADDRESSES_H

#include "shared/Memory.h"
#define NameAddress(address, name) constexpr unsigned long long name = address

// Using Memory::Addresses to get memory addresses is longer to type,
//  but improves code readability by explicitly stating what the address means.
// It also means that people can use this file as a reference for iOS memory addresses.

namespace Memory::Addresses {

NameAddress(0x1001df890, scriptFlagHandler);
NameAddress(0x1001cf474, scriptReadNextArgs);
NameAddress(0x1001cfb04, scriptReadVariable);

NameAddress(0x1005c11d8, opcodeHandlerTable);
NameAddress(0x10020980c, defaultOpcodeHandler);

NameAddress(0x1007d3af8, scriptTime);

NameAddress(0x1001d0f40, advanceGameScripts);

NameAddress(0x1004e831c, handleTouch);

}

#endif //CSIOS_CMAKE_ADDRESSES_H
