// Objective-C hooking stuff. This replaces the Logos %hook and %orig.
// Lots of this code appeared in some form in https://github.com/Squ1dd13/MaxOS.

#include "ObjectiveC.h"
#include <Foundation/Foundation.h>
#include <iostream>
#include <set>
#import <Logging.h>

// Finds the original implementation of the method and calls it.
void *callOrig(SEL hookedSelector, id target, ...) {
    Class targetClass = object_isClass(target) ? target : object_getClass(target);

    std::string selectorName = sel_getName(hookedSelector);
    std::string renamedOriginal = "original_imp_" + selectorName;

    SEL originalSelector = sel_registerName(renamedOriginal.c_str());

    // Get the method named hookedSelector as an instance method.
    bool isClassMethod = false;
    Method method = class_getInstanceMethod(targetClass, hookedSelector);

    // If the instance method is null, it must be a class method.
    if (!method_getImplementation(method)) {
        method = class_getClassMethod(targetClass, originalSelector);
        isClassMethod = true;
    }

    // TODO: Throw here?
    if (!method)
        return nullptr;

    // Create an NSMutableArray from the arguments.
    size_t argumentCount = method_getNumberOfArguments(method) - 2;
    NSMutableArray *arguments = [NSMutableArray array];

    va_list args;
    va_start(args, target);

    for (size_t i = 0; i < argumentCount; ++i) {
        void *arg = va_arg(args, void *);
        [arguments addObject:[NSValue valueWithPointer:arg]];
    }

    va_end(args);

    NSMethodSignature *sig = [isClassMethod ? targetClass : target methodSignatureForSelector:originalSelector];
    NSInvocation *invocation = [NSInvocation invocationWithMethodSignature:sig];
    [invocation setSelector:originalSelector];

    // TODO: Merge va_arg loop and this one.
    for (size_t i = 0; i < [arguments count]; ++i) {
        id obj = arguments[i];

        void *ptr = [obj pointerValue];
        [invocation setArgument:&ptr atIndex:i + 2];
    }

    [invocation setTarget:target];
    [invocation invoke];

    NSUInteger len = [sig methodReturnLength];

    // We shouldn't be getting a memory leak here... I think...
    void *buffer = std::malloc(len);

    // If the return type is void, return null.
    // This should not be treated as a real value (the caller should know that).
    const char *retEnc = [sig methodReturnType];
    if (strcmp("v", retEnc) == 0) {
        return 0x0;// 0x0 because it looks like a face.
    }

    bool isObject = [@(retEnc) containsString:@"@"];
    [invocation getReturnValue:isObject ? &buffer : buffer];

    return buffer;
}

bool hookMethodName(SEL name, Class target, Class hook) {
    Method originalMethod = class_getInstanceMethod(target, name);
    Method hookMethod = class_getInstanceMethod(hook, name);

    if (not originalMethod or not hookMethod) {
        LogError("either original or hooked method not found");
        return false;
    }

    const char *origType = method_getTypeEncoding(originalMethod);
    const char *hookType = method_getTypeEncoding(hookMethod);

    if (std::strcmp(origType, hookType) != 0) {
        LogError("type encoding mismatch - method %s should return %s, but hook returns %s", sel_getName(name), origType, hookType);
        return false;
    }

    IMP targetImplementation = method_getImplementation(originalMethod);

    // Add the "orig" method to the target class so it can be called from the hook implementation.
    SEL origSelector = NSSelectorFromString([@"original_imp_" stringByAppendingString:NSStringFromSelector(name)]);

    if (not class_addMethod(target, origSelector, targetImplementation, origType)) {
        LogError("failed to add orig method for selector %s to class %s", sel_getName(name), class_getName(target));
        return false;
    }

    class_addMethod(target, name, method_getImplementation(originalMethod), origType);

    IMP previousImplementation = class_replaceMethod(target, name, method_getImplementation(hookMethod), origType);
    return previousImplementation != nullptr;
}

static auto methodComparator = [](const Method &a, const Method &b) {
    return std::hash<std::string>()(sel_getName(method_getName(a))) >
           std::hash<std::string>()(sel_getName(method_getName(b)));
};

std::set<Method, decltype(methodComparator)> getMethods(Class cls) {
    unsigned count {};
    Method *methods = class_copyMethodList(cls, &count);

    std::set<Method, decltype(methodComparator)> methodSet(methodComparator);
    for (unsigned i = 0; i < count; ++i)
        methodSet.insert(methods[i]);

    free(methods);

    return methodSet;
}

bool HookClass(const char *hookName, const char *targetName, bool meta) {
    Class targetClass = meta ? objc_getMetaClass(targetName) : objc_getClass(targetName);

    if (not targetClass) {
        LogError("could not find target class %s", targetName);
        return false;
    }

    Class hookClass = meta ? objc_getMetaClass(hookName) : objc_getClass(hookName);

    if (not hookClass) {
        LogError("could not find hook class %s", hookName);
        return false;
    }

    auto targetMethods = getMethods(targetClass);
    auto hookMethods = getMethods(hookClass);

    bool status = true;
    for (auto &targetMethod : targetMethods) {
        if (hookMethods.count(targetMethod)) {
            // Method in both classes, so hook.
            status &= hookMethodName(method_getName(targetMethod), targetClass, hookClass);
        }
    }

    // If this pass was for instance methods, return the result of doing the class methods.
    // Otherwise, return true because everything was successful on this pass.
    return !meta ? status & HookClass(hookName, targetName, true) : status;
}