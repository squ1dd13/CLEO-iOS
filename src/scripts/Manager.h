//
// Created by squ1dd13 on 08/11/2020.
//

#pragma once

#include "Script.h"
#include <set>

namespace Scripts {
    class Manager {
    public:
        static bool Initialized();
        static void Init();

        static void LoadScript(string_ref path);
        static void AdvanceScripts();

        static void Invoke(string_ref name);
        static std::set<std::string> &InvokedScripts();

        // Returns the script time, which is used to decide whether a script should continue execution
        //  after a 'wait' call.
        static uint32 GetScriptTime();
    };

}
