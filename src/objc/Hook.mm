// Objective-C hooking stuff. This replaces the Logos %hook and %orig.
// Lots of this code appeared in some form in https://github.com/Squ1dd13/MaxOS.

#include <iostream>
#include <set>
#include <Foundation/Foundation.h>
#include "Hook.h"

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
        return 0x0; // 0x0 because it looks like a face.
    }

    bool isObject = [@(retEnc) containsString:@"@"];
    [invocation getReturnValue:isObject ? &buffer : buffer];

    return buffer;
}

bool hookMethodName(SEL name, Class target, Class hook) {
    Method originalMethod = class_getInstanceMethod(target, name);
    Method hookMethod = class_getInstanceMethod(hook, name);

    if (not originalMethod or not hookMethod) {
        std::cout << "either original or hooked method not found\n";
        return false;
    }

    const char *origType = method_getTypeEncoding(originalMethod);
    const char *hookType = method_getTypeEncoding(hookMethod);

    if (std::strcmp(origType, hookType) != 0) {
        std::cout << "type encoding mismatch - method " << sel_getName(name) << " should return " << origType
                  << ", but hook returns " << hookType << '\n';
        return false;
    }

    IMP targetImplementation = method_getImplementation(originalMethod);

    // Add the "orig" method to the target class so it can be called from the hook implementation.
    SEL origSelector = NSSelectorFromString([@"original_imp_" stringByAppendingString:NSStringFromSelector(name)]);

    if (not class_addMethod(target, origSelector, targetImplementation, origType)) {
        std::cout << "failed to add orig method for selector " << sel_getName(name) << " to class "
                  << class_getName(target) << '\n';
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

bool HookManager::hookClassObjC(const char *hookName, const char *targetName, bool meta) {
    Class targetClass = meta ? objc_getMetaClass(targetName) : objc_getClass(targetName);

    if (not targetClass) {
        std::cout << "could not find target class " << targetName << '\n';
        return false;
    }

    Class hookClass = meta ? objc_getMetaClass(hookName) : objc_getClass(hookName);

    if (not hookClass) {
        std::cout << "could not find hook class " << hookName << '\n';
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
    return !meta ? status & hookClassObjC(hookName, targetName, true) : status;
}