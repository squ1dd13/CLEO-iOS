//
// Created by squ1dd13 on 08/11/2020.
//

#pragma once

class ScriptManager {
public:
    static void Init();

    static void LoadScript(string_ref path);
    static void AdvanceScripts();

    // Returns the script time, which is used to decide whether a script should continue execution
    //  after a 'wait' call.
    static uint32 GetScriptTime();

    static void UnloadAll();
};

