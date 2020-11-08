#ifndef PUREHOOK_HEADER
#define PUREHOOK_HEADER

#include <objc/runtime.h>
void *callOrig(SEL hookedSelector, id target, ...);
#define orig(...) callOrig(_cmd, self, ##__VA_ARGS__)

class HookManager {
public:
    static bool hookClassObjC(const char *hookName, const char *targetName, bool meta = false);
};

#define macro_tostr(x) #x
#define hookobject(classname) interface zzzHook_##classname : NSObject \
@end\
@implementation zzzHook_##classname\
+(void)load {\
    HookManager::hookClassObjC(macro_tostr(zzzHook_##classname), #classname, false);\
}

// Useful for when 'self' needs to be used like an object of the superclass of the hooked class.
#define hookbase(classname, base) interface zzzHook_##classname : base \
@end\
@implementation zzzHook_##classname\
+(void)load {\
    HookManager::hookClassObjC(macro_tostr(zzzHook_##classname), #classname, false);\
}

// TODO: Add %hookf stuff here.

#endif