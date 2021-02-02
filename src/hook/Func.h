//
// Created by squ1dd13 on 17/01/2021.
//

#include <type_traits>

#define $pasta(x, y)  x##y
#define $pasta2(x, y) $pasta(x, y)

// %ctor
#define Constructor [[maybe_unused]] __attribute__((constructor)) void $pasta2($constructor, __LINE__)()
// %dtor
#define Destructor [[maybe_unused]] __attribute__((destructor)) void $pasta2($destructor, __LINE__)()

/*
 * Uses MSHookFunction by default. To change the function used for hooking, define CustomHookFunction
 * BEFORE including this header. The function must have the following type signature:
 *  void(void *, void *, void **)
 *
 * The parameters are:
 *  1: Pointer to original function.
 *  2: Pointer to replacement implementation.
 *  3: Optional function pointer return for calling the original implementation.
 *
 * You may need to write a wrapper function if your hooking library does not have such a function.
 */

#pragma once

#include <mach-o/dyld.h>

#ifndef CustomHookFunction
#ifndef SUBSTRATE_H_
#include <Substrate.h>
#endif
#endif

namespace $all_function_hooks {
    static constexpr int NoSlide = -2;

    template <typename T>
    static void *$get_clever_addr(T x, int imageIndex = -1) {
        if (std::is_integral<T>::value) {
            if (imageIndex == NoSlide) {
                return (void *)x;
            }

#ifndef CustomSlideImage
            imageIndex = imageIndex == -1 ? 0 : imageIndex;
#else
            imageIndex = imageIndex == -1 ? CustomSlideImage : imageIndex;
#endif
            return (void *)((unsigned long long)x + (unsigned long long)_dyld_get_image_vmaddr_slide(imageIndex));
        }
        return (void *)x;
    }

#ifndef CustomHookFunction
    static void (&$do_hook)(void *, void *, void **) = MSHookFunction;
#else
    void (&$do_hook)(void *, void *, void **) = CustomHookFunction;
#endif
}

// To make it obvious what the second value passed to the hook macro does.
// FIXME: Do we really need this?
#define ImageIndex

#define functionhook                \
    namespace $all_function_hooks { \
                                    \
        namespace

/*
 * Hook...(x) applies the hook to the given address or function.
 * If x is a raw address (e.g. 0x123456), the ASLR slide will be
 *  applied unless 'NoSlide' is passed (e.g. 'HookNoSave(0x123, NoSlide)').
 * A custom image index may be passed to change which image the slide
 *  should be taken from (e.g. 'HookNoSave(0x123, ImageIndex 2)').
 */
#define HookNoSave(...)                                                   \
    __attribute__((constructor)) void $hook_apply() {                     \
        void *$hook_addr;                                                 \
        $hook_addr = (void *)$get_clever_addr(__VA_ARGS__);               \
        $do_hook($hook_addr, (void *)&$the_hook_implementation, nullptr); \
    }                                                                     \
    }
#define HookSave(...)                                                                              \
    __attribute__((constructor)) void $hook_apply() {                                              \
        void *$hook_addr;                                                                          \
        $hook_addr = (void *)$get_clever_addr(__VA_ARGS__);                                        \
        $do_hook($hook_addr, (void *)&$the_hook_implementation, (void **)&$the_original_function); \
    }                                                                                              \
    }
#define Body     $the_hook_implementation
#define Original (*$the_original_function)