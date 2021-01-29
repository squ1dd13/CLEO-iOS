#pragma once

#include <objc/runtime.h>

void *callOrig(SEL hookedSelector, id target, ...);
bool HookClass(const char *hookName, const char *targetName, bool meta = false);

#define orig(...) callOrig(_cmd, self, ##__VA_ARGS__)
#define macro_tostr(x) #x

// Useful for when 'self' needs to be used like an object of the superclass of the hooked class.
#define hookbase(classname, base)                                       \
    interface zzzHook_##classname : base                                \
                                    @end                                \
                                    @implementation zzzHook_            \
    ##classname + (void)load {                                          \
        HookClass(macro_tostr(zzzHook_##classname), #classname, false); \
    }
