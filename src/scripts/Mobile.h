//
// Created by squ1dd13 on 29/01/2021.
//

#pragma once

#include "Script.h"

// Support for stuff added in the Android version.
namespace Scripts::Mobile {

    DeclareFunctionType(Handler, void, Script *);

    // Added with CLEO Android.
    // TODO: Learn about Android addressing modes.
    void GetLabelAddress(Script *script);
    void GetFunctionAddressByName(Script *script);
    void ContextCallFunction(Script *script);
    void ContextSetReg(Script *script);
    void ContextGetReg(Script *script);
    void GetGameVersion(Script *script);
    void GetImageBase(Script *script);
    void ReadMemory(Script *script);
    void WriteMemory(Script *script);
    void SetMutexVar(Script *script);
    void GetMutexVar(Script *script);
    void GetZoneState(Script *script);

    // Reimplemented with new behaviour.
    void IsZonePressed(Script *script);

    // Meta.
    Handler GetHandler(uint16 opcode);
}